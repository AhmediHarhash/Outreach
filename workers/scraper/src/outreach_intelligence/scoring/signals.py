"""Signal detection for lead intelligence."""

import logging
from datetime import datetime, timedelta, timezone
from typing import Any, Optional
from uuid import UUID

from ..models.lead import SignalEvent, SignalType, SignalCategory
from ..db.connection import fetch_all, fetch_one, execute_query, get_db

logger = logging.getLogger(__name__)


class SignalDetector:
    """Detects quality signals from company and contact data.

    Signals are categorized into:
    - Intent: Buying signals (funding, hiring, tech changes)
    - Fit: ICP match signals (industry, size, geography)
    - Engagement: Interaction signals (replies, opens, connections)
    """

    # Signal expiration in days
    SIGNAL_TTL = {
        SignalType.FUNDING_ROUND: 90,
        SignalType.EXECUTIVE_HIRE: 60,
        SignalType.JOB_POSTING: 30,
        SignalType.TECH_ADOPTION: 60,
        SignalType.NEWS_MENTION: 14,
        SignalType.GROWTH_INDICATOR: 90,
        SignalType.CONTRACT_ENDING: 30,
        SignalType.WEBSITE_CHANGE: 7,
    }

    # Score impact by signal type
    SIGNAL_IMPACT = {
        SignalType.FUNDING_ROUND: 30,
        SignalType.EXECUTIVE_HIRE: 15,
        SignalType.JOB_POSTING: 20,
        SignalType.TECH_ADOPTION: 20,
        SignalType.NEWS_MENTION: 10,
        SignalType.GROWTH_INDICATOR: 15,
        SignalType.CONTRACT_ENDING: 25,
        SignalType.WEBSITE_CHANGE: 5,
    }

    @classmethod
    def detect_signals(
        cls,
        company_data: dict[str, Any],
        previous_data: Optional[dict[str, Any]] = None,
    ) -> list[SignalEvent]:
        """Detect signals from company data.

        Compares current data with previous data to find changes.

        Args:
            company_data: Current enriched company data
            previous_data: Previously stored company data for comparison

        Returns:
            List of detected signal events
        """
        signals: list[SignalEvent] = []
        now = datetime.now(timezone.utc)

        # Funding signal
        funding_signal = cls._detect_funding_signal(company_data, previous_data)
        if funding_signal:
            signals.append(funding_signal)

        # Hiring signal
        hiring_signal = cls._detect_hiring_signal(company_data, previous_data)
        if hiring_signal:
            signals.append(hiring_signal)

        # Tech adoption signal
        tech_signals = cls._detect_tech_changes(company_data, previous_data)
        signals.extend(tech_signals)

        # Growth signal
        growth_signal = cls._detect_growth_signal(company_data, previous_data)
        if growth_signal:
            signals.append(growth_signal)

        # News signal
        news_signals = cls._detect_news_signals(company_data)
        signals.extend(news_signals)

        return signals

    @classmethod
    def _detect_funding_signal(
        cls,
        data: dict[str, Any],
        prev: Optional[dict[str, Any]],
    ) -> Optional[SignalEvent]:
        """Detect recent funding rounds."""
        last_funding = data.get("last_funding_date")
        if not last_funding:
            return None

        try:
            if isinstance(last_funding, str):
                funding_date = datetime.fromisoformat(last_funding.replace("Z", "+00:00"))
            else:
                funding_date = last_funding

            # Only signal if funding is recent (within 90 days)
            days_ago = (datetime.now(timezone.utc) - funding_date).days
            if days_ago > 90:
                return None

            # Check if this is a new funding event
            if prev:
                prev_funding = prev.get("last_funding_date")
                if prev_funding == last_funding:
                    return None  # Already known

            # Calculate confidence based on recency
            confidence = max(0.5, 1 - (days_ago / 90))

            return SignalEvent(
                signal_type=SignalType.FUNDING_ROUND,
                signal_category=SignalCategory.INTENT,
                signal_data={
                    "stage": data.get("funding_stage"),
                    "amount": data.get("total_funding"),
                    "date": str(funding_date),
                    "days_ago": days_ago,
                },
                score_impact=cls.SIGNAL_IMPACT[SignalType.FUNDING_ROUND],
                confidence=confidence,
                source=", ".join(data.get("sources", [])),
                signal_date=funding_date,
                expires_at=datetime.now(timezone.utc) + timedelta(
                    days=cls.SIGNAL_TTL[SignalType.FUNDING_ROUND]
                ),
            )

        except (ValueError, TypeError) as e:
            logger.debug(f"Could not parse funding date: {e}")
            return None

    @classmethod
    def _detect_hiring_signal(
        cls,
        data: dict[str, Any],
        prev: Optional[dict[str, Any]],
    ) -> Optional[SignalEvent]:
        """Detect hiring activity."""
        is_hiring = data.get("is_hiring", False)
        open_positions = data.get("open_positions", 0)

        if not is_hiring or open_positions == 0:
            return None

        # Check if hiring has increased
        prev_positions = prev.get("open_positions", 0) if prev else 0
        if prev_positions >= open_positions:
            return None  # Not increasing

        # Calculate confidence based on number of positions
        confidence = min(1.0, 0.5 + (open_positions / 20))

        return SignalEvent(
            signal_type=SignalType.JOB_POSTING,
            signal_category=SignalCategory.INTENT,
            signal_data={
                "open_positions": open_positions,
                "previous_positions": prev_positions,
                "increase": open_positions - prev_positions,
            },
            score_impact=cls.SIGNAL_IMPACT[SignalType.JOB_POSTING],
            confidence=confidence,
            source=", ".join(data.get("sources", [])),
            signal_date=datetime.now(timezone.utc),
            expires_at=datetime.now(timezone.utc) + timedelta(
                days=cls.SIGNAL_TTL[SignalType.JOB_POSTING]
            ),
        )

    @classmethod
    def _detect_tech_changes(
        cls,
        data: dict[str, Any],
        prev: Optional[dict[str, Any]],
    ) -> list[SignalEvent]:
        """Detect technology stack changes."""
        signals = []

        current_tech = set()
        tech_stack = data.get("tech_stack", [])
        for tech in tech_stack:
            if isinstance(tech, dict):
                current_tech.add(tech.get("name", "").lower())
            else:
                current_tech.add(str(tech).lower())

        prev_tech = set()
        if prev:
            for tech in prev.get("tech_stack", []):
                if isinstance(tech, dict):
                    prev_tech.add(tech.get("name", "").lower())
                else:
                    prev_tech.add(str(tech).lower())

        # Find new technologies
        new_techs = current_tech - prev_tech

        for tech in list(new_techs)[:3]:  # Limit to 3 signals
            signals.append(SignalEvent(
                signal_type=SignalType.TECH_ADOPTION,
                signal_category=SignalCategory.INTENT,
                signal_data={
                    "technology": tech,
                    "category": "unknown",  # Would need lookup table
                },
                score_impact=cls.SIGNAL_IMPACT[SignalType.TECH_ADOPTION],
                confidence=0.7,
                source=", ".join(data.get("sources", [])),
                signal_date=datetime.now(timezone.utc),
                expires_at=datetime.now(timezone.utc) + timedelta(
                    days=cls.SIGNAL_TTL[SignalType.TECH_ADOPTION]
                ),
            ))

        return signals

    @classmethod
    def _detect_growth_signal(
        cls,
        data: dict[str, Any],
        prev: Optional[dict[str, Any]],
    ) -> Optional[SignalEvent]:
        """Detect company growth indicators."""
        if not prev:
            return None

        current_employees = data.get("employee_count")
        prev_employees = prev.get("employee_count")

        if not current_employees or not prev_employees:
            return None

        # Calculate growth rate
        growth_rate = (current_employees - prev_employees) / prev_employees

        # Only signal if growth is significant (>20%)
        if growth_rate < 0.2:
            return None

        confidence = min(1.0, 0.5 + (growth_rate / 2))

        return SignalEvent(
            signal_type=SignalType.GROWTH_INDICATOR,
            signal_category=SignalCategory.INTENT,
            signal_data={
                "current_employees": current_employees,
                "previous_employees": prev_employees,
                "growth_rate": round(growth_rate * 100, 1),
            },
            score_impact=cls.SIGNAL_IMPACT[SignalType.GROWTH_INDICATOR],
            confidence=confidence,
            source=", ".join(data.get("sources", [])),
            signal_date=datetime.now(timezone.utc),
            expires_at=datetime.now(timezone.utc) + timedelta(
                days=cls.SIGNAL_TTL[SignalType.GROWTH_INDICATOR]
            ),
        )

    @classmethod
    def _detect_news_signals(
        cls,
        data: dict[str, Any],
    ) -> list[SignalEvent]:
        """Detect relevant news mentions."""
        signals = []
        recent_news = data.get("recent_news", [])

        for news in recent_news[:2]:  # Limit to 2 news signals
            title = news.get("title", "")
            url = news.get("url")
            posted_on = news.get("posted_on")

            # Skip if too old
            if posted_on:
                try:
                    if isinstance(posted_on, str):
                        news_date = datetime.fromisoformat(posted_on.replace("Z", "+00:00"))
                    else:
                        news_date = posted_on

                    days_ago = (datetime.now(timezone.utc) - news_date).days
                    if days_ago > 14:
                        continue
                except (ValueError, TypeError):
                    continue
            else:
                news_date = datetime.now(timezone.utc)

            signals.append(SignalEvent(
                signal_type=SignalType.NEWS_MENTION,
                signal_category=SignalCategory.INTENT,
                signal_data={
                    "title": title,
                    "publisher": news.get("publisher"),
                },
                score_impact=cls.SIGNAL_IMPACT[SignalType.NEWS_MENTION],
                confidence=0.6,
                source="crunchbase",
                source_url=url,
                signal_date=news_date,
                expires_at=datetime.now(timezone.utc) + timedelta(
                    days=cls.SIGNAL_TTL[SignalType.NEWS_MENTION]
                ),
            ))

        return signals

    @classmethod
    async def save_signals(
        cls,
        signals: list[SignalEvent],
        lead_id: Optional[UUID] = None,
        company_domain: Optional[str] = None,
    ) -> int:
        """Save detected signals to database.

        Returns:
            Number of signals saved
        """
        import json

        saved = 0
        for signal in signals:
            try:
                await execute_query(
                    """
                    INSERT INTO signal_events (
                        lead_id, company_domain, signal_type, signal_category,
                        signal_data, score_impact, confidence, source, source_url,
                        signal_date, detected_at, expires_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    """,
                    lead_id,
                    company_domain,
                    signal.signal_type.value,
                    signal.signal_category.value,
                    json.dumps(signal.signal_data),
                    signal.score_impact,
                    signal.confidence,
                    signal.source,
                    signal.source_url,
                    signal.signal_date,
                    signal.detected_at,
                    signal.expires_at,
                )
                saved += 1
            except Exception as e:
                logger.error(f"Failed to save signal: {e}")

        return saved

    @classmethod
    async def get_active_signals(
        cls,
        lead_id: Optional[UUID] = None,
        company_domain: Optional[str] = None,
    ) -> list[SignalEvent]:
        """Get active (non-expired) signals for a lead or company."""
        if lead_id:
            rows = await fetch_all(
                """
                SELECT * FROM signal_events
                WHERE lead_id = $1
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY detected_at DESC
                """,
                lead_id,
            )
        elif company_domain:
            rows = await fetch_all(
                """
                SELECT * FROM signal_events
                WHERE company_domain = $1
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY detected_at DESC
                """,
                company_domain,
            )
        else:
            return []

        return [
            SignalEvent(
                id=row["id"],
                lead_id=row["lead_id"],
                company_domain=row["company_domain"],
                signal_type=SignalType(row["signal_type"]),
                signal_category=SignalCategory(row["signal_category"]),
                signal_data=row["signal_data"],
                score_impact=row["score_impact"],
                confidence=row["confidence"],
                source=row["source"],
                source_url=row["source_url"],
                signal_date=row["signal_date"],
                detected_at=row["detected_at"],
                expires_at=row["expires_at"],
                is_processed=row["is_processed"],
                processed_at=row["processed_at"],
            )
            for row in rows
        ]

    @classmethod
    async def cleanup_expired_signals(cls) -> int:
        """Remove expired signals from database.

        Returns:
            Number of signals removed
        """
        result = await execute_query(
            "DELETE FROM signal_events WHERE expires_at < NOW()"
        )
        count = int(result.split()[-1]) if result else 0
        if count > 0:
            logger.info(f"Cleaned up {count} expired signals")
        return count
