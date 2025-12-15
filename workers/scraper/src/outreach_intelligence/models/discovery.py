"""Discovered leads models."""

from datetime import datetime
from enum import Enum
from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class DiscoveryStatus(str, Enum):
    """Status of a discovered lead."""

    NEW = "new"  # Just discovered, awaiting review
    REVIEWED = "reviewed"  # User has seen it
    ACCEPTED = "accepted"  # Added to main leads
    REJECTED = "rejected"  # User rejected
    DUPLICATE = "duplicate"  # Already exists


class DiscoveredLead(BaseModel):
    """A lead discovered through automated search."""

    id: Optional[UUID] = None
    user_id: UUID
    icp_id: Optional[UUID] = None

    # Company info
    company_name: str
    company_domain: Optional[str] = None
    company_linkedin: Optional[str] = None

    # Contact info (decision maker)
    contact_name: Optional[str] = None
    contact_title: Optional[str] = None
    contact_email: Optional[str] = None
    contact_linkedin: Optional[str] = None

    # Enrichment data
    company_data: dict[str, Any] = Field(default_factory=dict)
    contact_data: dict[str, Any] = Field(default_factory=dict)

    # Scoring
    preliminary_score: int = Field(default=0, ge=0, le=100)
    score_breakdown: dict[str, Any] = Field(default_factory=dict)

    # Signals that triggered discovery
    discovery_signals: list[dict[str, Any]] = Field(default_factory=list)

    # Status
    status: DiscoveryStatus = DiscoveryStatus.NEW
    rejection_reason: Optional[str] = None

    # Source tracking
    source: Optional[str] = None  # Which service found this
    source_id: Optional[str] = None  # ID in source system

    # Timing
    discovered_at: datetime = Field(default_factory=datetime.utcnow)
    reviewed_at: Optional[datetime] = None
    accepted_at: Optional[datetime] = None

    # Link to actual lead if accepted
    converted_lead_id: Optional[UUID] = None

    @property
    def is_actionable(self) -> bool:
        """Check if lead can be acted upon."""
        return self.status in (DiscoveryStatus.NEW, DiscoveryStatus.REVIEWED)

    @property
    def has_contact(self) -> bool:
        """Check if we have contact information."""
        return bool(self.contact_email or self.contact_linkedin)
