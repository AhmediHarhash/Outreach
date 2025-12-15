# Voice Copilot - Testing Guide

## Quick Start

1. **Build the app:**
   ```cmd
   cd C:\Users\PC\Desktop\voice-copilot
   cargo build --release
   ```

2. **Run it:**
   ```cmd
   target\release\voice-copilot.exe
   ```

## Features to Test

### 1. Audio Source Selection
- [ ] Click the audio source dropdown
- [ ] See "System Audio (All)" as default
- [ ] If Zoom/Discord/Teams is running, it should appear with "RUNNING" badge
- [ ] Click "Refresh" to detect newly opened apps
- [ ] Select an app to capture only that app's audio

### 2. Mode Selection
- [ ] Click Sales/Interview/Technical buttons
- [ ] Active mode should be highlighted blue

### 3. UI Modes
- [ ] Click "Full" - normal window
- [ ] Click "Overlay" - compact view
- [ ] Click "Mini" - minimal view

### 4. Start/Stop Listening
- [ ] Click "Start Listening"
- [ ] Status should change to "Listening" with green dot
- [ ] Click "Stop Listening" to stop

### 5. Settings Panel
- [ ] Click the ⚙️ button
- [ ] Settings panel should open
- [ ] API keys should be pre-filled from .env
- [ ] Click outside or 'x' to close

### 6. Keyboard Shortcuts
- [ ] `Ctrl+Shift+S` - Start/Stop listening
- [ ] `Ctrl+Shift+H` - Hide/Show window
- [ ] `Ctrl+Shift+M` - Switch mode
- [ ] `Ctrl+Shift+C` - Copy last suggestion

### 7. System Tray (if implemented)
- [ ] Icon appears in system tray
- [ ] Right-click shows menu
- [ ] Show/Hide window works
- [ ] Start/Stop listening works
- [ ] Mode selection works
- [ ] Quit exits the app

## API Keys Required

You have configured in `.env`:
- [x] DEEPGRAM_API_KEY - For speech-to-text
- [x] OPENAI_API_KEY - For GPT-4o responses

Optional:
- [ ] ANTHROPIC_API_KEY - For Claude responses (recommended)
- [ ] GOOGLE_AI_API_KEY - For Gemini Flash (fast responses)

## Troubleshooting

### Build Errors
If you get build errors, try:
```cmd
cargo clean
cargo build --release
```

### No Audio Capture
- Make sure you have audio output device selected
- On Windows, WASAPI loopback should work automatically
- Try selecting "System Audio (All)" instead of specific app

### API Errors
- Check your API keys in Settings
- Make sure you have internet connection
- Check the console for error messages

### Hotkeys Not Working
- Make sure no other app is using the same shortcuts
- Try restarting the app
- Check Windows focus

## Files Modified

- `src/ui/app.rs` - Main UI with all features
- `src/ui/runtime.rs` - Pipeline integration
- `src/ui/hotkeys.rs` - Global keyboard shortcuts
- `src/ui/tray.rs` - System tray
- `src/ui/settings.rs` - Settings panel
- `src/capture/app_audio.rs` - App-specific audio capture
- `src/main.rs` - .env loading
