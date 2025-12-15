"""Clearbit API client for company enrichment."""

import base64
import logging
from typing import Any, Optional

from .base import BaseEnrichmentClient, RateLimitedClient
from ..config import get_settings
from ..models.enrichment import EnrichmentResult
from ..models.company import FundingStage

logger = logging.getLogger(__name__)


class ClearbitClient(BaseEnrichmentClient):
    """Clearbit API client.

    Clearbit provides:
    - Company enrichment with detailed firmographic data
    - Person enrichment by email
    - Company autocomplete
    - Tech stack detection
    """

    SOURCE_NAME = "clearbit"
    ENTITY_TYPE = "company"

    def __init__(self, api_key: str):
        settings = get_settings()
        super().__init__(api_key, settings.clearbit_rate_limit)

    @property
    def base_url(self) -> str:
        return "https://company.clearbit.com/v2"

    def _get_http_client(self) -> RateLimitedClient:
        """Override to set Basic auth header."""
        if self._http is None:
            self._http = RateLimitedClient(
                base_url=self.base_url,
                api_key=self.api_key,
                rate_limit=self.rate_limit,
            )
            # Clearbit uses Basic auth with API key
            auth_str = base64.b64encode(f"{self.api_key}:".encode()).decode()
            self._http._client = None  # Reset to pick up new headers

        return self._http

    async def enrich_company(self, domain: str) -> EnrichmentResult:
        """Enrich company by domain using Clearbit.

        Args:
            domain: Company domain (e.g., "stripe.com")

        Returns:
            EnrichmentResult with detailed company data
        """
        # Check cache first
        cached = await self._check_cache(domain)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()

            # Clearbit uses Basic auth
            auth_header = base64.b64encode(f"{self.api_key}:".encode()).decode()

            response = await http.request(
                "GET",
                f"/companies/find",
                params={"domain": domain},
                headers={
                    "Authorization": f"Basic {auth_header}",
                },
            )

            if response.status_code == 200:
                data = response.json()
                company_data = self._transform_company(data)

                # Cache the result
                await self._cache_result(domain, company_data)

                return self._make_result(
                    success=True,
                    data=company_data,
                    credits_used=1,
                )

            elif response.status_code == 404:
                return self._make_result(
                    success=False,
                    error="Company not found",
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
            logger.error(f"Clearbit company enrichment failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        limit: int = 5,
    ) -> EnrichmentResult:
        """Clearbit doesn't have a direct contact finder.

        Use the people enrichment API if you have an email.
        """
        return self._make_result(
            success=False,
            error="Clearbit does not support contact finding. Use Apollo or Hunter.",
        )

    async def verify_email(self, email: str) -> EnrichmentResult:
        """Clearbit doesn't have email verification.

        Use Hunter for email verification.
        """
        return self._make_result(
            success=False,
            error="Clearbit does not support email verification. Use Hunter.",
        )

    async def enrich_person(self, email: str) -> EnrichmentResult:
        """Enrich person by email using Clearbit Person API.

        Args:
            email: Person's email address

        Returns:
            EnrichmentResult with person data
        """
        cache_key = f"person:{email}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            auth_header = base64.b64encode(f"{self.api_key}:".encode()).decode()

            response = await http.request(
                "GET",
                "/people/find",
                params={"email": email},
                headers={
                    "Authorization": f"Basic {auth_header}",
                },
            )

            if response.status_code == 200:
                data = response.json()
                person_data = self._transform_person(data)

                await self._cache_result(cache_key, person_data)

                return self._make_result(
                    success=True,
                    data=person_data,
                    credits_used=1,
                )

            elif response.status_code == 404:
                return self._make_result(
                    success=False,
                    error="Person not found",
                )
            else:
                return self._make_result(
                    success=False,
                    error=f"API error: {response.status_code}",
                )

        except Exception as e:
            logger.error(f"Clearbit person enrichment failed: {e}")
            return self._make_result(success=False, error=str(e))

    def _transform_company(self, clearbit_data: dict) -> dict[str, Any]:
        """Transform Clearbit company data to our format."""
        # Map Clearbit employee range
        employee_range = clearbit_data.get("metrics", {}).get("employeesRange")

        # Map funding info
        funding = clearbit_data.get("metrics", {}).get("raised")
        funding_stage = None
        if funding:
            if funding < 1_000_000:
                funding_stage = FundingStage.SEED
            elif funding < 10_000_000:
                funding_stage = FundingStage.SERIES_A
            elif funding < 50_000_000:
                funding_stage = FundingStage.SERIES_B
            elif funding < 100_000_000:
                funding_stage = FundingStage.SERIES_C
            else:
                funding_stage = FundingStage.SERIES_D_PLUS

        geo = clearbit_data.get("geo", {})
        metrics = clearbit_data.get("metrics", {})

        return {
            "domain": clearbit_data.get("domain"),
            "name": clearbit_data.get("name"),
            "legal_name": clearbit_data.get("legalName"),
            "description": clearbit_data.get("description"),
            "logo_url": clearbit_data.get("logo"),
            "website": clearbit_data.get("url"),
            "industry": clearbit_data.get("category", {}).get("industry"),
            "industry_group": clearbit_data.get("category", {}).get("industryGroup"),
            "sub_industry": clearbit_data.get("category", {}).get("subIndustry"),
            "sector": clearbit_data.get("category", {}).get("sector"),
            "tags": clearbit_data.get("tags", []),
            "employee_count": metrics.get("employees"),
            "employee_range": employee_range,
            "annual_revenue": metrics.get("annualRevenue"),
            "revenue_range": metrics.get("estimatedAnnualRevenue"),
            "funding_stage": funding_stage,
            "total_funding": funding,
            "country": geo.get("country"),
            "country_code": geo.get("countryCode"),
            "state": geo.get("state"),
            "city": geo.get("city"),
            "address": geo.get("streetAddress"),
            "timezone": clearbit_data.get("timeZone"),
            "linkedin_url": clearbit_data.get("linkedin", {}).get("handle"),
            "twitter_url": clearbit_data.get("twitter", {}).get("handle"),
            "facebook_url": clearbit_data.get("facebook", {}).get("handle"),
            "crunchbase_url": clearbit_data.get("crunchbase", {}).get("handle"),
            "tech_stack": clearbit_data.get("tech", []),
            "tech_categories": list(
                set(t.get("category") for t in clearbit_data.get("tech", []) if t.get("category"))
            ),
            "founded_year": clearbit_data.get("foundedYear"),
            "sources": ["clearbit"],
        }

    def _transform_person(self, clearbit_data: dict) -> dict[str, Any]:
        """Transform Clearbit person data to our format."""
        employment = clearbit_data.get("employment", {})
        geo = clearbit_data.get("geo", {})

        return {
            "email": clearbit_data.get("email"),
            "first_name": clearbit_data.get("name", {}).get("givenName"),
            "last_name": clearbit_data.get("name", {}).get("familyName"),
            "full_name": clearbit_data.get("name", {}).get("fullName"),
            "title": employment.get("title"),
            "company_name": employment.get("name"),
            "company_domain": employment.get("domain"),
            "seniority": employment.get("seniority"),
            "city": geo.get("city"),
            "state": geo.get("state"),
            "country": geo.get("country"),
            "linkedin_url": clearbit_data.get("linkedin", {}).get("handle"),
            "twitter_url": clearbit_data.get("twitter", {}).get("handle"),
            "photo_url": clearbit_data.get("avatar"),
            "sources": ["clearbit"],
        }
