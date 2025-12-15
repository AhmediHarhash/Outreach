"""Enrichment job models."""

from datetime import datetime
from enum import Enum
from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class EnrichmentJobType(str, Enum):
    """Types of enrichment jobs."""

    ENRICH_LEAD = "enrich_lead"  # Full lead enrichment
    ENRICH_COMPANY = "enrich_company"  # Company-only enrichment
    FIND_CONTACTS = "find_contacts"  # Find decision makers
    VERIFY_EMAIL = "verify_email"  # Email verification
    DETECT_SIGNALS = "detect_signals"  # Check for new signals
    SCORE_LEAD = "score_lead"  # Recalculate lead score
    DISCOVER_LEADS = "discover_leads"  # Find new leads matching ICP


class EnrichmentJobStatus(str, Enum):
    """Job status states."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class EnrichmentResult(BaseModel):
    """Result from an enrichment operation."""

    success: bool
    source: str
    data: dict[str, Any] = Field(default_factory=dict)
    error: Optional[str] = None
    credits_used: int = 0
    cached: bool = False
    fetched_at: datetime = Field(default_factory=datetime.utcnow)


class EnrichmentJob(BaseModel):
    """An enrichment job in the queue."""

    id: Optional[UUID] = None
    user_id: UUID

    job_type: EnrichmentJobType
    status: EnrichmentJobStatus = EnrichmentJobStatus.PENDING
    priority: int = Field(default=0, ge=-10, le=10)

    # Target
    lead_id: Optional[UUID] = None
    company_domain: Optional[str] = None
    icp_id: Optional[UUID] = None

    # Configuration
    config: dict[str, Any] = Field(default_factory=dict)
    """
    Example configs:
    - enrich_lead: {"sources": ["apollo", "clearbit"], "fields": ["company", "contact"]}
    - discover_leads: {"limit": 50, "sources": ["apollo"], "min_score": 60}
    - find_contacts: {"titles": ["CTO", "VP Engineering"], "limit": 5}
    """

    # Results
    result: Optional[dict[str, Any]] = None
    error_message: Optional[str] = None
    credits_used: int = 0

    # Timing
    scheduled_at: datetime = Field(default_factory=datetime.utcnow)
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None

    # Retry handling
    attempt_count: int = 0
    max_attempts: int = 3
    next_retry_at: Optional[datetime] = None

    created_at: datetime = Field(default_factory=datetime.utcnow)

    @property
    def is_retriable(self) -> bool:
        """Check if job can be retried."""
        return (
            self.status == EnrichmentJobStatus.FAILED
            and self.attempt_count < self.max_attempts
        )

    @property
    def duration_seconds(self) -> Optional[float]:
        """Get job duration in seconds."""
        if self.started_at and self.completed_at:
            return (self.completed_at - self.started_at).total_seconds()
        return None
