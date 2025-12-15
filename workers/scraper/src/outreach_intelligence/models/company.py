"""Company data models."""

from datetime import datetime
from enum import Enum
from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, Field, HttpUrl


class CompanySize(str, Enum):
    """Company size categories."""

    MICRO = "1-10"
    SMALL = "11-50"
    MEDIUM = "51-200"
    LARGE = "201-500"
    ENTERPRISE = "501-1000"
    MEGA = "1000+"


class FundingStage(str, Enum):
    """Funding round stages."""

    PRE_SEED = "Pre-Seed"
    SEED = "Seed"
    SERIES_A = "Series A"
    SERIES_B = "Series B"
    SERIES_C = "Series C"
    SERIES_D_PLUS = "Series D+"
    IPO = "IPO"
    PRIVATE = "Private"
    BOOTSTRAPPED = "Bootstrapped"


class FundingRound(BaseModel):
    """A single funding round."""

    stage: FundingStage
    amount: Optional[int] = None  # USD
    currency: str = "USD"
    date: Optional[datetime] = None
    investors: list[str] = Field(default_factory=list)
    valuation: Optional[int] = None
    source_url: Optional[str] = None


class TechStackItem(BaseModel):
    """A technology in the company's stack."""

    name: str
    category: str  # frontend, backend, database, devops, analytics, etc.
    confidence: float = Field(default=1.0, ge=0.0, le=1.0)
    first_detected: Optional[datetime] = None
    source: Optional[str] = None


class Company(BaseModel):
    """Enriched company data."""

    # Identification
    domain: str
    name: str
    legal_name: Optional[str] = None

    # Basic info
    description: Optional[str] = None
    logo_url: Optional[str] = None
    website: Optional[str] = None

    # Industry & category
    industry: Optional[str] = None
    industry_group: Optional[str] = None
    sub_industry: Optional[str] = None
    sector: Optional[str] = None
    tags: list[str] = Field(default_factory=list)

    # Size metrics
    employee_count: Optional[int] = None
    employee_range: Optional[CompanySize] = None
    annual_revenue: Optional[int] = None  # USD
    revenue_range: Optional[str] = None

    # Funding
    funding_stage: Optional[FundingStage] = None
    total_funding: Optional[int] = None  # USD
    funding_rounds: list[FundingRound] = Field(default_factory=list)
    last_funding_date: Optional[datetime] = None

    # Technology
    tech_stack: list[TechStackItem] = Field(default_factory=list)
    tech_categories: list[str] = Field(default_factory=list)

    # Location
    country: Optional[str] = None
    country_code: Optional[str] = None
    state: Optional[str] = None
    city: Optional[str] = None
    address: Optional[str] = None
    timezone: Optional[str] = None

    # Social & online presence
    linkedin_url: Optional[str] = None
    twitter_url: Optional[str] = None
    facebook_url: Optional[str] = None
    crunchbase_url: Optional[str] = None

    # Contacts at company
    employee_count_on_linkedin: Optional[int] = None
    key_people: list[dict[str, Any]] = Field(default_factory=list)

    # Signals & activity
    is_hiring: bool = False
    open_positions: int = 0
    recent_news: list[dict[str, Any]] = Field(default_factory=list)

    # Meta
    enriched_at: datetime = Field(default_factory=datetime.utcnow)
    sources: list[str] = Field(default_factory=list)
    confidence_score: float = Field(default=1.0, ge=0.0, le=1.0)

    def get_employee_range(self) -> Optional[CompanySize]:
        """Get company size category from employee count."""
        if not self.employee_count:
            return self.employee_range

        count = self.employee_count
        if count <= 10:
            return CompanySize.MICRO
        elif count <= 50:
            return CompanySize.SMALL
        elif count <= 200:
            return CompanySize.MEDIUM
        elif count <= 500:
            return CompanySize.LARGE
        elif count <= 1000:
            return CompanySize.ENTERPRISE
        return CompanySize.MEGA
