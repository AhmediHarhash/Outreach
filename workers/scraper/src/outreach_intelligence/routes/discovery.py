"""Lead discovery API routes."""

import logging
from typing import Any, Optional
from uuid import UUID

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..enrichment import EnrichmentAggregator
from ..scoring import ScoringEngine, SignalDetector
from ..models.icp import ICPProfile
from ..models.discovery import DiscoveredLead, DiscoveryStatus
from ..db.connection import fetch_one, fetch_all, execute_query

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/v1/discover", tags=["discovery"])


class DiscoverLeadsRequest(BaseModel):
    """Request to discover new leads matching ICP."""

    user_id: UUID
    icp_id: Optional[UUID] = Field(
        default=None,
        description="ICP profile to use (uses default if not provided)",
    )
    limit: int = Field(default=25, ge=1, le=100)
    min_score: int = Field(
        default=60,
        ge=0,
        le=100,
        description="Minimum score threshold",
    )
    sources: Optional[list[str]] = Field(
        default=None,
        description="Sources to search (apollo, crunchbase)",
    )

    # API keys
    apollo_key: Optional[str] = None
    crunchbase_key: Optional[str] = None


class DiscoverLeadsResponse(BaseModel):
    """Response with discovered leads."""

    success: bool
    discovered: int
    above_threshold: int
    leads: list[dict[str, Any]]


class ReviewLeadRequest(BaseModel):
    """Request to review a discovered lead."""

    lead_id: UUID
    user_id: UUID
    action: str = Field(
        ...,
        description="Action to take: accept, reject, skip",
    )
    rejection_reason: Optional[str] = None


class SignalSearchRequest(BaseModel):
    """Request to search for signal-based leads."""

    user_id: UUID
    signal_type: str = Field(
        ...,
        description="Signal type: funding, hiring, tech_change",
    )
    days: int = Field(default=30, ge=1, le=90)
    industries: Optional[list[str]] = None
    min_funding: Optional[int] = None
    limit: int = Field(default=50, ge=1, le=100)

    crunchbase_key: Optional[str] = None
    apollo_key: Optional[str] = None


@router.post("", response_model=DiscoverLeadsResponse)
async def discover_leads(request: DiscoverLeadsRequest):
    """Discover new leads matching user's ICP.

    Searches across configured data sources for companies
    that match the ideal customer profile, then scores and
    ranks them.
    """
    try:
        # Load ICP
        icp = None
        icp_row = None

        if request.icp_id:
            icp_row = await fetch_one(
                "SELECT * FROM icp_profiles WHERE id = $1 AND user_id = $2",
                request.icp_id,
                request.user_id,
            )
        else:
            icp_row = await fetch_one(
                "SELECT * FROM icp_profiles WHERE user_id = $1 AND is_default = true",
                request.user_id,
            )

        if icp_row:
            icp = ICPProfile.from_db_row(dict(icp_row))

        if not icp:
            raise HTTPException(
                status_code=400,
                detail="No ICP profile found. Please create an ICP first.",
            )

        # Create aggregator
        aggregator = EnrichmentAggregator(
            user_id=request.user_id,
            apollo_key=request.apollo_key,
            crunchbase_key=request.crunchbase_key,
        )

        if not aggregator.apollo:
            raise HTTPException(
                status_code=400,
                detail="Apollo API key required for lead discovery.",
            )

        # Search for companies matching ICP
        search_result = await aggregator.apollo.search_companies(
            industries=[str(i) for i in icp.filters.industries] if icp.filters.industries else None,
            employee_count_min=icp.filters.company_size_min,
            employee_count_max=icp.filters.company_size_max,
            countries=[str(c) for c in icp.filters.countries] if icp.filters.countries else None,
            limit=request.limit * 2,  # Get extra to filter
        )

        if not search_result.success:
            raise HTTPException(
                status_code=500,
                detail=f"Search failed: {search_result.error}",
            )

        companies = search_result.data.get("companies", [])

        # Score and filter
        engine = ScoringEngine(icp=icp)
        discovered_leads = []

        for company in companies:
            # Check for duplicates
            existing = await fetch_one(
                """
                SELECT id FROM leads WHERE user_id = $1 AND company_domain = $2
                UNION
                SELECT id FROM discovered_leads WHERE user_id = $1 AND company_domain = $2
                """,
                request.user_id,
                company.get("domain"),
            )

            if existing:
                continue

            # Detect signals
            signals = SignalDetector.detect_signals(company)

            # Calculate preliminary score
            import uuid
            temp_lead_id = uuid.uuid4()

            score = await engine.score_lead(
                lead_id=temp_lead_id,
                company_data=company,
                signals=signals,
            )

            if score.total_score >= request.min_score:
                # Save to discovered_leads table
                discovered = DiscoveredLead(
                    user_id=request.user_id,
                    icp_id=icp.id,
                    company_name=company.get("name", "Unknown"),
                    company_domain=company.get("domain"),
                    company_linkedin=company.get("linkedin_url"),
                    company_data=company,
                    preliminary_score=score.total_score,
                    score_breakdown=score.score_breakdown.model_dump(),
                    discovery_signals=[s.model_dump() for s in signals],
                    source="apollo",
                )

                await _save_discovered_lead(discovered)

                discovered_leads.append({
                    "company_name": discovered.company_name,
                    "company_domain": discovered.company_domain,
                    "industry": company.get("industry"),
                    "employee_count": company.get("employee_count"),
                    "funding_stage": company.get("funding_stage"),
                    "score": score.total_score,
                    "tier": score.tier.value,
                    "signals": [s.signal_type.value for s in signals],
                })

                if len(discovered_leads) >= request.limit:
                    break

        await aggregator.close()

        return DiscoverLeadsResponse(
            success=True,
            discovered=len(discovered_leads),
            above_threshold=len(discovered_leads),
            leads=discovered_leads,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Lead discovery failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/signals")
async def discover_by_signals(request: SignalSearchRequest):
    """Discover leads based on specific signals.

    Find companies showing high-intent signals like:
    - Recent funding rounds
    - Active hiring
    - Technology changes
    """
    try:
        if request.signal_type == "funding":
            if not request.crunchbase_key:
                raise HTTPException(
                    status_code=400,
                    detail="Crunchbase API key required for funding signals.",
                )

            aggregator = EnrichmentAggregator(
                user_id=request.user_id,
                crunchbase_key=request.crunchbase_key,
            )

            companies = await aggregator.get_funding_signals(
                days=request.days,
                industries=request.industries,
                limit=request.limit,
            )

            await aggregator.close()

            return {
                "success": True,
                "signal_type": "funding",
                "count": len(companies),
                "companies": companies,
            }

        elif request.signal_type == "hiring":
            if not request.apollo_key:
                raise HTTPException(
                    status_code=400,
                    detail="Apollo API key required for hiring signals.",
                )

            # Use Apollo to find companies actively hiring
            aggregator = EnrichmentAggregator(
                user_id=request.user_id,
                apollo_key=request.apollo_key,
            )

            # Search for companies with open positions
            result = await aggregator.apollo.search_companies(
                industries=request.industries,
                limit=request.limit,
            )

            if result.success:
                # Filter to only hiring companies
                hiring_companies = [
                    c for c in result.data.get("companies", [])
                    if c.get("is_hiring") and c.get("open_positions", 0) > 0
                ]

                await aggregator.close()

                return {
                    "success": True,
                    "signal_type": "hiring",
                    "count": len(hiring_companies),
                    "companies": hiring_companies,
                }

            await aggregator.close()
            return {
                "success": False,
                "error": result.error,
            }

        else:
            raise HTTPException(
                status_code=400,
                detail=f"Unknown signal type: {request.signal_type}",
            )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Signal discovery failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/pending/{user_id}")
async def get_pending_discoveries(
    user_id: UUID,
    limit: int = 50,
    offset: int = 0,
):
    """Get discovered leads pending review."""
    try:
        rows = await fetch_all(
            """
            SELECT * FROM discovered_leads
            WHERE user_id = $1 AND status = 'new'
            ORDER BY preliminary_score DESC
            LIMIT $2 OFFSET $3
            """,
            user_id,
            limit,
            offset,
        )

        leads = []
        for row in rows:
            leads.append({
                "id": str(row["id"]),
                "company_name": row["company_name"],
                "company_domain": row["company_domain"],
                "contact_name": row["contact_name"],
                "contact_title": row["contact_title"],
                "score": row["preliminary_score"],
                "source": row["source"],
                "discovered_at": row["discovered_at"].isoformat(),
            })

        # Get total count
        count_row = await fetch_one(
            "SELECT COUNT(*) as count FROM discovered_leads WHERE user_id = $1 AND status = 'new'",
            user_id,
        )

        return {
            "leads": leads,
            "count": len(leads),
            "total": count_row["count"] if count_row else 0,
        }

    except Exception as e:
        logger.error(f"Failed to get pending discoveries: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/review")
async def review_discovered_lead(request: ReviewLeadRequest):
    """Review and act on a discovered lead.

    Actions:
    - accept: Convert to actual lead
    - reject: Mark as rejected with reason
    - skip: Mark as reviewed but no action
    """
    try:
        # Get the discovered lead
        row = await fetch_one(
            "SELECT * FROM discovered_leads WHERE id = $1 AND user_id = $2",
            request.lead_id,
            request.user_id,
        )

        if not row:
            raise HTTPException(status_code=404, detail="Discovered lead not found")

        if request.action == "accept":
            # Create actual lead from discovered lead
            lead_id = await _convert_to_lead(dict(row))

            # Update discovered lead status
            await execute_query(
                """
                UPDATE discovered_leads
                SET status = 'accepted', reviewed_at = NOW(), accepted_at = NOW(),
                    converted_lead_id = $2
                WHERE id = $1
                """,
                request.lead_id,
                lead_id,
            )

            return {
                "success": True,
                "action": "accepted",
                "lead_id": str(lead_id),
            }

        elif request.action == "reject":
            await execute_query(
                """
                UPDATE discovered_leads
                SET status = 'rejected', reviewed_at = NOW(), rejection_reason = $2
                WHERE id = $1
                """,
                request.lead_id,
                request.rejection_reason,
            )

            return {
                "success": True,
                "action": "rejected",
            }

        elif request.action == "skip":
            await execute_query(
                """
                UPDATE discovered_leads
                SET status = 'reviewed', reviewed_at = NOW()
                WHERE id = $1
                """,
                request.lead_id,
            )

            return {
                "success": True,
                "action": "skipped",
            }

        else:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid action: {request.action}",
            )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Failed to review lead: {e}")
        raise HTTPException(status_code=500, detail=str(e))


async def _save_discovered_lead(lead: DiscoveredLead) -> UUID:
    """Save discovered lead to database."""
    import json
    import uuid

    lead_id = uuid.uuid4()

    await execute_query(
        """
        INSERT INTO discovered_leads (
            id, user_id, icp_id, company_name, company_domain, company_linkedin,
            contact_name, contact_title, contact_email, contact_linkedin,
            company_data, contact_data, preliminary_score, score_breakdown,
            discovery_signals, status, source, discovered_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        """,
        lead_id,
        lead.user_id,
        lead.icp_id,
        lead.company_name,
        lead.company_domain,
        lead.company_linkedin,
        lead.contact_name,
        lead.contact_title,
        lead.contact_email,
        lead.contact_linkedin,
        json.dumps(lead.company_data),
        json.dumps(lead.contact_data),
        lead.preliminary_score,
        json.dumps(lead.score_breakdown),
        json.dumps(lead.discovery_signals),
        lead.status.value,
        lead.source,
    )

    return lead_id


async def _convert_to_lead(discovered: dict) -> UUID:
    """Convert discovered lead to actual lead."""
    import json
    import uuid

    lead_id = uuid.uuid4()

    await execute_query(
        """
        INSERT INTO leads (
            id, user_id, company_name, company_domain, name, title, email,
            linkedin_url, company_data, status, source, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'new', $10, NOW())
        """,
        lead_id,
        discovered["user_id"],
        discovered["company_name"],
        discovered["company_domain"],
        discovered["contact_name"],
        discovered["contact_title"],
        discovered["contact_email"],
        discovered["contact_linkedin"],
        json.dumps(discovered["company_data"]) if discovered["company_data"] else "{}",
        discovered["source"] or "discovery",
    )

    return lead_id
