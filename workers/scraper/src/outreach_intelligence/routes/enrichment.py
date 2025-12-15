"""Enrichment API routes."""

import logging
from typing import Any, Optional
from uuid import UUID

from fastapi import APIRouter, HTTPException, Depends, Header
from pydantic import BaseModel, Field

from ..enrichment import EnrichmentAggregator
from ..scoring.signals import SignalDetector

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/v1/enrich", tags=["enrichment"])


# Request/Response models
class EnrichCompanyRequest(BaseModel):
    """Request to enrich a company."""

    domain: str = Field(..., description="Company domain (e.g., stripe.com)")
    sources: Optional[list[str]] = Field(
        default=None,
        description="Specific sources to use (apollo, clearbit, hunter, crunchbase)",
    )
    user_id: UUID = Field(..., description="User ID for API key lookup")

    # API keys can be provided directly (for testing) or fetched from DB
    apollo_key: Optional[str] = None
    clearbit_key: Optional[str] = None
    hunter_key: Optional[str] = None
    crunchbase_key: Optional[str] = None


class EnrichCompanyResponse(BaseModel):
    """Response with enriched company data."""

    success: bool
    domain: str
    data: dict[str, Any]
    sources_used: list[str]
    credits_used: int
    signals_detected: int = 0


class FindContactsRequest(BaseModel):
    """Request to find contacts at a company."""

    domain: str
    titles: Optional[list[str]] = Field(
        default=None,
        description="Filter by job titles",
    )
    departments: Optional[list[str]] = Field(
        default=None,
        description="Filter by departments",
    )
    limit: int = Field(default=5, ge=1, le=25)
    user_id: UUID

    # API keys
    apollo_key: Optional[str] = None
    hunter_key: Optional[str] = None


class FindContactsResponse(BaseModel):
    """Response with found contacts."""

    success: bool
    domain: str
    contacts: list[dict[str, Any]]
    total: int
    sources_used: list[str]


class VerifyEmailRequest(BaseModel):
    """Request to verify an email address."""

    email: str
    user_id: UUID
    hunter_key: Optional[str] = None


class VerifyEmailResponse(BaseModel):
    """Response with email verification result."""

    success: bool
    email: str
    valid: Optional[bool]
    confidence: float = 0.0
    details: Optional[dict[str, Any]] = None


class FindEmailRequest(BaseModel):
    """Request to find email for a person."""

    domain: str
    first_name: str
    last_name: str
    user_id: UUID
    hunter_key: Optional[str] = None


@router.post("/company", response_model=EnrichCompanyResponse)
async def enrich_company(request: EnrichCompanyRequest):
    """Enrich company data from multiple sources.

    Aggregates data from Apollo, Clearbit, Hunter, and Crunchbase
    to provide comprehensive company intelligence.
    """
    try:
        # Create aggregator with provided keys
        aggregator = EnrichmentAggregator(
            user_id=request.user_id,
            apollo_key=request.apollo_key,
            clearbit_key=request.clearbit_key,
            hunter_key=request.hunter_key,
            crunchbase_key=request.crunchbase_key,
        )

        if not aggregator.available_sources:
            raise HTTPException(
                status_code=400,
                detail="No enrichment sources configured. Please provide API keys.",
            )

        # Enrich company
        result = await aggregator.enrich_company(
            domain=request.domain,
            sources=request.sources,
        )

        # Detect signals from enriched data
        signals = SignalDetector.detect_signals(result)
        if signals:
            await SignalDetector.save_signals(
                signals,
                company_domain=request.domain,
            )

        # Close connections
        await aggregator.close()

        return EnrichCompanyResponse(
            success="error" not in result,
            domain=request.domain,
            data=result,
            sources_used=result.get("sources", []),
            credits_used=result.get("credits_used", 0),
            signals_detected=len(signals),
        )

    except Exception as e:
        logger.error(f"Company enrichment failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/contacts", response_model=FindContactsResponse)
async def find_contacts(request: FindContactsRequest):
    """Find decision maker contacts at a company.

    Uses Apollo and Hunter to find contacts matching
    specified titles or departments.
    """
    try:
        aggregator = EnrichmentAggregator(
            user_id=request.user_id,
            apollo_key=request.apollo_key,
            hunter_key=request.hunter_key,
        )

        if not aggregator.available_sources:
            raise HTTPException(
                status_code=400,
                detail="No contact finding sources configured. Need Apollo or Hunter API key.",
            )

        result = await aggregator.find_contacts(
            domain=request.domain,
            titles=request.titles,
            limit=request.limit,
        )

        await aggregator.close()

        return FindContactsResponse(
            success=len(result.get("contacts", [])) > 0,
            domain=request.domain,
            contacts=result.get("contacts", []),
            total=result.get("total", 0),
            sources_used=result.get("sources", []),
        )

    except Exception as e:
        logger.error(f"Contact finding failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/verify-email", response_model=VerifyEmailResponse)
async def verify_email(request: VerifyEmailRequest):
    """Verify an email address.

    Checks deliverability, detects disposable/catch-all emails.
    """
    try:
        aggregator = EnrichmentAggregator(
            user_id=request.user_id,
            hunter_key=request.hunter_key,
        )

        if not aggregator.hunter:
            raise HTTPException(
                status_code=400,
                detail="Hunter API key required for email verification.",
            )

        result = await aggregator.verify_email(request.email)
        await aggregator.close()

        return VerifyEmailResponse(
            success="error" not in result,
            email=request.email,
            valid=result.get("valid"),
            confidence=result.get("confidence", 0),
            details=result.get("details"),
        )

    except Exception as e:
        logger.error(f"Email verification failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/find-email")
async def find_email(request: FindEmailRequest):
    """Find email for a specific person at a company.

    Uses Hunter's email finder to generate and verify
    the most likely email address.
    """
    try:
        aggregator = EnrichmentAggregator(
            user_id=request.user_id,
            hunter_key=request.hunter_key,
        )

        if not aggregator.hunter:
            raise HTTPException(
                status_code=400,
                detail="Hunter API key required for email finding.",
            )

        result = await aggregator.find_email(
            domain=request.domain,
            first_name=request.first_name,
            last_name=request.last_name,
        )
        await aggregator.close()

        return {
            "success": bool(result.get("email")),
            **result,
        }

    except Exception as e:
        logger.error(f"Email finding failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))
