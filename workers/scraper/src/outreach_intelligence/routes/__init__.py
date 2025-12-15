"""FastAPI routes for the intelligence worker."""

from .health import router as health_router
from .enrichment import router as enrichment_router
from .scoring import router as scoring_router
from .discovery import router as discovery_router

__all__ = [
    "health_router",
    "enrichment_router",
    "scoring_router",
    "discovery_router",
]
