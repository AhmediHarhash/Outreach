"""Application settings with environment variable support."""

from functools import lru_cache
from typing import Optional

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application configuration loaded from environment variables."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    # Application
    app_name: str = "Outreach Intelligence"
    app_version: str = "0.2.0"
    debug: bool = False
    environment: str = Field(default="development", alias="ENVIRONMENT")

    # Database
    database_url: str = Field(..., alias="DATABASE_URL")
    db_pool_size: int = 5
    db_max_overflow: int = 10

    # Redis (for job queue)
    redis_url: str = Field(default="redis://localhost:6379/0", alias="REDIS_URL")

    # API Keys encryption
    encryption_key: str = Field(..., alias="ENCRYPTION_KEY")

    # Rate limiting defaults (requests per minute)
    default_rate_limit: int = 60

    # Enrichment API rate limits (per minute)
    apollo_rate_limit: int = 100
    clearbit_rate_limit: int = 600
    hunter_rate_limit: int = 25
    crunchbase_rate_limit: int = 200

    # Cache settings (seconds)
    enrichment_cache_ttl: int = 60 * 60 * 24 * 30  # 30 days
    company_cache_ttl: int = 60 * 60 * 24 * 7  # 7 days
    signal_cache_ttl: int = 60 * 60 * 24  # 1 day

    # Job processing
    max_concurrent_jobs: int = 5
    job_timeout_seconds: int = 300
    max_retry_attempts: int = 3

    # Scoring weights (defaults, can be overridden by ICP)
    default_intent_weight: int = 40
    default_fit_weight: int = 35
    default_accessibility_weight: int = 25

    # CORS
    cors_origins: list[str] = Field(
        default=["http://localhost:3000", "https://outreach-web.vercel.app"]
    )

    # Internal API (for callbacks to main API)
    api_base_url: str = Field(
        default="http://localhost:3001", alias="API_BASE_URL"
    )
    api_internal_key: Optional[str] = Field(default=None, alias="API_INTERNAL_KEY")


@lru_cache
def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()
