"""FastAPI application for Hekax Scraper Worker"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI(
    title="Hekax Scraper Worker",
    description="LinkedIn scraping, CV processing, and email automation",
    version="0.1.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "hekax-scraper"}


@app.get("/")
async def root():
    return {
        "service": "hekax-scraper",
        "version": "0.1.0",
        "endpoints": {
            "scrape_linkedin": "/api/v1/scrape/linkedin",
            "process_cv": "/api/v1/cv/process",
            "send_email": "/api/v1/email/send",
        },
    }


# TODO: Add routes
# - /api/v1/scrape/linkedin - Scrape LinkedIn profile
# - /api/v1/scrape/company - Scrape company info
# - /api/v1/cv/parse - Parse CV/resume
# - /api/v1/cv/improve - AI-powered CV improvement
# - /api/v1/email/send - Send email
# - /api/v1/email/template - Manage email templates
