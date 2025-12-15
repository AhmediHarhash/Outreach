"""Lead scoring API routes."""

import logging
from typing import Any, Optional
from uuid import UUID

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..scoring import ScoringEngine, SignalDetector
from ..models.lead import ScoreTier
from ..models.icp import ICPProfile
from ..db.connection import fetch_one, fetch_all

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/v1/score", tags=["scoring"])


class ScoreLeadRequest(BaseModel):
    """Request to score a lead."""

    lead_id: UUID
    user_id: UUID
    icp_id: Optional[UUID] = Field(
        default=None,
        description="ICP profile to use for scoring (uses default if not provided)",
    )
    company_data: dict[str, Any] = Field(..., description="Enriched company data")
    contact_data: Optional[dict[str, Any]] = Field(
        default=None,
        description="Contact/person data",
    )


class ScoreLeadResponse(BaseModel):
    """Response with lead score."""

    lead_id: UUID
    intent_score: int
    fit_score: int
    accessibility_score: int
    total_score: int
    tier: str
    score_breakdown: dict[str, Any]
    active_signals: list[str]
    score_change: Optional[int]


class BatchScoreRequest(BaseModel):
    """Request to score multiple leads."""

    lead_ids: list[UUID]
    user_id: UUID
    icp_id: Optional[UUID] = None


class GetScoresRequest(BaseModel):
    """Request to get scores by tier."""

    user_id: UUID
    tier: Optional[str] = Field(
        default=None,
        description="Filter by tier (hot, warm, nurture, cold)",
    )
    limit: int = Field(default=50, ge=1, le=200)


@router.post("/lead", response_model=ScoreLeadResponse)
async def score_lead(request: ScoreLeadRequest):
    """Calculate score for a single lead.

    Uses the provided company and contact data along with
    detected signals to calculate intent, fit, and accessibility scores.
    """
    try:
        # Load ICP if specified
        icp = None
        if request.icp_id:
            row = await fetch_one(
                "SELECT * FROM icp_profiles WHERE id = $1 AND user_id = $2",
                request.icp_id,
                request.user_id,
            )
            if row:
                icp = ICPProfile.from_db_row(dict(row))
        else:
            # Get default ICP for user
            row = await fetch_one(
                "SELECT * FROM icp_profiles WHERE user_id = $1 AND is_default = true",
                request.user_id,
            )
            if row:
                icp = ICPProfile.from_db_row(dict(row))

        # Get active signals for this lead
        signals = await SignalDetector.get_active_signals(lead_id=request.lead_id)

        # Calculate score
        engine = ScoringEngine(icp=icp)
        score = await engine.score_lead(
            lead_id=request.lead_id,
            company_data=request.company_data,
            contact_data=request.contact_data,
            signals=signals,
        )

        return ScoreLeadResponse(
            lead_id=score.lead_id,
            intent_score=score.intent_score,
            fit_score=score.fit_score,
            accessibility_score=score.accessibility_score,
            total_score=score.total_score,
            tier=score.tier.value,
            score_breakdown=score.score_breakdown.model_dump(),
            active_signals=score.active_signals,
            score_change=score.score_change,
        )

    except Exception as e:
        logger.error(f"Lead scoring failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/batch")
async def batch_score_leads(request: BatchScoreRequest):
    """Score multiple leads in batch.

    Useful for re-scoring leads when ICP changes or
    for initial scoring of imported leads.
    """
    try:
        # Load ICP
        icp = None
        if request.icp_id:
            row = await fetch_one(
                "SELECT * FROM icp_profiles WHERE id = $1 AND user_id = $2",
                request.icp_id,
                request.user_id,
            )
            if row:
                icp = ICPProfile.from_db_row(dict(row))

        engine = ScoringEngine(icp=icp)
        results = []
        errors = []

        for lead_id in request.lead_ids:
            try:
                # Get lead data
                lead_row = await fetch_one(
                    """
                    SELECT l.*, lcs.total_score as current_score
                    FROM leads l
                    LEFT JOIN lead_current_scores lcs ON l.id = lcs.lead_id
                    WHERE l.id = $1 AND l.user_id = $2
                    """,
                    lead_id,
                    request.user_id,
                )

                if not lead_row:
                    errors.append({"lead_id": str(lead_id), "error": "Lead not found"})
                    continue

                # Get signals
                signals = await SignalDetector.get_active_signals(lead_id=lead_id)

                # Score
                score = await engine.score_lead(
                    lead_id=lead_id,
                    company_data=dict(lead_row).get("company_data", {}),
                    contact_data={
                        "email": lead_row.get("email"),
                        "title": lead_row.get("title"),
                    },
                    signals=signals,
                )

                results.append({
                    "lead_id": str(lead_id),
                    "total_score": score.total_score,
                    "tier": score.tier.value,
                    "score_change": score.score_change,
                })

            except Exception as e:
                errors.append({"lead_id": str(lead_id), "error": str(e)})

        return {
            "success": len(errors) == 0,
            "scored": len(results),
            "errors": len(errors),
            "results": results,
            "error_details": errors if errors else None,
        }

    except Exception as e:
        logger.error(f"Batch scoring failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/tiers/{user_id}")
async def get_score_distribution(user_id: UUID):
    """Get distribution of leads across score tiers."""
    try:
        rows = await fetch_all(
            """
            SELECT tier, COUNT(*) as count, AVG(total_score) as avg_score
            FROM lead_current_scores lcs
            JOIN leads l ON lcs.lead_id = l.id
            WHERE l.user_id = $1
            GROUP BY tier
            ORDER BY avg_score DESC
            """,
            user_id,
        )

        distribution = {
            "hot": {"count": 0, "avg_score": 0},
            "warm": {"count": 0, "avg_score": 0},
            "nurture": {"count": 0, "avg_score": 0},
            "cold": {"count": 0, "avg_score": 0},
        }

        for row in rows:
            tier = row["tier"]
            if tier in distribution:
                distribution[tier] = {
                    "count": row["count"],
                    "avg_score": round(row["avg_score"], 1),
                }

        total = sum(d["count"] for d in distribution.values())

        return {
            "user_id": str(user_id),
            "total_leads": total,
            "distribution": distribution,
        }

    except Exception as e:
        logger.error(f"Failed to get score distribution: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/leads/{user_id}")
async def get_scored_leads(
    user_id: UUID,
    tier: Optional[str] = None,
    min_score: Optional[int] = None,
    limit: int = 50,
    offset: int = 0,
):
    """Get scored leads for a user, optionally filtered by tier."""
    try:
        # Build query
        query = """
            SELECT l.*, lcs.total_score, lcs.tier, lcs.intent_score,
                   lcs.fit_score, lcs.accessibility_score, lcs.calculated_at
            FROM leads l
            JOIN lead_current_scores lcs ON l.id = lcs.lead_id
            WHERE l.user_id = $1
        """
        params = [user_id]

        if tier:
            query += f" AND lcs.tier = ${len(params) + 1}"
            params.append(tier)

        if min_score:
            query += f" AND lcs.total_score >= ${len(params) + 1}"
            params.append(min_score)

        query += f" ORDER BY lcs.total_score DESC LIMIT ${len(params) + 1} OFFSET ${len(params) + 2}"
        params.extend([limit, offset])

        rows = await fetch_all(query, *params)

        leads = []
        for row in rows:
            leads.append({
                "id": str(row["id"]),
                "company_name": row["company_name"],
                "contact_name": row["name"],
                "email": row["email"],
                "total_score": row["total_score"],
                "tier": row["tier"],
                "intent_score": row["intent_score"],
                "fit_score": row["fit_score"],
                "accessibility_score": row["accessibility_score"],
                "scored_at": row["calculated_at"].isoformat() if row["calculated_at"] else None,
            })

        return {
            "leads": leads,
            "count": len(leads),
            "limit": limit,
            "offset": offset,
        }

    except Exception as e:
        logger.error(f"Failed to get scored leads: {e}")
        raise HTTPException(status_code=500, detail=str(e))
