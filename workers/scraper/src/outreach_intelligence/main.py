"""FastAPI application for Outreach Intelligence Worker.

A quality-first lead intelligence system providing:
- Multi-source company enrichment
- ICP-based lead scoring
- Signal detection and monitoring
- Automated lead discovery
"""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .config import get_settings
from .db import init_db, close_db
from .routes import (
    health_router,
    enrichment_router,
    scoring_router,
    discovery_router,
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan handler."""
    # Startup
    logger.info("Starting Outreach Intelligence Worker...")
    settings = get_settings()

    try:
        await init_db()
        logger.info("Database connection established")
    except Exception as e:
        logger.error(f"Failed to connect to database: {e}")
        # Continue anyway - some endpoints might not need DB

    logger.info(f"Environment: {settings.environment}")
    logger.info(f"Version: {settings.app_version}")

    yield

    # Shutdown
    logger.info("Shutting down...")
    await close_db()
    logger.info("Cleanup complete")


# Create FastAPI application
settings = get_settings()

app = FastAPI(
    title="Outreach Intelligence Worker",
    description="""
## Lead Intelligence System

A quality-first lead intelligence service that helps you find the **right leads at the right time**.

### Key Features

- **Multi-Source Enrichment**: Aggregate data from Apollo, Clearbit, Hunter, and Crunchbase
- **ICP-Based Scoring**: Score leads against your Ideal Customer Profile
- **Signal Detection**: Track funding, hiring, tech changes, and other buying signals
- **Smart Discovery**: Find new leads matching your ICP automatically

### Scoring Model

```
Lead Score = (Intent × 40%) + (Fit × 35%) + (Accessibility × 25%)
```

- **Intent (0-100)**: Buying signals like recent funding, hiring activity
- **Fit (0-100)**: How well the company matches your ICP
- **Accessibility (0-100)**: Ease of reaching decision makers

### Score Tiers

| Score | Tier | Action |
|-------|------|--------|
| 80-100 | Hot | Immediate personalized outreach |
| 60-79 | Warm | Priority queue, semi-personalized |
| 40-59 | Nurture | Add to drip campaign |
| 0-39 | Cold | Monitor for signal changes |
    """,
    version=settings.app_version,
    lifespan=lifespan,
)

# CORS configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
)

# Include routers
app.include_router(health_router)
app.include_router(enrichment_router)
app.include_router(scoring_router)
app.include_router(discovery_router)


# Error handlers
@app.exception_handler(Exception)
async def global_exception_handler(request, exc):
    """Global exception handler."""
    logger.error(f"Unhandled exception: {exc}", exc_info=True)
    return {
        "error": "Internal server error",
        "detail": str(exc) if settings.debug else "An unexpected error occurred",
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "outreach_intelligence.main:app",
        host="0.0.0.0",
        port=8000,
        reload=settings.debug,
    )
