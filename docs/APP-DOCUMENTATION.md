# Outreach - Complete Application Documentation

> **Version:** 1.0.0
> **Last Updated:** December 15, 2024
> **Visionaries:** AhmediHarhash & Claude
> **Mission:** Build the world's best AI-powered outreach platform

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Modules](#core-modules)
4. [User Features](#user-features)
5. [AI Systems](#ai-systems)
6. [Data Flow](#data-flow)
7. [API Reference](#api-reference)
8. [Deployment](#deployment)

---

## Overview

### What is Outreach?

Outreach is an all-in-one AI-powered platform that helps professionals:
- **Find leads** through intelligent web scraping
- **Understand leads** with AI-powered analysis
- **Communicate** with personalized, professional emails
- **Track progress** with smart analytics
- **Grow skills** with personalized learning recommendations
- **Close deals** with real-time voice AI assistance

### Unique Value Proposition

| Feature | Description | Competitors |
|---------|-------------|-------------|
| Voice Capture + AI Hints | Real-time assistance during calls | ❌ None have this |
| Claude-Powered Content | Highest quality AI writing | Most use GPT-3.5 |
| CV Customization | Per-job resume optimization | ❌ None |
| Lead Intelligence | Pros/cons analysis + opportunity matching | Basic in others |
| Skill Development | Learning path recommendations | ❌ None |
| Desktop Stealth Mode | Invisible overlay during calls | ❌ None |

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          OUTREACH PLATFORM                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐               │
│  │   WEB APP   │     │ DESKTOP APP │     │  MOBILE APP │               │
│  │  (Next.js)  │     │(Rust/Dioxus)│     │  (Future)   │               │
│  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘               │
│         │                   │                   │                       │
│         └───────────────────┼───────────────────┘                       │
│                             ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      API GATEWAY (Node.js)                       │   │
│  │  • Authentication  • Rate Limiting  • Request Routing            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                             │                                           │
│         ┌───────────────────┼───────────────────┐                       │
│         ▼                   ▼                   ▼                       │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐               │
│  │   AI ENGINE │     │EMAIL SERVICE│     │SCRAPER WORKER│              │
│  │Claude+OpenAI│     │  (AWS SES)  │     │  (Python)   │               │
│  └─────────────┘     └─────────────┘     └─────────────┘               │
│         │                   │                   │                       │
│         └───────────────────┼───────────────────┘                       │
│                             ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    DATABASE (PostgreSQL)                         │   │
│  │  • Users  • Leads  • Emails  • Templates  • Analytics            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Repository Structure

```
Outreach/
├── apps/
│   ├── api/                 # Backend API (Node.js/Express)
│   │   ├── src/
│   │   │   ├── routes/      # API endpoints
│   │   │   ├── services/    # Business logic
│   │   │   ├── middleware/  # Auth, validation
│   │   │   └── db/          # Database migrations
│   │   └── Dockerfile
│   │
│   ├── web/                 # Web Dashboard (Next.js)
│   │   ├── src/
│   │   │   ├── app/         # Pages
│   │   │   ├── components/  # UI components
│   │   │   └── lib/         # Utilities
│   │   └── package.json
│   │
│   └── desktop/             # Desktop App (Rust/Dioxus)
│       ├── src/
│       │   ├── ui/          # UI components
│       │   ├── audio/       # Voice capture
│       │   ├── brain/       # AI pipeline
│       │   └── config/      # Settings
│       └── Cargo.toml
│
├── workers/
│   └── scraper/             # Python scraping worker
│       ├── scrapers/        # LinkedIn, websites, etc.
│       ├── enrichers/       # Data enrichment
│       └── requirements.txt
│
├── docs/                    # Documentation
│   ├── APP-DOCUMENTATION.md
│   ├── API-REFERENCE.md
│   ├── FEATURES.md
│   └── PLANS.md
│
└── packages/
    └── shared/              # Shared types (future)
```

### Deployment

| Component | Platform | URL |
|-----------|----------|-----|
| API | Railway | outreachapi.hekax.com |
| Web | Vercel | outreach.hekax.com |
| Scraper Worker | Railway | (internal) |
| Desktop | Local .exe | N/A |
| Database | Neon | (PostgreSQL) |

---

## Core Modules

### 1. Lead Management

**Purpose:** Store, organize, and analyze potential contacts.

**Lead Data Structure:**
```typescript
interface Lead {
  id: string;
  userId: string;

  // Contact Information
  firstName: string;
  lastName: string;
  email: string;
  phone?: string;
  linkedinUrl?: string;
  title: string;

  // Company Information
  companyName: string;
  companyDomain?: string;
  companySize?: string;
  industry?: string;
  location?: string;

  // AI Analysis
  pros: string[];              // Strengths of this lead
  cons: string[];              // Challenges/things to prepare for
  opportunities: string[];     // How this lead could help user
  aiSummary: string;           // Quick AI-generated summary
  matchScore: number;          // 0-100 compatibility score

  // Skills Matching
  requiredSkills?: string[];   // Skills they're looking for
  userMatchingSkills?: string[]; // User skills that match
  skillGaps?: string[];        // Skills user should develop

  // Outreach Status
  status: 'new' | 'researching' | 'contacted' | 'replied' | 'meeting' | 'won' | 'lost';
  emailsSent: number;
  emailsOpened: number;
  lastContactedAt?: Date;
  nextFollowUpAt?: Date;

  // Metadata
  source: string;              // Where lead came from
  tags: string[];
  notes: string;
  createdAt: Date;
  updatedAt: Date;
}
```

### 2. Email System

**Purpose:** Send professional, tracked emails at scale.

**Email Features:**
- Single and bulk sending via AWS SES
- Template management with variables
- Open tracking (invisible pixel)
- Click tracking (redirect links)
- Reply detection
- Bounce/complaint handling
- Auto-cleanup of unreplied emails (after 2 years)

**Email Data Structure:**
```typescript
interface Email {
  id: string;
  userId: string;
  leadId: string;

  // Content
  subject: string;
  bodyHtml: string;
  bodyText: string;
  templateId?: string;

  // AI Context
  purpose: string;           // Why this email was sent
  leadPros: string[];        // What makes this lead valuable
  leadCons: string[];        // Things to be aware of
  aiReasoning: string;       // OpenAI analysis of opportunity

  // Attachments
  attachments: {
    filename: string;
    type: 'cv' | 'proposal' | 'document';
    url: string;
  }[];

  // Tracking
  status: 'draft' | 'queued' | 'sent' | 'delivered' | 'opened' | 'clicked' | 'replied' | 'bounced';
  sentAt?: Date;
  openedAt?: Date;
  clickedAt?: Date;
  repliedAt?: Date;

  // Auto-cleanup
  expiresAt: Date;           // Auto-delete if no reply (2 years)

  // Metadata
  createdAt: Date;
}
```

### 3. User Profile & Progress

**Purpose:** Track user's growth and provide personalized recommendations.

**Profile Features:**
- Skills inventory
- Progress tracking
- Learning recommendations
- Course links (edX, YouTube, Coursera)
- Achievement system
- Career goals

**User Profile Structure:**
```typescript
interface UserProfile {
  userId: string;

  // Basic Info
  fullName: string;
  headline: string;          // e.g., "Full Stack Developer"
  bio: string;

  // Skills
  skills: {
    name: string;
    level: 'beginner' | 'intermediate' | 'advanced' | 'expert';
    endorsements: number;
    lastUsed?: Date;
  }[];

  // Progress Tracking
  progress: {
    leadsContacted: number;
    emailsSent: number;
    repliesReceived: number;
    meetingsBooked: number;
    dealsWon: number;
    totalRevenue: number;
  };

  // Learning Path
  recommendedSkills: {
    skill: string;
    reason: string;          // Why this skill is recommended
    priority: 'high' | 'medium' | 'low';
    resources: {
      title: string;
      type: 'video' | 'course' | 'article' | 'book';
      platform: 'youtube' | 'edx' | 'coursera' | 'udemy' | 'other';
      url: string;
      isFree: boolean;
      duration?: string;
    }[];
  }[];

  // Career Goals
  goals: {
    title: string;
    targetDate?: Date;
    progress: number;        // 0-100
    milestones: string[];
  }[];

  // Documents
  cvs: {
    id: string;
    name: string;
    url: string;
    isDefault: boolean;
    lastUsed?: Date;
  }[];

  // Settings
  preferences: {
    emailSignature: string;
    defaultTone: 'formal' | 'professional' | 'casual' | 'friendly';
    industries: string[];    // Preferred industries
    jobTypes: string[];      // Preferred job types
  };
}
```

### 4. AI Engine

**Purpose:** Power intelligent features with Claude and OpenAI.

**Claude API (Quality/Creativity):**
- Professional email writing
- CV improvement suggestions
- Cover letter generation
- Proposal writing
- Template creation
- Personalization at scale

**OpenAI API (Speed/Logic):**
- Lead scoring
- Opportunity analysis (pros/cons)
- Skills gap analysis
- Quick summaries
- Intent classification
- Match reasoning

**AI Analysis Output:**
```typescript
interface LeadAnalysis {
  leadId: string;

  // Opportunity Assessment
  matchScore: number;        // 0-100

  pros: {
    point: string;
    reasoning: string;
  }[];

  cons: {
    point: string;
    mitigation: string;      // How to handle this
  }[];

  opportunities: {
    description: string;
    howToLeverage: string;
    potentialValue: string;
  }[];

  // Skills Analysis
  skillsMatch: {
    skill: string;
    userLevel: string;
    requiredLevel: string;
    gap: boolean;
  }[];

  // Recommended Actions
  nextSteps: string[];

  // Talking Points (for calls)
  talkingPoints: string[];
  questionsToAsk: string[];

  // Generated at
  analyzedAt: Date;
  model: string;             // Which AI model was used
}
```

### 5. Voice Capture & Real-time AI

**Purpose:** Provide AI assistance during live calls.

**Features:**
- Real-time voice capture
- Speaker separation
- Live transcription
- AI-powered suggestions
- Context injection from lead data
- Post-call summary

**Integration with Outreach:**
- When user gets a reply → notify app
- When call scheduled → prepare context
- During call → inject lead analysis
- After call → update lead status, generate follow-up

---

## User Features

### Dashboard Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  OUTREACH DASHBOARD                                    [Profile]│
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │ Leads: 247   │ │ Emails: 89   │ │ Replies: 12  │            │
│  │ +23 this week│ │ 67% opened   │ │ 13% rate     │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ RECENT EMAILS                                                ││
│  │ ┌─────────────────────────────────────────────────────────┐ ││
│  │ │ To: John Smith @ Acme Corp                    ⚡ Opened │ ││
│  │ │ Subject: Partnership Opportunity                        │ ││
│  │ │ Why sent: High match score (87%), needs our service     │ ││
│  │ │ Pros: Growing company, decision maker, tech-forward     │ ││
│  │ │ Cons: Busy Q4, might have existing vendor               │ ││
│  │ │ AI Insight: Could help with scaling challenges          │ ││
│  │ └─────────────────────────────────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ YOUR PROGRESS                              [View Full]       ││
│  │                                                              ││
│  │ Skills to Develop:                                          ││
│  │ • Negotiation (High Priority)                               ││
│  │   └─ Free Course: edX - Art of Negotiation                 ││
│  │ • Data Analysis (Medium Priority)                           ││
│  │   └─ YouTube: Data Analysis for Beginners                  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Email Sent View

For each sent email, user sees:

1. **Recipient Info:**
   - Name, title, company
   - LinkedIn profile
   - Contact history

2. **Why This Email Was Sent:**
   - Purpose/goal
   - Campaign it belongs to
   - AI reasoning for targeting

3. **Lead Analysis:**
   - Pros (why this lead is valuable)
   - Cons (challenges to prepare for)
   - Opportunity assessment

4. **AI Insights:**
   - How this person could help user
   - Skills match/gap
   - Potential collaboration areas
   - Recommended talking points

5. **Tracking:**
   - Sent/delivered/opened/clicked
   - Time of interactions
   - Reply notification

### Profile & Progress

User's profile shows:

1. **Skill Inventory:**
   - Current skills with levels
   - Skills gap analysis
   - Progress over time

2. **Learning Recommendations:**
   - Skills to develop (prioritized)
   - Free courses (edX, YouTube, Coursera)
   - Paid courses if preferred
   - Estimated time to complete

3. **Career Progress:**
   - Goals tracking
   - Milestones achieved
   - Performance metrics

4. **Documents:**
   - Multiple CVs for different purposes
   - Cover letter templates
   - Proposals

---

## Data Flow

### Email Sending Flow

```
User creates email
       │
       ▼
┌─────────────────┐
│ AI Enhancement  │ ← Claude generates/improves content
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Lead Analysis   │ ← OpenAI analyzes pros/cons/opportunities
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Email Queued    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ SES Sends Email │ ← AWS SES with tracking pixels
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Track Events    │ ← Opens, clicks, bounces
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Reply Detection │ ← Notify user, trigger call prep
└─────────────────┘
```

### Reply → Call Flow

```
Reply detected
       │
       ▼
┌─────────────────┐
│ Notify User     │ ← Push notification / email
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ User schedules  │
│ call with lead  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Desktop App     │ ← Prepare lead context
│ prepares data   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ During Call     │ ← Real-time AI hints with lead context
│ Voice AI active │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Post-Call       │ ← Summary, next steps, update CRM
└─────────────────┘
```

---

## Deployment Info

### Current Status

| Component | Status | URL |
|-----------|--------|-----|
| API | ✅ Live | outreachapi.hekax.com |
| Web | ✅ Live | outreach.hekax.com |
| Desktop | ✅ Built | Local .exe |
| Scraper | 🔄 Pending | Railway |
| Email Service | 🔄 Building | AWS SES configured |

### Environment Variables

**API (.env):**
```
DATABASE_URL=postgresql://...
JWT_SECRET=...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
AWS_REGION=us-east-1
SES_FROM_EMAIL=support@hekax.com
OPENAI_API_KEY=...
ANTHROPIC_API_KEY=...  # For Claude
```

---

## Next Steps

See [PLANS.md](./PLANS.md) for detailed implementation roadmap.

---

*This documentation is a living document. Update as features are built.*
