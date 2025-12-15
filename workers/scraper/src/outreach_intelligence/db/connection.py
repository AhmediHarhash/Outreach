"""Database connection management with asyncpg."""

import logging
from contextlib import asynccontextmanager
from typing import AsyncGenerator, Optional

import asyncpg
from asyncpg import Pool

from ..config import get_settings

logger = logging.getLogger(__name__)

_pool: Optional[Pool] = None


async def init_db() -> Pool:
    """Initialize database connection pool."""
    global _pool

    if _pool is not None:
        return _pool

    settings = get_settings()

    try:
        _pool = await asyncpg.create_pool(
            settings.database_url,
            min_size=2,
            max_size=settings.db_pool_size + settings.db_max_overflow,
            command_timeout=60,
            statement_cache_size=100,
        )
        logger.info("Database connection pool initialized")
        return _pool
    except Exception as e:
        logger.error(f"Failed to initialize database pool: {e}")
        raise


async def close_db() -> None:
    """Close database connection pool."""
    global _pool

    if _pool is not None:
        await _pool.close()
        _pool = None
        logger.info("Database connection pool closed")


async def get_pool() -> Pool:
    """Get the database connection pool."""
    global _pool

    if _pool is None:
        _pool = await init_db()

    return _pool


@asynccontextmanager
async def get_db() -> AsyncGenerator[asyncpg.Connection, None]:
    """Get a database connection from the pool."""
    pool = await get_pool()

    async with pool.acquire() as connection:
        yield connection


async def execute_query(query: str, *args) -> str:
    """Execute a query and return status."""
    async with get_db() as conn:
        return await conn.execute(query, *args)


async def fetch_one(query: str, *args) -> Optional[asyncpg.Record]:
    """Fetch a single row."""
    async with get_db() as conn:
        return await conn.fetchrow(query, *args)


async def fetch_all(query: str, *args) -> list[asyncpg.Record]:
    """Fetch all rows."""
    async with get_db() as conn:
        return await conn.fetch(query, *args)
