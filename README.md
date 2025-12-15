# Outreach

Voice capture, info scraping, CV processing, and email automation platform.

## Architecture

```
Outreach/
├── apps/
│   ├── desktop/        # Rust/Dioxus - Desktop voice capture app
│   ├── api/            # Rust/Axum - Backend API
│   └── web/            # Next.js - Web dashboard
├── workers/
│   └── scraper/        # Python/FastAPI - Scraping & automation worker
└── packages/
    └── shared/         # Shared types (future)
```

## Deployment

| App | Platform | URL |
|-----|----------|-----|
| `apps/api` | Railway | api.outreach.io |
| `apps/web` | Vercel | app.outreach.io |
| `workers/scraper` | Railway | (internal) |
| `apps/desktop` | Local | .exe installer |

## Getting Started

### Prerequisites
- Rust 1.75+
- Node.js 20+
- Python 3.11+
- pnpm (recommended)

### Development

```bash
# Clone the repo
git clone https://github.com/AhmediHarhash/Outreach.git
cd Outreach

# Install web dependencies
cd apps/web && npm install && cd ../..

# Install Python worker dependencies
cd workers/scraper && pip install -e . && cd ../..

# Run desktop app (dev)
cd apps/desktop && cargo run

# Run API (dev)
cd apps/api && cargo run

# Run web (dev)
cd apps/web && npm run dev

# Run scraper worker (dev)
cd workers/scraper && uvicorn hekax_scraper.main:app --reload
```

## Features

- [ ] Voice Capture (WASAPI, Deepgram STT)
- [ ] Real-time AI Assistance (Claude, GPT-4, Gemini)
- [ ] LinkedIn Scraping
- [ ] CV/Resume Processing
- [ ] Email Automation
- [ ] Lead Management
- [ ] Analytics Dashboard

## License

MIT
