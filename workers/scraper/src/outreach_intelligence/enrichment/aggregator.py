"""Enrichment aggregator - combines data from multiple sources."""

import asyncio
import logging
from typing import Any, Optional
from uuid import UUID

from .apollo import ApolloClient
from .clearbit import ClearbitClient
from .hunter import HunterClient
from .crunchbase import CrunchbaseClient
from ..db.connection import fetch_one
from ..models.enrichment import EnrichmentResult
from ..models.company import Company
from ..models.contact import Contact

logger = logging.getLogger(__name__)


class EnrichmentAggregator:
    """Aggregates enrichment data from multiple sources.

    Prioritizes data based on source reliability and recency.
    Cross-references for accuracy.
    """

    def __init__(
        self,
        user_id: UUID,
        apollo_key: Optional[str] = None,
        clearbit_key: Optional[str] = None,
        hunter_key: Optional[str] = None,
        crunchbase_key: Optional[str] = None,
    ):
        self.user_id = user_id

        # Initialize clients for available services
        self.apollo = ApolloClient(apollo_key) if apollo_key else None
        self.clearbit = ClearbitClient(clearbit_key) if clearbit_key else None
        self.hunter = HunterClient(hunter_key) if hunter_key else None
        self.crunchbase = CrunchbaseClient(crunchbase_key) if crunchbase_key else None

        self._clients = [
            c for c in [self.apollo, self.clearbit, self.hunter, self.crunchbase]
            if c is not None
        ]

    @classmethod
    async def from_user_credentials(cls, user_id: UUID) -> "EnrichmentAggregator":
        """Create aggregator with user's stored API credentials."""
        # Fetch user's API keys from database
        credentials = {}

        services = ["apollo", "clearbit", "hunter", "crunchbase"]
        for service in services:
            row = await fetch_one(
                """
                SELECT api_key_encrypted, is_valid
                FROM enrichment_credentials
                WHERE user_id = $1 AND service = $2 AND is_valid = true
                """,
                user_id,
                service,
            )

            if row:
                # Note: In production, decrypt the API key here
                # credentials[service] = decrypt(row["api_key_encrypted"])
                # For now, we'll need the key passed in
                pass

        return cls(
            user_id=user_id,
            apollo_key=credentials.get("apollo"),
            clearbit_key=credentials.get("clearbit"),
            hunter_key=credentials.get("hunter"),
            crunchbase_key=credentials.get("crunchbase"),
        )

    @property
    def available_sources(self) -> list[str]:
        """List of available enrichment sources."""
        sources = []
        if self.apollo:
            sources.append("apollo")
        if self.clearbit:
            sources.append("clearbit")
        if self.hunter:
            sources.append("hunter")
        if self.crunchbase:
            sources.append("crunchbase")
        return sources

    async def enrich_company(
        self,
        domain: str,
        sources: Optional[list[str]] = None,
    ) -> dict[str, Any]:
        """Enrich company from multiple sources and merge data.

        Args:
            domain: Company domain
            sources: Specific sources to use (defaults to all available)

        Returns:
            Merged company data from all sources
        """
        if not self._clients:
            return {"error": "No enrichment sources configured", "domain": domain}

        # Determine which sources to query
        use_sources = sources or self.available_sources

        # Run enrichment tasks in parallel
        tasks = []

        if "apollo" in use_sources and self.apollo:
            tasks.append(("apollo", self.apollo.enrich_company(domain)))

        if "clearbit" in use_sources and self.clearbit:
            tasks.append(("clearbit", self.clearbit.enrich_company(domain)))

        if "hunter" in use_sources and self.hunter:
            tasks.append(("hunter", self.hunter.enrich_company(domain)))

        if "crunchbase" in use_sources and self.crunchbase:
            tasks.append(("crunchbase", self.crunchbase.enrich_company(domain)))

        # Execute all in parallel
        results: dict[str, EnrichmentResult] = {}
        for source, task in tasks:
            try:
                result = await task
                results[source] = result
            except Exception as e:
                logger.error(f"Enrichment from {source} failed: {e}")
                results[source] = EnrichmentResult(
                    success=False,
                    source=source,
                    error=str(e),
                )

        # Merge results
        return self._merge_company_data(domain, results)

    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        limit: int = 5,
    ) -> dict[str, Any]:
        """Find contacts at a company using available sources.

        Prioritizes Apollo for contact finding, falls back to Hunter.
        """
        contacts = []
        sources_used = []
        total_credits = 0

        # Apollo is best for contact finding
        if self.apollo:
            result = await self.apollo.find_contacts(
                domain=domain,
                titles=titles,
                limit=limit,
            )
            if result.success:
                contacts.extend(result.data.get("contacts", []))
                sources_used.append("apollo")
                total_credits += result.credits_used

        # Use Hunter as supplement/fallback
        if self.hunter and len(contacts) < limit:
            remaining = limit - len(contacts)
            result = await self.hunter.find_contacts(
                domain=domain,
                titles=titles,
                limit=remaining,
            )
            if result.success:
                # Deduplicate by email
                existing_emails = {c.get("email") for c in contacts if c.get("email")}
                new_contacts = [
                    c for c in result.data.get("contacts", [])
                    if c.get("email") not in existing_emails
                ]
                contacts.extend(new_contacts)
                sources_used.append("hunter")
                total_credits += result.credits_used

        # Enrich contacts with verification if we have Hunter
        if self.hunter and contacts:
            contacts = await self._verify_contacts(contacts)

        return {
            "domain": domain,
            "contacts": contacts[:limit],
            "total": len(contacts),
            "sources": sources_used,
            "credits_used": total_credits,
        }

    async def verify_email(self, email: str) -> dict[str, Any]:
        """Verify an email address.

        Hunter is the primary source for email verification.
        """
        if self.hunter:
            result = await self.hunter.verify_email(email)
            if result.success:
                return result.data

        # Fallback to Apollo if available
        if self.apollo:
            result = await self.apollo.verify_email(email)
            if result.success:
                return result.data

        return {
            "email": email,
            "valid": None,
            "error": "No email verification service available",
        }

    async def find_email(
        self,
        domain: str,
        first_name: str,
        last_name: str,
    ) -> dict[str, Any]:
        """Find email for a specific person.

        Uses Hunter's email finder as primary source.
        """
        if self.hunter:
            result = await self.hunter.find_email(
                domain=domain,
                first_name=first_name,
                last_name=last_name,
            )
            if result.success:
                return result.data

        return {
            "email": None,
            "error": "Email finder not available",
        }

    async def get_funding_signals(
        self,
        days: int = 30,
        industries: Optional[list[str]] = None,
        limit: int = 50,
    ) -> list[dict[str, Any]]:
        """Get recent funding signals for lead discovery.

        Uses Crunchbase as primary source.
        """
        if not self.crunchbase:
            return []

        result = await self.crunchbase.get_recent_funding(
            days=days,
            industries=industries,
            limit=limit,
        )

        if result.success:
            return result.data.get("companies", [])

        return []

    async def _verify_contacts(
        self,
        contacts: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Verify emails for a list of contacts."""
        if not self.hunter:
            return contacts

        verified = []
        for contact in contacts:
            email = contact.get("email")
            if email and not contact.get("email_verified"):
                result = await self.hunter.verify_email(email)
                if result.success:
                    contact["email_verified"] = result.data.get("valid", False)
                    contact["email_confidence"] = result.data.get("confidence", 0)
            verified.append(contact)

        return verified

    def _merge_company_data(
        self,
        domain: str,
        results: dict[str, EnrichmentResult],
    ) -> dict[str, Any]:
        """Merge company data from multiple sources.

        Priority order for different fields:
        - Basic info (name, description): Clearbit > Apollo > Crunchbase
        - Funding info: Crunchbase > Clearbit > Apollo
        - Tech stack: Clearbit > Apollo
        - Contact counts: Hunter > Apollo
        - Employee count: Clearbit > Apollo
        """
        merged: dict[str, Any] = {
            "domain": domain,
            "sources": [],
            "credits_used": 0,
        }

        # Track which sources succeeded
        for source, result in results.items():
            if result.success:
                merged["sources"].append(source)
                merged["credits_used"] += result.credits_used

        # No data found
        if not merged["sources"]:
            return {
                "domain": domain,
                "error": "No data found from any source",
                "sources": [],
                "credits_used": sum(r.credits_used for r in results.values()),
            }

        # Define field priorities
        # Format: field_name -> [source1, source2, ...] in priority order
        field_priority = {
            "name": ["clearbit", "apollo", "crunchbase"],
            "legal_name": ["clearbit"],
            "description": ["clearbit", "apollo", "crunchbase"],
            "logo_url": ["clearbit", "apollo"],
            "website": ["clearbit", "apollo"],
            "industry": ["clearbit", "apollo"],
            "industry_group": ["clearbit"],
            "sub_industry": ["clearbit"],
            "sector": ["clearbit"],
            "tags": ["clearbit"],
            "employee_count": ["clearbit", "apollo"],
            "employee_range": ["clearbit", "apollo", "crunchbase"],
            "annual_revenue": ["clearbit", "apollo"],
            "revenue_range": ["clearbit"],
            "funding_stage": ["crunchbase", "clearbit", "apollo"],
            "total_funding": ["crunchbase", "clearbit", "apollo"],
            "funding_rounds": ["crunchbase"],
            "last_funding_date": ["crunchbase", "apollo"],
            "num_funding_rounds": ["crunchbase"],
            "country": ["clearbit", "apollo"],
            "country_code": ["clearbit"],
            "state": ["clearbit", "apollo"],
            "city": ["clearbit", "apollo"],
            "address": ["clearbit"],
            "timezone": ["clearbit"],
            "linkedin_url": ["clearbit", "apollo", "crunchbase"],
            "twitter_url": ["clearbit", "apollo", "crunchbase"],
            "facebook_url": ["clearbit", "apollo"],
            "crunchbase_url": ["clearbit", "crunchbase"],
            "tech_stack": ["clearbit", "apollo"],
            "tech_categories": ["clearbit"],
            "founded_year": ["clearbit", "crunchbase"],
            "email_pattern": ["hunter"],
            "total_emails": ["hunter"],
            "is_hiring": ["apollo"],
            "open_positions": ["apollo"],
            "recent_news": ["crunchbase"],
        }

        # Merge fields by priority
        for field, sources in field_priority.items():
            for source in sources:
                if source not in results:
                    continue
                result = results[source]
                if not result.success:
                    continue

                value = result.data.get(field)
                if value is not None and value != "" and value != []:
                    merged[field] = value
                    break

        # Special handling for arrays - merge rather than replace
        array_fields = ["tech_stack", "tags", "categories", "funding_rounds", "recent_news"]
        for field in array_fields:
            all_values = []
            for source, result in results.items():
                if result.success:
                    values = result.data.get(field, [])
                    if isinstance(values, list):
                        all_values.extend(values)

            if all_values:
                # Deduplicate while preserving order
                seen = set()
                unique = []
                for item in all_values:
                    # Handle both string and dict items
                    key = str(item) if isinstance(item, dict) else item
                    if key not in seen:
                        seen.add(key)
                        unique.append(item)
                merged[field] = unique

        return merged

    async def close(self) -> None:
        """Close all client connections."""
        for client in self._clients:
            await client.close()
