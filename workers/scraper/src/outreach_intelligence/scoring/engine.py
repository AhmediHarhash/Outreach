"""Lead scoring engine - calculates quality scores based on ICP."""

import logging
from datetime import datetime, timedelta, timezone
from typing import Any, Optional
from uuid import UUID

from ..models.lead import (
    LeadScore,
    ScoreBreakdown,
    ScoreComponent,
    ScoreTier,
    SignalEvent,
)
from ..models.company import Company, FundingStage, CompanySize
from ..models.contact import Contact, SeniorityLevel
from ..models.icp import ICPProfile, ScoringWeights
from ..db.connection import fetch_one, fetch_all, execute_query

logger = logging.getLogger(__name__)


class ScoringEngine:
    """Calculates lead scores based on ICP matching.

    Score = (Intent × weight) + (Fit × weight) + (Accessibility × weight)

    Each component is scored 0-100, then weighted.
    """

    # Default weights if no ICP specified
    DEFAULT_WEIGHTS = ScoringWeights(intent=40, fit=35, accessibility=25)

    def __init__(self, icp: Optional[ICPProfile] = None):
        self.icp = icp
        self.weights = icp.weights if icp else self.DEFAULT_WEIGHTS

    async def score_lead(
        self,
        lead_id: UUID,
        company_data: dict[str, Any],
        contact_data: Optional[dict[str, Any]] = None,
        signals: Optional[list[SignalEvent]] = None,
    ) -> LeadScore:
        """Calculate comprehensive lead score.

        Args:
            lead_id: UUID of the lead
            company_data: Enriched company data
            contact_data: Contact/person data if available
            signals: Active signals for this lead

        Returns:
            LeadScore with breakdown
        """
        breakdown = ScoreBreakdown()

        # Calculate each component
        intent_score, intent_breakdown = self._calculate_intent_score(
            company_data, signals or []
        )
        fit_score, fit_breakdown = self._calculate_fit_score(company_data)
        accessibility_score, accessibility_breakdown = self._calculate_accessibility_score(
            contact_data or {}
        )

        breakdown.intent = intent_breakdown
        breakdown.fit = fit_breakdown
        breakdown.accessibility = accessibility_breakdown

        # Calculate weighted total
        total_score = int(
            (intent_score * self.weights.intent / 100)
            + (fit_score * self.weights.fit / 100)
            + (accessibility_score * self.weights.accessibility / 100)
        )

        # Ensure total is capped at 100
        total_score = min(100, max(0, total_score))

        # Get previous score for change tracking
        previous = await self._get_previous_score(lead_id)

        # Determine tier
        tier = LeadScore.calculate_tier(total_score)

        # Create score record
        score = LeadScore(
            lead_id=lead_id,
            icp_id=self.icp.id if self.icp else None,
            intent_score=intent_score,
            fit_score=fit_score,
            accessibility_score=accessibility_score,
            total_score=total_score,
            tier=tier,
            score_breakdown=breakdown,
            active_signals=[s.signal_type.value for s in (signals or [])],
            previous_score=previous,
            score_change=total_score - previous if previous else None,
        )

        # Save to database
        await self._save_score(score)

        return score

    def _calculate_intent_score(
        self,
        company_data: dict[str, Any],
        signals: list[SignalEvent],
    ) -> tuple[int, dict[str, ScoreComponent]]:
        """Calculate intent score based on buying signals.

        Signals that indicate a company is ready to buy:
        - Recent funding (budget available)
        - Actively hiring (growing)
        - Tech stack changes (evaluating solutions)
        - Leadership changes (new priorities)
        - Industry growth (market timing)
        """
        score = 0
        breakdown: dict[str, ScoreComponent] = {}

        # Recent funding signal (+30)
        last_funding = company_data.get("last_funding_date")
        if last_funding:
            try:
                if isinstance(last_funding, str):
                    funding_date = datetime.fromisoformat(last_funding.replace("Z", "+00:00"))
                else:
                    funding_date = last_funding

                days_ago = (datetime.now(timezone.utc) - funding_date).days

                if days_ago <= 30:
                    points = 30
                    reason = f"Raised funding {days_ago} days ago"
                elif days_ago <= 90:
                    points = 20
                    reason = f"Raised funding {days_ago} days ago"
                elif days_ago <= 180:
                    points = 10
                    reason = f"Raised funding ~{days_ago // 30} months ago"
                else:
                    points = 0
                    reason = ""

                if points > 0:
                    stage = company_data.get("funding_stage")
                    amount = company_data.get("total_funding")
                    if stage:
                        reason += f" ({stage})"
                    if amount:
                        reason += f" - ${amount:,}"

                    score += points
                    breakdown["recent_funding"] = ScoreComponent(
                        points=points,
                        reason=reason,
                        source="company_data",
                    )
            except (ValueError, TypeError):
                pass

        # Hiring signals (+25)
        if company_data.get("is_hiring"):
            open_positions = company_data.get("open_positions", 0)
            if open_positions >= 10:
                points = 25
                reason = f"Actively hiring: {open_positions}+ positions"
            elif open_positions >= 5:
                points = 20
                reason = f"Actively hiring: {open_positions} positions"
            elif open_positions > 0:
                points = 15
                reason = f"Hiring: {open_positions} positions"
            else:
                points = 10
                reason = "Currently hiring"

            score += points
            breakdown["hiring_signals"] = ScoreComponent(
                points=points,
                reason=reason,
                source="company_data",
            )

        # Process explicit signals
        for signal in signals:
            signal_type = signal.signal_type.value

            if signal_type == "tech_adoption" and signal_type not in breakdown:
                # Tech change signal (+20)
                tech = signal.signal_data.get("technology", "new technology")
                score += 20
                breakdown["tech_change"] = ScoreComponent(
                    points=20,
                    reason=f"Recently adopted {tech}",
                    source=signal.source or "signal",
                )

            elif signal_type == "executive_hire" and signal_type not in breakdown:
                # Leadership change (+15)
                title = signal.signal_data.get("title", "executive")
                score += 15
                breakdown["leadership_change"] = ScoreComponent(
                    points=15,
                    reason=f"New {title} hired",
                    source=signal.source or "signal",
                )

            elif signal_type == "news_mention" and "news_coverage" not in breakdown:
                # Press coverage (+10)
                score += 10
                breakdown["news_coverage"] = ScoreComponent(
                    points=10,
                    reason="Recent press coverage",
                    source=signal.source or "signal",
                )

        return min(100, score), breakdown

    def _calculate_fit_score(
        self,
        company_data: dict[str, Any],
    ) -> tuple[int, dict[str, ScoreComponent]]:
        """Calculate fit score based on ICP matching.

        How well does this company match the ideal customer profile:
        - Industry match
        - Company size
        - Technology stack
        - Geographic location
        - Funding stage
        """
        score = 0
        breakdown: dict[str, ScoreComponent] = {}

        # If no ICP, give base score
        if not self.icp:
            return 50, {"default": ScoreComponent(
                points=50,
                reason="No ICP defined - default fit score",
            )}

        filters = self.icp.filters

        # Industry match (+25)
        company_industry = company_data.get("industry", "").lower()
        if filters.industries:
            target_industries = [i.lower() for i in filters.industries]
            excluded = [i.lower() for i in filters.excluded_industries]

            if any(company_industry in ind or ind in company_industry
                   for ind in target_industries):
                score += 25
                breakdown["industry_match"] = ScoreComponent(
                    points=25,
                    reason=f"Industry '{company_industry}' matches target",
                )
            elif company_industry in excluded:
                # Penalty for excluded industry
                score -= 30
                breakdown["industry_excluded"] = ScoreComponent(
                    points=-30,
                    reason=f"Industry '{company_industry}' is excluded",
                )
            else:
                # Partial match for related industries
                score += 10
                breakdown["industry_partial"] = ScoreComponent(
                    points=10,
                    reason=f"Industry '{company_industry}' - partial match",
                )

        # Company size match (+25)
        employee_count = company_data.get("employee_count")
        if employee_count and (filters.company_size_min or filters.company_size_max):
            min_size = filters.company_size_min or 0
            max_size = filters.company_size_max or float("inf")

            if min_size <= employee_count <= max_size:
                score += 25
                breakdown["size_match"] = ScoreComponent(
                    points=25,
                    reason=f"{employee_count} employees - within target range",
                )
            elif employee_count < min_size:
                # Too small
                ratio = employee_count / min_size
                points = int(15 * ratio)
                breakdown["size_small"] = ScoreComponent(
                    points=points,
                    reason=f"{employee_count} employees - below minimum ({min_size})",
                )
                score += points
            else:
                # Too large
                score += 5
                breakdown["size_large"] = ScoreComponent(
                    points=5,
                    reason=f"{employee_count} employees - above maximum ({max_size})",
                )

        # Technology match (+20)
        tech_stack = company_data.get("tech_stack", [])
        if isinstance(tech_stack, list) and filters.tech_requirements.must_have:
            tech_names = [
                t.get("name", t) if isinstance(t, dict) else str(t)
                for t in tech_stack
            ]
            tech_lower = [t.lower() for t in tech_names]

            must_have = [t.lower() for t in filters.tech_requirements.must_have]
            nice_to_have = [t.lower() for t in filters.tech_requirements.nice_to_have]
            avoid = [t.lower() for t in filters.tech_requirements.avoid]

            # Check must-have techs
            must_have_matches = sum(
                1 for t in must_have
                if any(t in tech for tech in tech_lower)
            )
            if must_have_matches > 0:
                points = min(20, 10 + (must_have_matches * 5))
                score += points
                breakdown["tech_must_have"] = ScoreComponent(
                    points=points,
                    reason=f"{must_have_matches}/{len(must_have)} required technologies found",
                )

            # Bonus for nice-to-have
            nice_matches = sum(
                1 for t in nice_to_have
                if any(t in tech for tech in tech_lower)
            )
            if nice_matches > 0:
                points = min(10, nice_matches * 3)
                score += points
                breakdown["tech_nice_to_have"] = ScoreComponent(
                    points=points,
                    reason=f"{nice_matches} bonus technologies found",
                )

            # Penalty for avoid techs
            avoid_matches = sum(
                1 for t in avoid
                if any(t in tech for tech in tech_lower)
            )
            if avoid_matches > 0:
                penalty = min(20, avoid_matches * 10)
                score -= penalty
                breakdown["tech_avoid"] = ScoreComponent(
                    points=-penalty,
                    reason=f"{avoid_matches} red-flag technologies found",
                )

        # Geographic match (+15)
        country = company_data.get("country_code") or company_data.get("country", "")
        if filters.countries and country:
            if country.upper() in [c.upper() for c in filters.countries]:
                score += 15
                breakdown["geo_match"] = ScoreComponent(
                    points=15,
                    reason=f"Location ({country}) matches target regions",
                )
            elif country.upper() in [c.upper() for c in filters.excluded_countries]:
                score -= 20
                breakdown["geo_excluded"] = ScoreComponent(
                    points=-20,
                    reason=f"Location ({country}) is excluded",
                )

        # Funding stage match (+15)
        funding_stage = company_data.get("funding_stage")
        if funding_stage and filters.funding_stages:
            stage_values = [s.value if hasattr(s, "value") else str(s)
                           for s in filters.funding_stages]
            if funding_stage in stage_values or str(funding_stage) in stage_values:
                score += 15
                breakdown["funding_stage_match"] = ScoreComponent(
                    points=15,
                    reason=f"Funding stage ({funding_stage}) matches target",
                )

        return max(0, min(100, score)), breakdown

    def _calculate_accessibility_score(
        self,
        contact_data: dict[str, Any],
    ) -> tuple[int, dict[str, ScoreComponent]]:
        """Calculate accessibility score.

        How easy is it to reach the decision maker:
        - Email found and verified
        - LinkedIn profile available
        - Phone number available
        - Multiple contacts found
        - Previous interaction history
        """
        score = 0
        breakdown: dict[str, ScoreComponent] = {}

        # Email found (+30)
        email = contact_data.get("email")
        if email:
            verified = contact_data.get("email_verified", False)
            confidence = contact_data.get("email_confidence", 0.5)

            if verified:
                score += 30
                breakdown["email_verified"] = ScoreComponent(
                    points=30,
                    reason="Verified email found",
                )
            elif confidence >= 0.8:
                score += 25
                breakdown["email_high_confidence"] = ScoreComponent(
                    points=25,
                    reason=f"Email found ({int(confidence * 100)}% confidence)",
                )
            else:
                score += 15
                breakdown["email_found"] = ScoreComponent(
                    points=15,
                    reason="Email found (unverified)",
                )

        # LinkedIn profile (+25)
        linkedin = contact_data.get("linkedin_url")
        if linkedin:
            score += 25
            breakdown["linkedin_found"] = ScoreComponent(
                points=25,
                reason="LinkedIn profile available",
            )

        # Phone number (+20)
        phone = contact_data.get("phone")
        if phone:
            score += 20
            breakdown["phone_found"] = ScoreComponent(
                points=20,
                reason="Phone number available",
            )

        # Title/seniority match (+15)
        title = contact_data.get("title", "")
        seniority = contact_data.get("seniority")

        if self.icp and self.icp.filters.target_titles:
            target_titles = [t.lower() for t in self.icp.filters.target_titles]
            if any(t in title.lower() for t in target_titles):
                score += 15
                breakdown["title_match"] = ScoreComponent(
                    points=15,
                    reason=f"Title '{title}' matches target",
                )
        elif seniority in [SeniorityLevel.C_LEVEL, SeniorityLevel.VP, SeniorityLevel.DIRECTOR]:
            score += 10
            breakdown["seniority_match"] = ScoreComponent(
                points=10,
                reason=f"Senior contact ({seniority})",
            )

        # Multiple contacts bonus (+10)
        contact_count = contact_data.get("_contact_count", 1)
        if contact_count > 1:
            points = min(10, contact_count * 3)
            score += points
            breakdown["multiple_contacts"] = ScoreComponent(
                points=points,
                reason=f"{contact_count} contacts found at company",
            )

        return min(100, score), breakdown

    async def _get_previous_score(self, lead_id: UUID) -> Optional[int]:
        """Get the previous score for a lead."""
        row = await fetch_one(
            """
            SELECT total_score
            FROM lead_scores
            WHERE lead_id = $1
            ORDER BY calculated_at DESC
            LIMIT 1
            """,
            lead_id,
        )
        return row["total_score"] if row else None

    async def _save_score(self, score: LeadScore) -> None:
        """Save score to database."""
        import json

        await execute_query(
            """
            INSERT INTO lead_scores (
                lead_id, icp_id, intent_score, fit_score, accessibility_score,
                total_score, tier, score_breakdown, active_signals,
                previous_score, score_change, calculated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            """,
            score.lead_id,
            score.icp_id,
            score.intent_score,
            score.fit_score,
            score.accessibility_score,
            score.total_score,
            score.tier.value,
            json.dumps(score.score_breakdown.model_dump()),
            json.dumps(score.active_signals),
            score.previous_score,
            score.score_change,
            score.calculated_at,
        )

    @staticmethod
    async def get_leads_by_tier(
        user_id: UUID,
        tier: ScoreTier,
        limit: int = 50,
    ) -> list[dict[str, Any]]:
        """Get leads by tier for a user."""
        rows = await fetch_all(
            """
            SELECT l.*, ls.total_score, ls.tier, ls.score_breakdown
            FROM leads l
            JOIN lead_current_scores ls ON l.id = ls.lead_id
            WHERE l.user_id = $1 AND ls.tier = $2
            ORDER BY ls.total_score DESC
            LIMIT $3
            """,
            user_id,
            tier.value,
            limit,
        )
        return [dict(row) for row in rows]
