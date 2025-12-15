# Outreach - Master Plans

> All planned features and implementation roadmaps in one place.
> When starting a session, reference this document.

---

## Current Phase: Phase 1 - Email System

### Phase 1: Email Foundation (NOW)
| Task | Status | Priority |
|------|--------|----------|
| Email service with AWS SES | 🔄 Building | P0 |
| Send single email endpoint | ⏳ Pending | P0 |
| Send bulk emails | ⏳ Pending | P0 |
| Email templates CRUD | ⏳ Pending | P0 |
| Track opens (pixel) | ⏳ Pending | P1 |
| Track clicks (redirect) | ⏳ Pending | P1 |
| Bounce/complaint handling | ⏳ Pending | P1 |
| Unsubscribe management | ⏳ Pending | P1 |
| Auto-delete old emails (2 years) | ⏳ Pending | P2 |

### Phase 2: Lead Scraping
| Task | Status | Priority |
|------|--------|----------|
| LinkedIn profile scraper | ⏳ Pending | P0 |
| Company website scraper | ⏳ Pending | P0 |
| CSV/Excel import | ⏳ Pending | P0 |
| Lead deduplication | ⏳ Pending | P1 |
| Email verification (ZeroBounce) | ⏳ Pending | P1 |
| Apollo.io integration | ⏳ Pending | P2 |
| Hunter.io integration | ⏳ Pending | P2 |

### Phase 3: AI Content Engine
| Task | Status | Priority |
|------|--------|----------|
| Claude API integration | ⏳ Pending | P0 |
| OpenAI API integration | ⏳ Pending | P0 |
| Lead analysis (pros/cons/opportunities) | ⏳ Pending | P0 |
| Email personalization | ⏳ Pending | P0 |
| Subject line generator | ⏳ Pending | P1 |
| CV analyzer | ⏳ Pending | P1 |
| CV improver per job | ⏳ Pending | P1 |
| Cover letter generator | ⏳ Pending | P1 |
| Proposal generator | ⏳ Pending | P2 |

### Phase 4: User Profile & Progress
| Task | Status | Priority |
|------|--------|----------|
| User profile page | ⏳ Pending | P0 |
| Skills inventory | ⏳ Pending | P0 |
| Progress tracking | ⏳ Pending | P0 |
| Learning recommendations | ⏳ Pending | P1 |
| Course links (edX, YouTube) | ⏳ Pending | P1 |
| Achievement system | ⏳ Pending | P2 |
| Career goals tracking | ⏳ Pending | P2 |

### Phase 5: Campaign Management
| Task | Status | Priority |
|------|--------|----------|
| Drip sequences | ⏳ Pending | P0 |
| A/B testing | ⏳ Pending | P1 |
| Optimal send time | ⏳ Pending | P1 |
| Reply detection & auto-stop | ⏳ Pending | P0 |
| Follow-up automation | ⏳ Pending | P1 |

### Phase 6: Voice + Outreach Integration
| Task | Status | Priority |
|------|--------|----------|
| Reply notification system | ⏳ Pending | P0 |
| Lead context injection to desktop | ⏳ Pending | P0 |
| Pre-call preparation | ⏳ Pending | P1 |
| Post-call summary → CRM | ⏳ Pending | P1 |

### Phase 7: Advanced Features
| Task | Status | Priority |
|------|--------|----------|
| Lead scoring (AI) | ⏳ Pending | P1 |
| Intent signals | ⏳ Pending | P2 |
| Multi-channel (LinkedIn, SMS) | ⏳ Pending | P2 |
| Calendar integration | ⏳ Pending | P2 |
| CRM integrations | ⏳ Pending | P2 |
| Team collaboration | ⏳ Pending | P3 |
| White-label | ⏳ Pending | P3 |

---

## Completed Work

### Infrastructure ✅
- [x] Monorepo setup (Outreach)
- [x] API rewritten from Rust to Node.js
- [x] API deployed to Railway (outreachapi.hekax.com)
- [x] Web deployed to Vercel (outreach.hekax.com)
- [x] Database on Neon PostgreSQL
- [x] AWS SES configured
- [x] Desktop app with voice capture

### Database Schema ✅
- [x] Users & authentication
- [x] Leads table
- [x] Recordings table
- [x] Email templates table
- [x] Email log table
- [x] Jobs queue table

---

## Feature Specifications

### Email Sent View
Each sent email displays:
```
┌─────────────────────────────────────────────────────────────────┐
│ EMAIL TO: John Smith                                    [Opened]│
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ RECIPIENT                                                        │
│ • VP of Engineering @ TechCorp                                  │
│ • john.smith@techcorp.com                                       │
│ • LinkedIn: linkedin.com/in/johnsmith                           │
│                                                                  │
│ WHY THIS EMAIL WAS SENT                                         │
│ • Part of "Enterprise Sales Q4" campaign                        │
│ • Match score: 87%                                              │
│ • AI selected for: decision maker, growing company              │
│                                                                  │
│ PROS (Why valuable)                                             │
│ ✓ Decision maker with budget authority                          │
│ ✓ Company growing 40% YoY                                       │
│ ✓ Tech-forward, likely to adopt new solutions                   │
│ ✓ Previously worked at company we closed                        │
│                                                                  │
│ CONS (Prepare for)                                              │
│ ⚠ Busy Q4, might delay decisions                                │
│ ⚠ May have existing vendor relationship                         │
│ ⚠ Large org = longer sales cycle                                │
│                                                                  │
│ AI INSIGHT                                                       │
│ "John's company is scaling their engineering team. Based on     │
│ their job postings and tech stack, they likely need help with   │
│ developer productivity tools. Your experience with similar      │
│ companies (TechA, TechB) could resonate. Consider mentioning    │
│ the 40% efficiency gain case study."                            │
│                                                                  │
│ WHAT THEY COULD HELP WITH                                       │
│ • Enterprise reference customer                                  │
│ • Case study opportunity                                        │
│ • Potential $50k-100k deal                                      │
│ • Network intro to other VPs                                    │
│                                                                  │
│ TRACKING                                                         │
│ Sent: Dec 15, 2024 9:00 AM                                      │
│ Delivered: Dec 15, 2024 9:01 AM                                 │
│ Opened: Dec 15, 2024 2:34 PM (3 times)                          │
│ Clicked: Dec 15, 2024 2:35 PM (link: case-study)                │
│                                                                  │
│ [Mark as Replied] [Schedule Follow-up] [Prepare for Call]       │
└─────────────────────────────────────────────────────────────────┘
```

### User Profile & Progress
```
┌─────────────────────────────────────────────────────────────────┐
│ MY PROFILE                                          [Edit]      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ Ahmed Harhash                                                    │
│ Founder & CEO @ Hekax                                           │
│                                                                  │
│ ═══════════════════════════════════════════════════════════════ │
│                                                                  │
│ YOUR SKILLS                                                      │
│ ┌────────────────────────────────────────────────────────────┐  │
│ │ • Product Development    ████████████████████░░░░ Expert   │  │
│ │ • Sales                  ██████████████░░░░░░░░░░ Advanced │  │
│ │ • Technical Leadership   ████████████████████░░░░ Expert   │  │
│ │ • Negotiation            ████████░░░░░░░░░░░░░░░░ Intermed │  │
│ │ • Public Speaking        ██████░░░░░░░░░░░░░░░░░░ Beginner │  │
│ └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ RECOMMENDED TO DEVELOP                                           │
│                                                                  │
│ 🎯 Negotiation (High Priority)                                  │
│    "Based on your deal sizes, improving negotiation could       │
│     increase close rate by 20%"                                 │
│                                                                  │
│    Free Resources:                                              │
│    📺 YouTube: "Never Split the Difference" Summary (20 min)    │
│    📚 edX: Negotiation and Conflict Resolution (Free)           │
│    🎓 Coursera: Successful Negotiation (Free audit)             │
│                                                                  │
│ 📊 Data Analysis (Medium Priority)                              │
│    "Many leads in your pipeline are data-driven. Speaking       │
│     their language could improve rapport."                      │
│                                                                  │
│    Free Resources:                                              │
│    📺 YouTube: "Excel for Business" (2 hours)                   │
│    📚 edX: Data Analysis Basics (Free)                          │
│                                                                  │
│ ═══════════════════════════════════════════════════════════════ │
│                                                                  │
│ YOUR PROGRESS THIS MONTH                                         │
│ • Leads contacted: 89 (+34%)                                    │
│ • Emails sent: 156                                              │
│ • Reply rate: 18% (above avg)                                   │
│ • Meetings booked: 7                                            │
│ • Deals closed: 2 ($45,000)                                     │
│                                                                  │
│ ═══════════════════════════════════════════════════════════════ │
│                                                                  │
│ YOUR CVS                                                         │
│ • Technical_CV_2024.pdf (Default)                               │
│ • Sales_CV_2024.pdf                                             │
│ • Consulting_CV_2024.pdf                                        │
│ [Upload New] [Generate from Profile]                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Integration Points

### Voice + Outreach Connection

```
OUTREACH                              DESKTOP APP
    │                                      │
    │  Reply received                      │
    ├─────────────────────────────────────►│ Notification
    │                                      │
    │  User marks "Call Scheduled"         │
    ├─────────────────────────────────────►│ Prepare context
    │                                      │
    │                                      │ During call:
    │                                      │ - Lead name
    │                                      │ - Company info
    │                                      │ - Pros/cons
    │                                      │ - Talking points
    │                                      │ - Questions to ask
    │                                      │
    │  Post-call summary                   │
    │◄─────────────────────────────────────┤
    │                                      │
    │  Update lead status                  │
    │  Generate follow-up                  │
```

---

## API Endpoints Needed

### Email Service
```
POST   /emails/send              Send single email
POST   /emails/bulk              Send bulk emails
GET    /emails                   List sent emails
GET    /emails/:id               Get email details with analysis
DELETE /emails/:id               Delete email
POST   /emails/:id/mark-replied  Mark as replied

POST   /templates                Create template
GET    /templates                List templates
PUT    /templates/:id            Update template
DELETE /templates/:id            Delete template

GET    /tracking/open/:emailId   Track email open (pixel)
GET    /tracking/click/:linkId   Track link click (redirect)

POST   /webhooks/ses             SES bounce/complaint webhook
```

### AI Analysis
```
POST   /ai/analyze-lead          Analyze lead (pros/cons/opportunities)
POST   /ai/generate-email        Generate email content
POST   /ai/improve-cv            Analyze and improve CV
POST   /ai/personalize           Personalize template for lead
```

### User Profile
```
GET    /profile                  Get user profile
PUT    /profile                  Update profile
GET    /profile/skills           Get skills inventory
POST   /profile/skills           Add skill
PUT    /profile/skills/:id       Update skill level
GET    /profile/recommendations  Get learning recommendations
GET    /profile/progress         Get progress stats
```

---

## Notes & Ideas

### Future Possibilities
- Mobile app (React Native)
- Browser extension for LinkedIn
- Chrome extension for email tracking
- Slack bot for notifications
- AI voice clone for cold calls
- Video personalization (Loom-like)

### Revenue Model Ideas
- Free: 50 emails/month, basic features
- Pro ($49/mo): 500 emails, AI features, scraping
- Enterprise ($199/mo): Unlimited, team, API access

---

*Last updated: December 15, 2024*
