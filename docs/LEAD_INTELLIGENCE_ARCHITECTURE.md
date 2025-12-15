# Lead Intelligence System - Architecture Design

## Philosophy: Quality Over Quantity

We're not building a scraper. We're building an **intelligence system** that finds the **right leads at the right time** for our customers.

### Core Principles
1. **Signal-Based Discovery** - Find leads showing buying signals, not random companies
2. **Multi-Source Verification** - Cross-reference data for accuracy
3. **Timing Intelligence** - Catch companies when they're ready to buy
4. **Ethical Data Collection** - API-first, respect ToS, no aggressive scraping
5. **Personalization** - Match leads to user's Ideal Customer Profile (ICP)

---

## Quality Signals We Track

### Tier 1: High-Intent Signals (Immediate Opportunity)
| Signal | Why It Matters | Data Source |
|--------|---------------|-------------|
| Just raised funding | Has budget, growing fast | Crunchbase, News APIs |
| Actively hiring | Expanding, has problems to solve | Job boards, LinkedIn |
| Tech stack change | Evaluating new solutions | BuiltWith, Wappalyzer |
| Leadership change | New decision makers, fresh budget | LinkedIn, News |
| Contract renewal season | Re-evaluating vendors | Industry knowledge |

### Tier 2: Fit Signals (Good Match)
| Signal | Why It Matters | Data Source |
|--------|---------------|-------------|
| Uses competitor product | Already understands the space | G2, Tech detection |
| Company size sweet spot | Budget + agility | Clearbit, Apollo |
| Industry match | Relevant pain points | Company data |
| Tech stack compatibility | Easy integration | BuiltWith |
| Geographic fit | Timezone, regulations | Company data |

### Tier 3: Engagement Signals (Warm Lead)
| Signal | Why It Matters | Data Source |
|--------|---------------|-------------|
| Visited pricing page | Active evaluation | First-party (if available) |
| Downloaded content | Researching solutions | First-party |
| Replied to email | Engaged | Internal tracking |
| Connected on LinkedIn | Relationship started | LinkedIn |

---

## Lead Scoring Model

```
LEAD_SCORE = (Intent × 40%) + (Fit × 35%) + (Accessibility × 25%)

Intent Score (0-100):
- Recent funding: +30
- Actively hiring in relevant roles: +25
- Tech stack change: +20
- Leadership change: +15
- Industry growth trend: +10

Fit Score (0-100):
- ICP industry match: +25
- Company size in range: +25
- Tech stack compatibility: +20
- Geographic fit: +15
- Uses complementary tools: +15

Accessibility Score (0-100):
- Direct email found: +30
- LinkedIn profile available: +25
- Phone number available: +20
- Multiple contacts found: +15
- Previous interaction: +10
```

### Score Tiers
| Score | Tier | Action |
|-------|------|--------|
| 80-100 | 🔥 Hot | Immediate outreach, personalized |
| 60-79 | 🌟 Warm | Priority queue, semi-personalized |
| 40-59 | 📊 Nurture | Add to drip campaign |
| 0-39 | ❄️ Cold | Monitor for signal changes |

---

## Data Sources (Ethical, API-First)

### Primary Sources (Recommended)
| Source | Data Provided | Integration |
|--------|--------------|-------------|
| **Apollo.io** | Contact info, company data, intent | API (user's key) |
| **Clearbit** | Company enrichment, tech stack | API |
| **Hunter.io** | Email verification, finding | API |
| **Crunchbase** | Funding, investors, news | API |
| **BuiltWith** | Technology detection | API |

### Secondary Sources (Supplementary)
| Source | Data Provided | Method |
|--------|--------------|--------|
| Company website | About, team, tech | Respectful crawl |
| Job boards | Hiring signals | API/RSS |
| News APIs | Funding, events | API |
| Google News | Recent mentions | API |

### User-Provided (Highest Quality)
| Source | Data Provided | Method |
|--------|--------------|--------|
| LinkedIn paste | Full profile | Manual input |
| CRM import | Existing relationships | CSV/API |
| Business card | Direct contact | OCR/Manual |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        OUTREACH WEB APP                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Lead Discovery│  │ ICP Settings │  │ Lead Inbox   │          │
│  │    Search     │  │   Builder    │  │  (Scored)    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────▲───────┘          │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          ▼                 ▼                 │
┌─────────────────────────────────────────────┼───────────────────┐
│                    OUTREACH API             │                    │
│  ┌──────────────┐  ┌──────────────┐  ┌─────┴────────┐          │
│  │ /discover    │  │ /icp         │  │ /leads       │          │
│  │  (trigger)   │  │  (profile)   │  │ (enriched)   │          │
│  └──────┬───────┘  └──────┬───────┘  └──────▲───────┘          │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          ▼                 ▼                 │
┌─────────────────────────────────────────────┼───────────────────┐
│              INTELLIGENCE WORKER (Python)    │                   │
│  ┌────────────────────────────────────────────────────────┐     │
│  │                    JOB QUEUE                            │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │     │
│  │  │ Enrich  │ │ Score   │ │ Discover│ │ Monitor │      │     │
│  │  │  Lead   │ │  Lead   │ │  Leads  │ │ Signals │      │     │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘      │     │
│  └───────┼───────────┼───────────┼───────────┼────────────┘     │
│          ▼           ▼           ▼           ▼                   │
│  ┌────────────────────────────────────────────────────────┐     │
│  │                 ENRICHMENT ENGINE                       │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │     │
│  │  │ Apollo  │ │Clearbit │ │ Hunter  │ │Crunchbase│     │     │
│  │  │ Client  │ │ Client  │ │ Client  │ │ Client  │      │     │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘      │     │
│  └────────────────────────────────────────────────────────┘     │
│          │                                                       │
│          ▼                                                       │
│  ┌────────────────────────────────────────────────────────┐     │
│  │                  SCORING ENGINE                         │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐                  │     │
│  │  │ Intent  │ │   Fit   │ │ Access  │ → Final Score    │     │
│  │  │ Scorer  │ │ Scorer  │ │ Scorer  │                  │     │
│  │  └─────────┘ └─────────┘ └─────────┘                  │─────┘
│  └────────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────┘
```

---

## ICP (Ideal Customer Profile) Builder

Users define their ideal customer:

```json
{
  "industries": ["SaaS", "FinTech", "HealthTech"],
  "companySizeMin": 50,
  "companySizeMax": 500,
  "fundingStages": ["Series A", "Series B", "Series C"],
  "techStack": {
    "mustHave": ["AWS", "React"],
    "niceToHave": ["Python", "PostgreSQL"],
    "avoid": ["Legacy systems"]
  },
  "geographies": ["US", "UK", "Canada"],
  "decisionMakers": {
    "titles": ["CTO", "VP Engineering", "Head of Product"],
    "departments": ["Engineering", "Product"]
  },
  "signals": {
    "recentFunding": true,
    "activelyHiring": true,
    "techStackChange": false
  }
}
```

---

## Workflow Examples

### 1. Proactive Discovery
```
User sets ICP → Worker runs daily →
Finds matching companies with signals →
Scores and ranks → Delivers to inbox
```

### 2. On-Demand Enrichment
```
User adds lead manually → API triggers enrichment →
Worker fetches from all sources →
Calculates score → Updates lead with full data
```

### 3. Signal Monitoring
```
User marks leads as "watching" →
Worker monitors for signals daily →
Alert when score changes significantly
```

---

## Rate Limiting & Ethics

### API Rate Limits
| Service | Limit | Our Approach |
|---------|-------|--------------|
| Apollo | 100/min | Queue with 700ms delay |
| Clearbit | 600/min | Queue with 100ms delay |
| Hunter | 25/search | Cache results 30 days |
| Crunchbase | 200/min | Queue with 400ms delay |

### Ethical Guidelines
1. **Never scrape aggressively** - API first, always
2. **Respect robots.txt** - When crawling company sites
3. **Cache intelligently** - Don't re-fetch same data
4. **User consent** - User provides their own API keys
5. **Data minimization** - Only collect what's needed
6. **Transparency** - Show users where data comes from

---

## Database Schema Additions

```sql
-- ICP Profiles
CREATE TABLE icp_profiles (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    name VARCHAR(100),
    config JSONB,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enrichment Cache
CREATE TABLE enrichment_cache (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(50),  -- company, person
    entity_key VARCHAR(255),  -- domain, email, linkedin_url
    source VARCHAR(50),       -- apollo, clearbit, hunter
    data JSONB,
    fetched_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    UNIQUE(entity_type, entity_key, source)
);

-- Lead Scores (historical)
CREATE TABLE lead_scores (
    id UUID PRIMARY KEY,
    lead_id UUID REFERENCES leads(id),
    intent_score INTEGER,
    fit_score INTEGER,
    accessibility_score INTEGER,
    total_score INTEGER,
    score_breakdown JSONB,
    calculated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Signal Events
CREATE TABLE signal_events (
    id UUID PRIMARY KEY,
    lead_id UUID REFERENCES leads(id),
    signal_type VARCHAR(50),
    signal_data JSONB,
    detected_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## Implementation Priority

1. **ICP Builder** - Let users define what "quality" means to them
2. **Scoring Engine** - Rank leads intelligently
3. **Enrichment Pipeline** - Multi-source data fetching
4. **Signal Detection** - Find timing opportunities
5. **Discovery UI** - Search and filter with intelligence
