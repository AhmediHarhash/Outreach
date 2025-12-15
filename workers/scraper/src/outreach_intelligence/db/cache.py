"""Enrichment cache for avoiding redundant API calls."""

import hashlib
import json
import logging
from datetime import datetime, timedelta, timezone
from typing import Any, Optional
from uuid import UUID

from .connection import get_db
from ..config import get_settings

logger = logging.getLogger(__name__)


class EnrichmentCache:
    """Cache for enrichment data to avoid redundant API calls."""

    @staticmethod
    def _hash_data(data: dict) -> str:
        """Generate SHA256 hash of data for change detection."""
        serialized = json.dumps(data, sort_keys=True, default=str)
        return hashlib.sha256(serialized.encode()).hexdigest()

    @classmethod
    async def get(
        cls,
        entity_type: str,
        entity_key: str,
        source: str,
    ) -> Optional[dict[str, Any]]:
        """
        Get cached enrichment data.

        Args:
            entity_type: Type of entity (company, person, email)
            entity_key: Unique identifier (domain, email, linkedin_url)
            source: Data source (apollo, clearbit, hunter, crunchbase)

        Returns:
            Cached data if found and not expired, None otherwise
        """
        async with get_db() as conn:
            row = await conn.fetchrow(
                """
                SELECT data, expires_at
                FROM enrichment_cache
                WHERE entity_type = $1
                  AND entity_key = $2
                  AND source = $3
                  AND expires_at > NOW()
                """,
                entity_type,
                entity_key.lower(),
                source,
            )

            if row:
                # Update hit count
                await conn.execute(
                    """
                    UPDATE enrichment_cache
                    SET hit_count = hit_count + 1,
                        last_hit_at = NOW()
                    WHERE entity_type = $1
                      AND entity_key = $2
                      AND source = $3
                    """,
                    entity_type,
                    entity_key.lower(),
                    source,
                )
                logger.debug(
                    f"Cache hit: {entity_type}/{entity_key} from {source}"
                )
                return row["data"]

            return None

    @classmethod
    async def set(
        cls,
        entity_type: str,
        entity_key: str,
        source: str,
        data: dict[str, Any],
        ttl_seconds: Optional[int] = None,
    ) -> None:
        """
        Cache enrichment data.

        Args:
            entity_type: Type of entity
            entity_key: Unique identifier
            source: Data source
            data: Data to cache
            ttl_seconds: Time to live in seconds (defaults from settings)
        """
        settings = get_settings()
        ttl = ttl_seconds or settings.enrichment_cache_ttl
        expires_at = datetime.now(timezone.utc) + timedelta(seconds=ttl)
        data_hash = cls._hash_data(data)

        async with get_db() as conn:
            await conn.execute(
                """
                INSERT INTO enrichment_cache (
                    entity_type, entity_key, source, data, data_hash,
                    fetched_at, expires_at, hit_count
                )
                VALUES ($1, $2, $3, $4, $5, NOW(), $6, 0)
                ON CONFLICT (entity_type, entity_key, source)
                DO UPDATE SET
                    data = EXCLUDED.data,
                    data_hash = EXCLUDED.data_hash,
                    fetched_at = NOW(),
                    expires_at = EXCLUDED.expires_at
                """,
                entity_type,
                entity_key.lower(),
                source,
                json.dumps(data),
                data_hash,
                expires_at,
            )
            logger.debug(
                f"Cached: {entity_type}/{entity_key} from {source} (TTL: {ttl}s)"
            )

    @classmethod
    async def invalidate(
        cls,
        entity_type: str,
        entity_key: str,
        source: Optional[str] = None,
    ) -> int:
        """
        Invalidate cached data.

        Args:
            entity_type: Type of entity
            entity_key: Unique identifier
            source: Optional - invalidate specific source only

        Returns:
            Number of cache entries invalidated
        """
        async with get_db() as conn:
            if source:
                result = await conn.execute(
                    """
                    DELETE FROM enrichment_cache
                    WHERE entity_type = $1
                      AND entity_key = $2
                      AND source = $3
                    """,
                    entity_type,
                    entity_key.lower(),
                    source,
                )
            else:
                result = await conn.execute(
                    """
                    DELETE FROM enrichment_cache
                    WHERE entity_type = $1
                      AND entity_key = $2
                    """,
                    entity_type,
                    entity_key.lower(),
                )

            count = int(result.split()[-1])
            if count > 0:
                logger.info(
                    f"Invalidated {count} cache entries for {entity_type}/{entity_key}"
                )
            return count

    @classmethod
    async def cleanup_expired(cls) -> int:
        """
        Remove expired cache entries.

        Returns:
            Number of entries removed
        """
        async with get_db() as conn:
            result = await conn.execute(
                "SELECT cleanup_enrichment_cache()"
            )
            # The function returns the count
            row = await conn.fetchrow("SELECT cleanup_enrichment_cache()")
            count = row[0] if row else 0
            if count > 0:
                logger.info(f"Cleaned up {count} expired cache entries")
            return count

    @classmethod
    async def has_changed(
        cls,
        entity_type: str,
        entity_key: str,
        source: str,
        new_data: dict[str, Any],
    ) -> bool:
        """
        Check if new data differs from cached data.

        Returns:
            True if data has changed or no cache exists
        """
        async with get_db() as conn:
            row = await conn.fetchrow(
                """
                SELECT data_hash
                FROM enrichment_cache
                WHERE entity_type = $1
                  AND entity_key = $2
                  AND source = $3
                """,
                entity_type,
                entity_key.lower(),
                source,
            )

            if not row:
                return True

            new_hash = cls._hash_data(new_data)
            return row["data_hash"] != new_hash
