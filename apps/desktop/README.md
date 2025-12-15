# Voice Copilot

A real-time AI assistant that listens to your voice calls and provides intelligent suggestions as you speak.

## Features

- **Universal Audio Capture** - Works with any app (Zoom, Discord, Meet, Teams, etc.)
- **Real-time Transcription** - Deepgram Nova-2 or OpenAI Realtime API
- **Cascading AI Pipeline**:
  - **Stage 1 (Capture)**: < 300ms - Audio to text
  - **Stage 2 (Flash)**: < 500ms - Quick bullet points (Gemini 2.0 Flash)
  - **Stage 3 (Deep)**: 1-3s streaming - Detailed response (Claude 3.5 Sonnet)
- **Context Modes**: Sales, Interview, Technical
- **Native Desktop App**: Rust + Dioxus (lightweight, fast)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    YOUR DESKTOP                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   Any Voice App (Zoom/Discord/Meet/Teams)               │
│                    │                                    │
│          ┌─────────▼─────────┐                          │
│          │  System Audio     │                          │
│          │  Capture (WASAPI) │                          │
│          └─────────┬─────────┘                          │
│                    │                                    │
│     ┌──────────────┼──────────────┐                    │
│     ▼              ▼              ▼                     │
│ ┌───────┐    ┌─────────┐    ┌──────────┐              │
│ │ STT   │───▶│ Flash   │───▶│ Deep     │              │
│ │Deepgram│    │ Gemini  │    │ Claude   │              │
│ │<300ms │    │ <500ms  │    │ streaming│              │
│ └───────┘    └─────────┘    └──────────┘              │
│                    │                                    │
│          ┌─────────▼─────────┐                          │
│          │  Floating Overlay │                          │
│          │  (Dioxus UI)      │                          │
│          └───────────────────┘                          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## What You See

```
┌─────────────────────────────────────┐
│ 🎤 "How much does your enterprise   │
│    plan cost?"                      │
├─────────────────────────────────────┤
│ ⚡ QUICK:                            │
│ "Pricing question - enterprise tier"│
│                                     │
│ 📌 MENTION:                          │
│ ★ Ask what budget they had in mind  │
│ • Mention value before price        │
│ • Reference ROI metrics             │
├─────────────────────────────────────┤
│ 🧠 DETAILED:                         │
│ "Before I give you specific numbers,│
│ I'd love to understand your needs   │
│ better. Our enterprise plan is      │
│ designed for teams of 50+, and      │
│ typically our customers see a 3x    │
│ ROI within the first quarter..."    │
│                                     │
│ 🔄 ASK THEM:                         │
│ "What's driving your evaluation     │
│ right now?"                         │
└─────────────────────────────────────┘
```

## Prerequisites

- **Rust** (1.75+)
- **API Keys**:
  - Deepgram (for STT) - [deepgram.com](https://deepgram.com)
  - Google AI (for Gemini Flash) - [ai.google.dev](https://ai.google.dev)
  - Anthropic (for Claude) - [anthropic.com](https://anthropic.com)
  - OR OpenAI (alternative for all) - [openai.com](https://openai.com)

## Installation

### 1. Clone and Build

```bash
git clone https://github.com/AhmediHarhash/voice-copilot.git
cd voice-copilot

# Build release
cargo build --release

# The binary will be at target/release/voice-copilot
```

### 2. Configure API Keys

On first launch, click the ⚙️ Settings button and enter your API keys.
Keys are stored securely in your OS keychain.

### 3. Run

```bash
./target/release/voice-copilot
```

Or just double-click the executable.

## Usage

1. **Start the app** - Small window appears
2. **Select your mode** - Sales, Interview, or Technical
3. **Join your call** - Zoom, Discord, Meet, anything
4. **Click "Start Listening"** - System audio capture begins
5. **Talk naturally** - AI suggestions appear in real-time
6. **Use the suggestions** - Quick bullets first, details stream in

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+S` | Start/Stop listening |
| `Ctrl+Shift+H` | Hide/Show window |
| `Ctrl+Shift+M` | Switch mode |
| `Ctrl+Shift+C` | Copy last suggestion |

## Project Structure

```
voice-copilot/
├── src/
│   ├── main.rs              # Entry point
│   ├── capture/             # Audio capture & STT
│   │   ├── audio.rs         # System audio (WASAPI)
│   │   ├── deepgram.rs      # Deepgram streaming
│   │   └── transcript.rs    # Transcript buffer
│   ├── flash/               # Stage 2 - Quick responses
│   │   ├── gemini.rs        # Gemini 2.0 Flash
│   │   └── bullet_extractor.rs
│   ├── deep/                # Stage 3 - Detailed responses
│   │   ├── claude.rs        # Claude 3.5 Sonnet
│   │   ├── gpt4o.rs         # GPT-4o
│   │   └── router.rs        # Model selection
│   ├── brain/               # Orchestration
│   │   ├── pipeline.rs      # Main pipeline
│   │   ├── context.rs       # Conversation tracking
│   │   └── modes/           # Sales/Interview/Technical
│   ├── ui/                  # Dioxus UI
│   │   ├── app.rs           # Main window
│   │   └── components/      # UI components
│   └── config/              # Settings
├── prompts/                 # AI prompt templates
└── Cargo.toml
```

## Configuration

Settings are stored at:
- **Windows**: `%APPDATA%\voice-copilot\settings.json`
- **macOS**: `~/Library/Application Support/voice-copilot/settings.json`
- **Linux**: `~/.config/voice-copilot/settings.json`

API keys are stored securely in your OS keychain.

## Model Selection

| Stage | Default | Alternative |
|-------|---------|-------------|
| STT | Deepgram Nova-2 | OpenAI Realtime |
| Flash | Gemini 2.0 Flash | GPT-4o-mini |
| Deep | Claude 3.5 Sonnet | GPT-4o, o1-preview |

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows | ✅ Full | WASAPI loopback for system audio |
| macOS | ⚠️ Partial | Requires BlackHole for system audio |
| Linux | ⚠️ Partial | Requires PulseAudio/PipeWire config |

## Privacy

- All audio is processed via API (Deepgram/OpenAI) - not stored locally
- Transcripts are kept in memory only during the session
- API keys are stored in your OS secure keychain
- No data is sent anywhere except the configured AI providers

## Future Plans

- [ ] Local Whisper support (fully offline STT)
- [ ] Voice output (AI speaks for you - OpenAI Realtime)
- [ ] Custom prompt editor
- [ ] Conversation analytics
- [ ] Team features

## License

MIT

## Support

For issues or feature requests, open a GitHub issue.

---

Built with Rust, Dioxus, and the power of AI.
