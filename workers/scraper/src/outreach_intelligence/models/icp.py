"""Ideal Customer Profile (ICP) models."""

from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, Field

from .company import FundingStage
from .contact import SeniorityLevel


class TechRequirements(BaseModel):
    """Technology stack requirements."""

    must_have: list[str] = Field(default_factory=list)
    nice_to_have: list[str] = Field(default_factory=list)
    avoid: list[str] = Field(default_factory=list)


class ICPFilters(BaseModel):
    """Filters derived from ICP for searching."""

    # Industries
    industries: list[str] = Field(default_factory=list)
    excluded_industries: list[str] = Field(default_factory=list)

    # Company size
    company_size_min: Optional[int] = None
    company_size_max: Optional[int] = None
    revenue_min: Optional[int] = None
    revenue_max: Optional[int] = None

    # Funding
    funding_stages: list[FundingStage] = Field(default_factory=list)
    min_funding_amount: Optional[int] = None
    recently_funded_days: Optional[int] = None

    # Technology
    tech_requirements: TechRequirements = Field(default_factory=TechRequirements)

    # Geography
    countries: list[str] = Field(default_factory=list)
    excluded_countries: list[str] = Field(default_factory=list)
    regions: list[str] = Field(default_factory=list)

    # Decision makers
    target_titles: list[str] = Field(default_factory=list)
    target_departments: list[str] = Field(default_factory=list)
    seniority_levels: list[SeniorityLevel] = Field(default_factory=list)

    # Signals
    require_recent_funding: bool = False
    require_hiring_signals: bool = False
    require_tech_change: bool = False


class ScoringWeights(BaseModel):
    """Weights for score calculation."""

    intent: int = Field(default=40, ge=0, le=100)
    fit: int = Field(default=35, ge=0, le=100)
    accessibility: int = Field(default=25, ge=0, le=100)

    def validate_total(self) -> bool:
        """Ensure weights sum to 100."""
        return self.intent + self.fit + self.accessibility == 100

    def normalize(self) -> "ScoringWeights":
        """Normalize weights to sum to 100."""
        total = self.intent + self.fit + self.accessibility
        if total == 0:
            return ScoringWeights()

        factor = 100 / total
        return ScoringWeights(
            intent=int(self.intent * factor),
            fit=int(self.fit * factor),
            accessibility=100 - int(self.intent * factor) - int(self.fit * factor),
        )


class ICPProfile(BaseModel):
    """Ideal Customer Profile configuration."""

    id: Optional[UUID] = None
    user_id: UUID
    name: str
    description: Optional[str] = None
    is_default: bool = False

    # All filter criteria
    filters: ICPFilters = Field(default_factory=ICPFilters)

    # Scoring weights
    weights: ScoringWeights = Field(default_factory=ScoringWeights)

    # Timestamps
    created_at: datetime = Field(default_factory=datetime.utcnow)
    updated_at: datetime = Field(default_factory=datetime.utcnow)

    @classmethod
    def from_db_row(cls, row) -> "ICPProfile":
        """Create ICP profile from database row."""
        filters = ICPFilters(
            industries=row.get("industries", []),
            excluded_industries=row.get("excluded_industries", []),
            company_size_min=row.get("company_size_min"),
            company_size_max=row.get("company_size_max"),
            revenue_min=row.get("revenue_min"),
            revenue_max=row.get("revenue_max"),
            funding_stages=[
                FundingStage(s) for s in row.get("funding_stages", [])
            ],
            min_funding_amount=row.get("min_funding_amount"),
            recently_funded_days=row.get("recently_funded_days"),
            tech_requirements=TechRequirements(
                must_have=row.get("tech_must_have", []),
                nice_to_have=row.get("tech_nice_to_have", []),
                avoid=row.get("tech_avoid", []),
            ),
            countries=row.get("countries", []),
            excluded_countries=row.get("excluded_countries", []),
            regions=row.get("regions", []),
            target_titles=row.get("target_titles", []),
            target_departments=row.get("target_departments", []),
            seniority_levels=[
                SeniorityLevel(s) for s in row.get("seniority_levels", [])
            ],
            require_recent_funding=row.get("require_recent_funding", False),
            require_hiring_signals=row.get("require_hiring_signals", False),
            require_tech_change=row.get("require_tech_change", False),
        )

        weights = ScoringWeights(
            intent=row.get("weight_intent", 40),
            fit=row.get("weight_fit", 35),
            accessibility=row.get("weight_accessibility", 25),
        )

        return cls(
            id=row.get("id"),
            user_id=row.get("user_id"),
            name=row.get("name", ""),
            description=row.get("description"),
            is_default=row.get("is_default", False),
            filters=filters,
            weights=weights,
            created_at=row.get("created_at"),
            updated_at=row.get("updated_at"),
        )
