"""Lead and scoring models."""

from datetime import datetime
from enum import Enum
from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class ScoreTier(str, Enum):
    """Lead quality tier based on total score."""

    HOT = "hot"  # 80-100: Immediate outreach
    WARM = "warm"  # 60-79: Priority queue
    NURTURE = "nurture"  # 40-59: Drip campaign
    COLD = "cold"  # 0-39: Monitor for changes


class SignalType(str, Enum):
    """Types of signals we track."""

    FUNDING_ROUND = "funding_round"
    EXECUTIVE_HIRE = "executive_hire"
    JOB_POSTING = "job_posting"
    TECH_ADOPTION = "tech_adoption"
    NEWS_MENTION = "news_mention"
    GROWTH_INDICATOR = "growth_indicator"
    CONTRACT_ENDING = "contract_ending"
    WEBSITE_CHANGE = "website_change"


class SignalCategory(str, Enum):
    """Signal categories for scoring."""

    INTENT = "intent"  # Buying signals
    FIT = "fit"  # ICP match signals
    ENGAGEMENT = "engagement"  # Interaction signals


class ScoreComponent(BaseModel):
    """Individual score component with reasoning."""

    points: int = Field(ge=0, le=100)
    reason: str
    source: Optional[str] = None


class ScoreBreakdown(BaseModel):
    """Detailed breakdown of lead score."""

    intent: dict[str, ScoreComponent] = Field(default_factory=dict)
    fit: dict[str, ScoreComponent] = Field(default_factory=dict)
    accessibility: dict[str, ScoreComponent] = Field(default_factory=dict)


class LeadScore(BaseModel):
    """Lead score with all components."""

    id: Optional[UUID] = None
    lead_id: UUID
    icp_id: Optional[UUID] = None

    # Individual scores (0-100)
    intent_score: int = Field(default=0, ge=0, le=100)
    fit_score: int = Field(default=0, ge=0, le=100)
    accessibility_score: int = Field(default=0, ge=0, le=100)

    # Weighted total
    total_score: int = Field(default=0, ge=0, le=100)

    # Tier based on total score
    tier: ScoreTier = ScoreTier.COLD

    # Detailed breakdown
    score_breakdown: ScoreBreakdown = Field(default_factory=ScoreBreakdown)

    # Active signals
    active_signals: list[str] = Field(default_factory=list)

    # Tracking changes
    previous_score: Optional[int] = None
    score_change: Optional[int] = None

    calculated_at: datetime = Field(default_factory=datetime.utcnow)

    @staticmethod
    def calculate_tier(score: int) -> ScoreTier:
        """Calculate tier from total score."""
        if score >= 80:
            return ScoreTier.HOT
        elif score >= 60:
            return ScoreTier.WARM
        elif score >= 40:
            return ScoreTier.NURTURE
        return ScoreTier.COLD


class SignalEvent(BaseModel):
    """A signal event detected for a lead or company."""

    id: Optional[UUID] = None
    lead_id: Optional[UUID] = None
    company_domain: Optional[str] = None

    signal_type: SignalType
    signal_category: SignalCategory
    signal_data: dict[str, Any]

    # Impact on score
    score_impact: int = Field(default=0, ge=0)
    confidence: float = Field(default=1.0, ge=0.0, le=1.0)

    # Source
    source: Optional[str] = None
    source_url: Optional[str] = None

    # Timing
    signal_date: Optional[datetime] = None
    detected_at: datetime = Field(default_factory=datetime.utcnow)
    expires_at: Optional[datetime] = None

    # Processing status
    is_processed: bool = False
    processed_at: Optional[datetime] = None


class Lead(BaseModel):
    """Lead with enrichment data and current score."""

    id: UUID
    user_id: UUID

    # Basic info
    company_name: str
    company_domain: Optional[str] = None

    # Contact info
    contact_name: Optional[str] = None
    contact_title: Optional[str] = None
    contact_email: Optional[str] = None
    contact_linkedin: Optional[str] = None

    # Enriched company data
    company_data: dict[str, Any] = Field(default_factory=dict)

    # Current score
    current_score: Optional[LeadScore] = None

    # Active signals
    signals: list[SignalEvent] = Field(default_factory=list)

    created_at: datetime
    updated_at: datetime
