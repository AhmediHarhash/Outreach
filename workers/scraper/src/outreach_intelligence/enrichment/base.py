"""Base enrichment client with rate limiting."""

import logging
from abc import ABC, abstractmethod
from typing import Any, Optional

import httpx
from aiolimiter import AsyncLimiter

from ..config import get_settings
from ..db.cache import EnrichmentCache
from ..models.enrichment import EnrichmentResult

logger = logging.getLogger(__name__)


class RateLimitedClient:
    """HTTP client with rate limiting support."""

    def __init__(
        self,
        base_url: str,
        api_key: str,
        rate_limit: int,  # requests per minute
        timeout: int = 30,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.rate_limit = rate_limit
        self.timeout = timeout

        # Rate limiter: X requests per minute
        self._limiter = AsyncLimiter(rate_limit, 60)

        # HTTP client
        self._client: Optional[httpx.AsyncClient] = None

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create HTTP client."""
        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                base_url=self.base_url,
                timeout=self.timeout,
                headers=self._get_headers(),
            )
        return self._client

    def _get_headers(self) -> dict[str, str]:
        """Get default headers. Override in subclass."""
        return {"Content-Type": "application/json"}

    async def request(
        self,
        method: str,
        endpoint: str,
        params: Optional[dict] = None,
        json: Optional[dict] = None,
        headers: Optional[dict] = None,
    ) -> httpx.Response:
        """Make rate-limited HTTP request."""
        async with self._limiter:
            client = await self._get_client()
            response = await client.request(
                method=method,
                url=endpoint,
                params=params,
                json=json,
                headers=headers,
            )
            return response

    async def get(
        self,
        endpoint: str,
        params: Optional[dict] = None,
    ) -> httpx.Response:
        """Make GET request."""
        return await self.request("GET", endpoint, params=params)

    async def post(
        self,
        endpoint: str,
        json: Optional[dict] = None,
        params: Optional[dict] = None,
    ) -> httpx.Response:
        """Make POST request."""
        return await self.request("POST", endpoint, json=json, params=params)

    async def close(self) -> None:
        """Close HTTP client."""
        if self._client and not self._client.is_closed:
            await self._client.aclose()
            self._client = None


class BaseEnrichmentClient(ABC):
    """Base class for enrichment API clients."""

    SOURCE_NAME: str = "unknown"
    ENTITY_TYPE: str = "unknown"

    def __init__(self, api_key: str, rate_limit: int):
        self.api_key = api_key
        self.rate_limit = rate_limit
        self._http: Optional[RateLimitedClient] = None

    @property
    @abstractmethod
    def base_url(self) -> str:
        """Base URL for the API."""
        pass

    def _get_http_client(self) -> RateLimitedClient:
        """Get HTTP client."""
        if self._http is None:
            self._http = RateLimitedClient(
                base_url=self.base_url,
                api_key=self.api_key,
                rate_limit=self.rate_limit,
            )
        return self._http

    async def _check_cache(
        self,
        entity_key: str,
    ) -> Optional[dict[str, Any]]:
        """Check cache for existing data."""
        return await EnrichmentCache.get(
            entity_type=self.ENTITY_TYPE,
            entity_key=entity_key,
            source=self.SOURCE_NAME,
        )

    async def _cache_result(
        self,
        entity_key: str,
        data: dict[str, Any],
        ttl_seconds: Optional[int] = None,
    ) -> None:
        """Cache enrichment result."""
        await EnrichmentCache.set(
            entity_type=self.ENTITY_TYPE,
            entity_key=entity_key,
            source=self.SOURCE_NAME,
            data=data,
            ttl_seconds=ttl_seconds,
        )

    def _make_result(
        self,
        success: bool,
        data: Optional[dict] = None,
        error: Optional[str] = None,
        credits_used: int = 0,
        cached: bool = False,
    ) -> EnrichmentResult:
        """Create enrichment result."""
        return EnrichmentResult(
            success=success,
            source=self.SOURCE_NAME,
            data=data or {},
            error=error,
            credits_used=credits_used,
            cached=cached,
        )

    async def close(self) -> None:
        """Close client resources."""
        if self._http:
            await self._http.close()
            self._http = None

    @abstractmethod
    async def enrich_company(self, domain: str) -> EnrichmentResult:
        """Enrich company by domain."""
        pass

    @abstractmethod
    async def find_contacts(
        self,
        domain: str,
        titles: Optional[list[str]] = None,
        limit: int = 5,
    ) -> EnrichmentResult:
        """Find contacts at a company."""
        pass

    @abstractmethod
    async def verify_email(self, email: str) -> EnrichmentResult:
        """Verify an email address."""
        pass
