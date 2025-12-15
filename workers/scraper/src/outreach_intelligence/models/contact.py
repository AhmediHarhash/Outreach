"""Contact and person models."""

from datetime import datetime
from enum import Enum
from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field


class SeniorityLevel(str, Enum):
    """Job seniority levels."""

    C_LEVEL = "C-Level"
    VP = "VP"
    DIRECTOR = "Director"
    MANAGER = "Manager"
    SENIOR = "Senior"
    MID = "Mid"
    JUNIOR = "Junior"
    INTERN = "Intern"


class ContactInfo(BaseModel):
    """Contact information with verification status."""

    email: Optional[str] = None
    email_verified: bool = False
    email_confidence: float = Field(default=0.0, ge=0.0, le=1.0)

    phone: Optional[str] = None
    phone_verified: bool = False

    linkedin_url: Optional[str] = None
    linkedin_verified: bool = False

    twitter_url: Optional[str] = None


class Contact(BaseModel):
    """A person/contact at a company."""

    # Identification
    id: Optional[UUID] = None
    email: Optional[str] = None
    linkedin_url: Optional[str] = None

    # Basic info
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    full_name: Optional[str] = None

    # Professional info
    title: Optional[str] = None
    department: Optional[str] = None
    seniority: Optional[SeniorityLevel] = None

    # Company association
    company_name: Optional[str] = None
    company_domain: Optional[str] = None

    # Contact details
    contact_info: ContactInfo = Field(default_factory=ContactInfo)

    # Location
    city: Optional[str] = None
    state: Optional[str] = None
    country: Optional[str] = None

    # Profile data
    headline: Optional[str] = None
    summary: Optional[str] = None
    skills: list[str] = Field(default_factory=list)
    experience_years: Optional[int] = None

    # Photo
    photo_url: Optional[str] = None

    # Social activity
    linkedin_connections: Optional[int] = None
    twitter_followers: Optional[int] = None

    # Meta
    enriched_at: datetime = Field(default_factory=datetime.utcnow)
    sources: list[str] = Field(default_factory=list)

    @property
    def display_name(self) -> str:
        """Get display name, preferring full name."""
        if self.full_name:
            return self.full_name
        parts = [self.first_name, self.last_name]
        return " ".join(p for p in parts if p) or "Unknown"

    @staticmethod
    def infer_seniority(title: str) -> Optional[SeniorityLevel]:
        """Infer seniority level from job title."""
        if not title:
            return None

        title_lower = title.lower()

        # C-Level
        if any(t in title_lower for t in ["ceo", "cto", "cfo", "coo", "cmo", "chief"]):
            return SeniorityLevel.C_LEVEL

        # VP
        if any(t in title_lower for t in ["vp", "vice president"]):
            return SeniorityLevel.VP

        # Director
        if "director" in title_lower:
            return SeniorityLevel.DIRECTOR

        # Manager
        if "manager" in title_lower or "head of" in title_lower:
            return SeniorityLevel.MANAGER

        # Senior
        if any(t in title_lower for t in ["senior", "sr.", "lead", "principal", "staff"]):
            return SeniorityLevel.SENIOR

        # Junior
        if any(t in title_lower for t in ["junior", "jr.", "associate", "entry"]):
            return SeniorityLevel.JUNIOR

        # Intern
        if "intern" in title_lower:
            return SeniorityLevel.INTERN

        return SeniorityLevel.MID
