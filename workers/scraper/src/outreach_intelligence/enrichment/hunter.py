"""Hunter.io API client for email finding and verification."""

import logging
from typing import Any, Optional

from .base import BaseEnrichmentClient
from ..config import get_settings
from ..models.enrichment import EnrichmentResult

logger = logging.getLogger(__name__)


class HunterClient(BaseEnrichmentClient):
    """Hunter.io API client.

    Hunter provides:
    - Email finding by domain
    - Email verification
    - Email pattern detection
    - Contact search
    """

    SOURCE_NAME = "hunter"
    ENTITY_TYPE = "email"

    def __init__(self, api_key: str):
        settings = get_settings()
        super().__init__(api_key, settings.hunter_rate_limit)

    @property
    def base_url(self) -> str:
        return "https://api.hunter.io/v2"

    async def enrich_company(self, domain: str) -> EnrichmentResult:
        """Get company info and email pattern from Hunter.

        Returns email pattern and available contacts count.
        """
        cached = await self._check_cache(domain)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            response = await http.get(
                "/domain-search",
                params={
                    "api_key": self.api_key,
                    "domain": domain,
                    "limit": 0,  # Just get meta info
                },
            )

            if response.status_code == 200:
                data = response.json().get("data", {})

                company_data = {
                    "domain": domain,
                    "name": data.get("organization"),
                    "email_pattern": data.get("pattern"),
                    "total_emails": data.get("emails_count", 0),
                    "department_breakdown": data.get("department_counts", {}),
                    "sources": ["hunter"],
                }

                await self._cache_result(domain, company_data)

                return self._make_result(
                    success=True,
                    data=company_data,
                    credits_used=1,
                )

            elif response.status_code == 401:
                return self._make_result(
                    success=False,
                    error="Invalid API key",
                )
            elif response.status_code == 429:
                return self._make_result(
                    success=False,
                    error="Rate limit exceeded",
                )
            else:
                return self._make_result(
                    success=False,
                    error=f"API error: {response.status_code}",
                )

        except Exception as e:
            logger.error(f"Hunter domain search failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        department: Optional[str] = None,
        limit: int = 10,
    ) -> EnrichmentResult:
        """Find email contacts at a company using Hunter.

        Args:
            domain: Company domain
            titles: Filter by job titles (not directly supported, filtered locally)
            department: Filter by department (executive, it, finance, etc.)
            limit: Maximum contacts to return

        Returns:
            EnrichmentResult with list of contacts
        """
        cache_key = f"{domain}:{department or 'all'}:{limit}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()

            params: dict[str, Any] = {
                "api_key": self.api_key,
                "domain": domain,
                "limit": min(limit, 100),
            }

            if department:
                params["department"] = department

            response = await http.get("/domain-search", params=params)

            if response.status_code == 200:
                data = response.json().get("data", {})
                emails = data.get("emails", [])

                contacts = []
                for email_data in emails:
                    contact = self._transform_contact(email_data)

                    # Local title filtering if specified
                    if titles and contact.get("title"):
                        title_lower = contact["title"].lower()
                        if not any(t.lower() in title_lower for t in titles):
                            continue

                    contacts.append(contact)

                    if len(contacts) >= limit:
                        break

                result_data = {
                    "contacts": contacts,
                    "total": len(contacts),
                    "email_pattern": data.get("pattern"),
                }

                settings = get_settings()
                await self._cache_result(
                    cache_key,
                    result_data,
                    ttl_seconds=settings.company_cache_ttl,
                )

                return self._make_result(
                    success=True,
                    data=result_data,
                    credits_used=1,
                )

            return self._make_result(
                success=False,
                error=f"API error: {response.status_code}",
            )

        except Exception as e:
            logger.error(f"Hunter contact search failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def verify_email(self, email: str) -> EnrichmentResult:
        """Verify an email address using Hunter.

        Args:
            email: Email address to verify

        Returns:
            EnrichmentResult with verification status
        """
        cache_key = f"verify:{email}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            response = await http.get(
                "/email-verifier",
                params={
                    "api_key": self.api_key,
                    "email": email,
                },
            )

            if response.status_code == 200:
                data = response.json().get("data", {})

                result_data = {
                    "email": email,
                    "valid": data.get("result") == "deliverable",
                    "status": data.get("result"),  # deliverable, undeliverable, risky, unknown
                    "score": data.get("score", 0),  # 0-100
                    "confidence": data.get("score", 0) / 100,
                    "details": {
                        "mx_records": data.get("mx_records"),
                        "smtp_server": data.get("smtp_server"),
                        "smtp_check": data.get("smtp_check"),
                        "accept_all": data.get("accept_all"),
                        "disposable": data.get("disposable"),
                        "webmail": data.get("webmail"),
                        "gibberish": data.get("gibberish"),
                    },
                }

                await self._cache_result(cache_key, result_data)

                return self._make_result(
                    success=True,
                    data=result_data,
                    credits_used=1,
                )

            return self._make_result(
                success=False,
                error=f"API error: {response.status_code}",
            )

        except Exception as e:
            logger.error(f"Hunter email verification failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def find_email(
        self,
        domain: str,
        first_name: str,
        last_name: str,
    ) -> EnrichmentResult:
        """Find email for a specific person at a company.

        Args:
            domain: Company domain
            first_name: Person's first name
            last_name: Person's last name

        Returns:
            EnrichmentResult with email and confidence
        """
        cache_key = f"find:{domain}:{first_name}:{last_name}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            response = await http.get(
                "/email-finder",
                params={
                    "api_key": self.api_key,
                    "domain": domain,
                    "first_name": first_name,
                    "last_name": last_name,
                },
            )

            if response.status_code == 200:
                data = response.json().get("data", {})

                result_data = {
                    "email": data.get("email"),
                    "confidence": data.get("score", 0) / 100,
                    "first_name": first_name,
                    "last_name": last_name,
                    "domain": domain,
                    "position": data.get("position"),
                    "linkedin_url": data.get("linkedin"),
                    "verified": data.get("verification", {}).get("status") == "valid",
                }

                await self._cache_result(cache_key, result_data)

                return self._make_result(
                    success=True,
                    data=result_data,
                    credits_used=1,
                )

            elif response.status_code == 404:
                return self._make_result(
                    success=False,
                    error="Email not found",
                )
            else:
                return self._make_result(
                    success=False,
                    error=f"API error: {response.status_code}",
                )

        except Exception as e:
            logger.error(f"Hunter email finder failed: {e}")
            return self._make_result(success=False, error=str(e))

    def _transform_contact(self, hunter_email: dict) -> dict[str, Any]:
        """Transform Hunter email result to our contact format."""
        return {
            "email": hunter_email.get("value"),
            "email_confidence": hunter_email.get("confidence", 0) / 100,
            "first_name": hunter_email.get("first_name"),
            "last_name": hunter_email.get("last_name"),
            "full_name": f"{hunter_email.get('first_name', '')} {hunter_email.get('last_name', '')}".strip(),
            "title": hunter_email.get("position"),
            "department": hunter_email.get("department"),
            "seniority": hunter_email.get("seniority"),
            "linkedin_url": hunter_email.get("linkedin"),
            "twitter_url": hunter_email.get("twitter"),
            "phone": hunter_email.get("phone_number"),
            "sources": ["hunter"],
        }
