"""Crunchbase API client for funding and news intelligence."""

import logging
from datetime import datetime, timedelta
from typing import Any, Optional

from .base import BaseEnrichmentClient
from ..config import get_settings
from ..models.enrichment import EnrichmentResult
from ..models.company import FundingStage, FundingRound

logger = logging.getLogger(__name__)


class CrunchbaseClient(BaseEnrichmentClient):
    """Crunchbase API client.

    Crunchbase provides:
    - Funding round information
    - Investor data
    - News and press coverage
    - Executive changes
    - Acquisition history

    Note: Crunchbase Basic API is limited. Pro API needed for full access.
    """

    SOURCE_NAME = "crunchbase"
    ENTITY_TYPE = "company"

    def __init__(self, api_key: str):
        settings = get_settings()
        super().__init__(api_key, settings.crunchbase_rate_limit)

    @property
    def base_url(self) -> str:
        return "https://api.crunchbase.com/api/v4"

    async def enrich_company(self, domain: str) -> EnrichmentResult:
        """Enrich company with funding data from Crunchbase.

        Args:
            domain: Company domain

        Returns:
            EnrichmentResult with funding and news data
        """
        cached = await self._check_cache(domain)
        if cached:
            return self._make_result(success=True, data=cached, cached=True)

        try:
            http = self._get_http_client()

            # First, search for the organization by domain
            response = await http.get(
                "/autocompletes",
                params={
                    "user_key": self.api_key,
                    "query": domain.split(".")[0],  # Use company name from domain
                    "collection_ids": "organizations",
                    "limit": 5,
                },
            )

            if response.status_code != 200:
                return self._make_result(
                    success=False,
                    error=f"API error: {response.status_code}",
                )

            results = response.json().get("entities", [])

            # Find matching organization
            org_id = None
            for result in results:
                identifier = result.get("identifier", {})
                props = result.get("properties", {})
                # Match by domain or name
                if (
                    props.get("website_url", "").endswith(domain)
                    or domain.split(".")[0].lower() in identifier.get("value", "").lower()
                ):
                    org_id = identifier.get("permalink")
                    break

            if not org_id:
                return self._make_result(
                    success=False,
                    error="Company not found in Crunchbase",
                )

            # Get full organization details
            org_response = await http.get(
                f"/entities/organizations/{org_id}",
                params={
                    "user_key": self.api_key,
                    "field_ids": "short_description,founded_on,num_employees_enum,"
                    "funding_total,last_funding_type,last_funding_at,"
                    "num_funding_rounds,ipo_status,website_url,"
                    "linkedin,twitter,categories",
                },
            )

            if org_response.status_code != 200:
                return self._make_result(
                    success=False,
                    error=f"Failed to fetch organization: {org_response.status_code}",
                )

            org_data = org_response.json().get("properties", {})

            # Get funding rounds
            funding_rounds = await self._get_funding_rounds(org_id)

            # Get recent news
            news = await self._get_news(org_id)

            company_data = {
                "domain": domain,
                "crunchbase_id": org_id,
                "description": org_data.get("short_description"),
                "founded_year": (
                    org_data.get("founded_on", "")[:4]
                    if org_data.get("founded_on")
                    else None
                ),
                "employee_range": org_data.get("num_employees_enum"),
                "total_funding": org_data.get("funding_total", {}).get("value"),
                "funding_currency": org_data.get("funding_total", {}).get("currency"),
                "funding_stage": self._map_funding_stage(org_data.get("last_funding_type")),
                "last_funding_date": org_data.get("last_funding_at"),
                "num_funding_rounds": org_data.get("num_funding_rounds"),
                "ipo_status": org_data.get("ipo_status"),
                "funding_rounds": funding_rounds,
                "recent_news": news,
                "linkedin_url": org_data.get("linkedin", {}).get("value"),
                "twitter_url": org_data.get("twitter", {}).get("value"),
                "categories": [
                    c.get("value") for c in org_data.get("categories", [])
                ],
                "sources": ["crunchbase"],
            }

            await self._cache_result(domain, company_data)

            return self._make_result(
                success=True,
                data=company_data,
                credits_used=1,
            )

        except Exception as e:
            logger.error(f"Crunchbase enrichment failed: {e}")
            return self._make_result(success=False, error=str(e))

    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        limit: int = 5,
    ) -> EnrichmentResult:
        """Crunchbase has limited people data in basic API."""
        return self._make_result(
            success=False,
            error="Crunchbase Basic API doesn't support contact finding. Use Apollo.",
        )

    async def verify_email(self, email: str) -> EnrichmentResult:
        """Crunchbase doesn't support email verification."""
        return self._make_result(
            success=False,
            error="Crunchbase doesn't support email verification. Use Hunter.",
        )

    async def _get_funding_rounds(self, org_id: str) -> list[dict[str, Any]]:
        """Get funding rounds for an organization."""
        try:
            http = self._get_http_client()
            response = await http.get(
                f"/entities/organizations/{org_id}/funding_rounds",
                params={
                    "user_key": self.api_key,
                    "limit": 10,
                },
            )

            if response.status_code != 200:
                return []

            rounds = response.json().get("entities", [])

            return [
                {
                    "stage": self._map_funding_stage(r.get("properties", {}).get("funding_type")),
                    "amount": r.get("properties", {}).get("money_raised", {}).get("value"),
                    "currency": r.get("properties", {}).get("money_raised", {}).get("currency", "USD"),
                    "date": r.get("properties", {}).get("announced_on"),
                    "investors": [],  # Would need separate API call
                }
                for r in rounds
            ]

        except Exception as e:
            logger.error(f"Failed to get funding rounds: {e}")
            return []

    async def _get_news(self, org_id: str, limit: int = 5) -> list[dict[str, Any]]:
        """Get recent news for an organization."""
        try:
            http = self._get_http_client()
            response = await http.get(
                f"/entities/organizations/{org_id}/press_references",
                params={
                    "user_key": self.api_key,
                    "limit": limit,
                },
            )

            if response.status_code != 200:
                return []

            news = response.json().get("entities", [])

            return [
                {
                    "title": n.get("properties", {}).get("title"),
                    "url": n.get("properties", {}).get("url"),
                    "posted_on": n.get("properties", {}).get("posted_on"),
                    "publisher": n.get("properties", {}).get("publisher"),
                }
                for n in news
            ]

        except Exception as e:
            logger.error(f"Failed to get news: {e}")
            return []

    async def get_recent_funding(
        self,
        days: int = 30,
        funding_types: Optional[list[str]] = None,
        industries: Optional[list[str]] = None,
        limit: int = 50,
    ) -> EnrichmentResult:
        """Search for companies with recent funding.

        This is key for signal-based discovery.

        Args:
            days: Look back period in days
            funding_types: Filter by funding type (seed, series_a, etc.)
            industries: Filter by industry category
            limit: Maximum results

        Returns:
            EnrichmentResult with recently funded companies
        """
        try:
            http = self._get_http_client()

            since_date = (datetime.utcnow() - timedelta(days=days)).strftime("%Y-%m-%d")

            # Build query
            query_params: dict[str, Any] = {
                "user_key": self.api_key,
                "limit": min(limit, 100),
            }

            # Note: Full search capabilities require Crunchbase Pro API
            # This is a simplified version using autocomplete

            response = await http.get(
                "/searches/funding_rounds",
                params={
                    "user_key": self.api_key,
                    "announced_on_gte": since_date,
                    "limit": limit,
                },
            )

            if response.status_code != 200:
                return self._make_result(
                    success=False,
                    error=f"API error: {response.status_code}",
                )

            rounds = response.json().get("entities", [])

            companies = []
            for r in rounds:
                props = r.get("properties", {})
                org = props.get("funded_organization", {})

                if funding_types:
                    if props.get("funding_type") not in funding_types:
                        continue

                companies.append({
                    "company_name": org.get("value"),
                    "company_id": org.get("permalink"),
                    "funding_type": props.get("funding_type"),
                    "amount": props.get("money_raised", {}).get("value"),
                    "currency": props.get("money_raised", {}).get("currency"),
                    "announced_on": props.get("announced_on"),
                    "signal_type": "funding_round",
                    "signal_strength": self._calculate_funding_signal_strength(
                        props.get("funding_type"),
                        props.get("money_raised", {}).get("value"),
                    ),
                })

            return self._make_result(
                success=True,
                data={
                    "companies": companies,
                    "total": len(companies),
                    "since_date": since_date,
                },
                credits_used=1,
            )

        except Exception as e:
            logger.error(f"Recent funding search failed: {e}")
            return self._make_result(success=False, error=str(e))

    def _map_funding_stage(self, crunchbase_type: Optional[str]) -> Optional[FundingStage]:
        """Map Crunchbase funding type to our FundingStage enum."""
        if not crunchbase_type:
            return None

        mapping = {
            "pre_seed": FundingStage.PRE_SEED,
            "seed": FundingStage.SEED,
            "series_a": FundingStage.SERIES_A,
            "series_b": FundingStage.SERIES_B,
            "series_c": FundingStage.SERIES_C,
            "series_d": FundingStage.SERIES_D_PLUS,
            "series_e": FundingStage.SERIES_D_PLUS,
            "series_f": FundingStage.SERIES_D_PLUS,
            "private_equity": FundingStage.PRIVATE,
            "ipo": FundingStage.IPO,
            "post_ipo_equity": FundingStage.IPO,
        }

        return mapping.get(crunchbase_type.lower())

    def _calculate_funding_signal_strength(
        self,
        funding_type: Optional[str],
        amount: Optional[int],
    ) -> int:
        """Calculate signal strength (0-100) based on funding."""
        score = 50  # Base score

        # Adjust by funding type
        type_scores = {
            "seed": 60,
            "series_a": 75,
            "series_b": 85,
            "series_c": 90,
            "series_d": 95,
        }
        if funding_type:
            score = type_scores.get(funding_type.lower(), score)

        # Adjust by amount
        if amount:
            if amount >= 100_000_000:
                score = min(100, score + 10)
            elif amount >= 50_000_000:
                score = min(100, score + 5)

        return score
