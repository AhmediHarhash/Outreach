"""Health check routes."""

from fastapi import APIRouter

from ..config import get_settings

router = APIRouter(tags=["health"])


@router.get("/health")
async def health_check():
    """Health check endpoint for load balancers."""
    settings = get_settings()
    return {
        "status": "healthy",
        "service": "outreach-intelligence",
        "version": settings.app_version,
        "environment": settings.environment,
    }


@router.get("/")
async def root():
    """Root endpoint with API information."""
    return {
        "service": "Outreach Intelligence Worker",
        "version": "0.2.0",
        "description": "Lead enrichment, scoring, and signal detection",
        "endpoints": {
            "health": "/health",
            "enrich_company": "POST /api/v1/enrich/company",
            "enrich_lead": "POST /api/v1/enrich/lead",
            "find_contacts": "POST /api/v1/enrich/contacts",
            "verify_email": "POST /api/v1/enrich/verify-email",
            "score_lead": "POST /api/v1/score/lead",
            "get_signals": "GET /api/v1/signals/{domain}",
            "discover_leads": "POST /api/v1/discover",
        },
    }
