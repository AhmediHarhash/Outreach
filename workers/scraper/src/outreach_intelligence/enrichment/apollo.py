"""Apollo.io API client for lead enrichment."""

import logging
from typing import Any, Optional

from .base import BaseEnrichmentClient
from ..config import get_settings
from ..models.enrichment import EnrichmentResult
from ..models.company import Company, FundingStage
from ..models.contact import Contact, SeniorityLevel

logger = logging.getLogger(__name__)


class ApolloClient(BaseEnrichmentClient):
    """Apollo.io API client.

    Apollo provides:
    - Company enrichment
    - Contact finding with detailed filters
    - Email verification
    - People search
    """

    SOURCE_NAME = "apollo"
    ENTITY_TYPE = "company"

    def __init__(self, api_key: str):
        settings = get_settings()
        super().__init__(api_key, settings.apollo_rate_limit)

    @property
    def base_url(self) -> str:
        return "https://api.apollo.io/v1"

    def _get_http_client(self):
        client = super()._get_http_client()
        # Apollo uses api_key in request body/params, not headers
        return client

    async def enrich_company(self, domain: str) -> EnrichmentResult:
        """Enrich company by domain using Apollo.

        Args:
            domain: Company domain (e.g., "stripe.com")

        Returns:
            EnrichmentResult with company data
        """
        # Check cache first
        cached = await self._check_cache(domain)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            response = await http.post(
                "/organizations/enrich",
                json={
                    "api_key": self.api_key,
                    "domain": domain,
                },
            )

            if response.status_code == 200:
                data = response.json()
                org = data.get("organization", {})

                if org:
                    # Transform to our company model format
                    company_data = self._transform_company(org)

                    # Cache the result
                    await self._cache_result(domain, company_data)

                    return self._make_result(
                        success=True,
                        data=company_data,
                        credits_used=1,
                    )

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
            logger.error(f"Apollo company enrichment failed: {e}")
            return self._make_result(
                success=False,
                error=str(e),
            )

    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        seniority: Optional[list[str]] = None,
        departments: Optional[list[str]] = None,
        limit: int = 5,
    ) -> EnrichmentResult:
        """Find contacts at a company using Apollo People Search.

        Args:
            domain: Company domain
            titles: Filter by job titles
            seniority: Filter by seniority (e.g., "c_suite", "vp", "director")
            departments: Filter by department (e.g., "engineering", "sales")
            limit: Maximum contacts to return

        Returns:
            EnrichmentResult with list of contacts
        """
        cache_key = f"{domain}:{','.join(titles or [])}:{limit}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()

            # Build search parameters
            search_params: dict[str, Any] = {
                "api_key": self.api_key,
                "q_organization_domains": domain,
                "per_page": min(limit, 25),
                "page": 1,
            }

            if titles:
                search_params["person_titles"] = titles

            if seniority:
                # Apollo seniority values: owner, founder, c_suite, partner,
                # vp, head, director, manager, senior, entry, intern
                search_params["person_seniorities"] = seniority

            if departments:
                search_params["person_departments"] = departments

            response = await http.post(
                "/mixed_people/search",
                json=search_params,
            )

            if response.status_code == 200:
                data = response.json()
                people = data.get("people", [])

                contacts = [
                    self._transform_contact(person)
                    for person in people[:limit]
                ]

                result_data = {"contacts": contacts, "total": len(contacts)}

                # Cache with shorter TTL for contacts
                settings = get_settings()
                await self._cache_result(
                    cache_key,
                    result_data,
                    ttl_seconds=settings.company_cache_ttl,  # 7 days
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
            logger.error(f"Apollo contact search failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def verify_email(self, email: str) -> EnrichmentResult:
        """Verify email address using Apollo.

        Note: Apollo doesn't have a dedicated email verification endpoint,
        but we can use people/match to check if an email exists.
        """
        cache_key = f"email:{email}"
        cached = await self._check_cache(cache_key)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()
            response = await http.post(
                "/people/match",
                json={
                    "api_key": self.api_key,
                    "email": email,
                },
            )

            if response.status_code == 200:
                data = response.json()
                person = data.get("person")

                result_data = {
                    "email": email,
                    "valid": person is not None,
                    "confidence": 0.9 if person else 0.0,
                    "person": self._transform_contact(person) if person else None,
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
            logger.error(f"Apollo email verification failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def search_companies(
        self,
        industries: Optional[list[str]] = None,
        employee_count_min: Optional[int] = None,
        employee_count_max: Optional[int] = None,
        funding_stages: Optional[list[str]] = None,
        countries: Optional[list[str]] = None,
        technologies: Optional[list[str]] = None,
        limit: int = 25,
    ) -> EnrichmentResult:
        """Search for companies matching criteria.

        Used for lead discovery based on ICP.
        """
        try:
            http = self._get_http_client()

            search_params: dict[str, Any] = {
                "api_key": self.api_key,
                "per_page": min(limit, 100),
                "page": 1,
            }

            if industries:
                search_params["organization_industry_tag_ids"] = industries

            if employee_count_min or employee_count_max:
                search_params["organization_num_employees_ranges"] = [
                    f"{employee_count_min or 1},{employee_count_max or 1000000}"
                ]

            if countries:
                search_params["organization_locations"] = countries

            if technologies:
                search_params["currently_using_any_of_technology_uids"] = technologies

            response = await http.post(
                "/mixed_companies/search",
                json=search_params,
            )

            if response.status_code == 200:
                data = response.json()
                orgs = data.get("organizations", [])

                companies = [
                    self._transform_company(org)
                    for org in orgs[:limit]
                ]

                return self._make_result(
                    success=True,
                    data={
                        "companies": companies,
                        "total": data.get("pagination", {}).get("total_entries", 0),
                    },
                    credits_used=1,
                )

            return self._make_result(
                success=False,
                error=f"API error: {response.status_code}",
            )

        except Exception as e:
            logger.error(f"Apollo company search failed: {e}")
            return self._make_result(success=False, error=str(e))

    def _transform_company(self, apollo_org: dict) -> dict[str, Any]:
        """Transform Apollo organization to our company format."""
        # Map Apollo funding stage to our enum
        funding_map = {
            "seed": FundingStage.SEED,
            "series_a": FundingStage.SERIES_A,
            "series_b": FundingStage.SERIES_B,
            "series_c": FundingStage.SERIES_C,
            "series_d": FundingStage.SERIES_D_PLUS,
            "series_e": FundingStage.SERIES_D_PLUS,
            "private_equity": FundingStage.PRIVATE,
            "ipo": FundingStage.IPO,
        }

        return {
            "domain": apollo_org.get("primary_domain") or apollo_org.get("domain"),
            "name": apollo_org.get("name"),
            "description": apollo_org.get("short_description"),
            "logo_url": apollo_org.get("logo_url"),
            "website": apollo_org.get("website_url"),
            "industry": apollo_org.get("industry"),
            "employee_count": apollo_org.get("estimated_num_employees"),
            "annual_revenue": apollo_org.get("annual_revenue"),
            "funding_stage": funding_map.get(
                apollo_org.get("latest_funding_stage", "").lower()
            ),
            "total_funding": apollo_org.get("total_funding"),
            "last_funding_date": apollo_org.get("latest_funding_round_date"),
            "country": apollo_org.get("country"),
            "city": apollo_org.get("city"),
            "state": apollo_org.get("state"),
            "linkedin_url": apollo_org.get("linkedin_url"),
            "twitter_url": apollo_org.get("twitter_url"),
            "facebook_url": apollo_org.get("facebook_url"),
            "tech_stack": apollo_org.get("technologies", []),
            "is_hiring": bool(apollo_org.get("current_job_openings")),
            "open_positions": len(apollo_org.get("current_job_openings", [])),
            "sources": ["apollo"],
        }

    def _transform_contact(self, apollo_person: dict) -> dict[str, Any]:
        """Transform Apollo person to our contact format."""
        if not apollo_person:
            return {}

        # Map Apollo seniority to our enum
        seniority_map = {
            "c_suite": SeniorityLevel.C_LEVEL,
            "owner": SeniorityLevel.C_LEVEL,
            "founder": SeniorityLevel.C_LEVEL,
            "partner": SeniorityLevel.C_LEVEL,
            "vp": SeniorityLevel.VP,
            "head": SeniorityLevel.DIRECTOR,
            "director": SeniorityLevel.DIRECTOR,
            "manager": SeniorityLevel.MANAGER,
            "senior": SeniorityLevel.SENIOR,
            "entry": SeniorityLevel.JUNIOR,
            "intern": SeniorityLevel.INTERN,
        }

        return {
            "first_name": apollo_person.get("first_name"),
            "last_name": apollo_person.get("last_name"),
            "full_name": apollo_person.get("name"),
            "email": apollo_person.get("email"),
            "title": apollo_person.get("title"),
            "seniority": seniority_map.get(apollo_person.get("seniority")),
            "department": apollo_person.get("departments", [None])[0],
            "company_name": apollo_person.get("organization", {}).get("name"),
            "company_domain": apollo_person.get("organization", {}).get("primary_domain"),
            "linkedin_url": apollo_person.get("linkedin_url"),
            "city": apollo_person.get("city"),
            "state": apollo_person.get("state"),
            "country": apollo_person.get("country"),
            "photo_url": apollo_person.get("photo_url"),
            "headline": apollo_person.get("headline"),
            "sources": ["apollo"],
        }
