//! Polished UI Styles
//!
//! Beautiful, S-tier quality CSS with animations and color-coded outputs.

/// Main application CSS - polished and beautiful
pub const POLISHED_CSS: &str = r##"
/* ============================================
   VOICE COPILOT - S-TIER POLISHED UI
   ============================================ */

/* CSS Reset and Root Variables */
:root {
    /* Dark Theme (Default) */
    --bg-primary: #0d1117;
    --bg-secondary: #161b22;
    --bg-tertiary: #21262d;
    --bg-hover: #30363d;
    --bg-glass: rgba(22, 27, 34, 0.85);

    --text-primary: #f0f6fc;
    --text-secondary: #8b949e;
    --text-muted: #484f58;

    --accent-blue: #58a6ff;
    --accent-green: #3fb950;
    --accent-yellow: #d29922;
    --accent-orange: #db6d28;
    --accent-red: #f85149;
    --accent-purple: #a371f7;
    --accent-cyan: #39c5cf;
    --accent-pink: #db61a2;

    /* Semantic Colors */
    --color-transcript: #8b949e;
    --color-flash: #58a6ff;
    --color-deep: #a371f7;
    --color-question: #39c5cf;
    --color-objection: #f85149;
    --color-buying-signal: #3fb950;
    --color-technical: #db6d28;
    --color-warning: #d29922;
    --color-success: #3fb950;

    --border-color: #30363d;
    --border-focus: #58a6ff;
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
    --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.5);
    --shadow-glow: 0 0 20px rgba(88, 166, 255, 0.15);

    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 16px;
    --radius-full: 9999px;

    --font-mono: 'SF Mono', 'Consolas', 'Monaco', monospace;
    --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;

    --transition-fast: 0.15s ease;
    --transition-normal: 0.25s ease;
    --transition-slow: 0.4s ease;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: var(--font-sans);
    font-size: 14px;
    line-height: 1.5;
    color: var(--text-primary);
    background: var(--bg-primary);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow: hidden;
    user-select: none;
}

/* ============================================
   MAIN CONTAINER
   ============================================ */

.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: linear-gradient(180deg, var(--bg-primary) 0%, #0a0d12 100%);
}

/* ============================================
   HEADER BAR
   ============================================ */

.header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    -webkit-app-region: drag;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 12px;
}

.logo {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 15px;
}

.logo-icon {
    width: 24px;
    height: 24px;
    background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
}

.mode-badge {
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-radius: var(--radius-full);
    background: var(--accent-blue);
    color: white;
}

.mode-badge.sales { background: var(--accent-green); }
.mode-badge.interview { background: var(--accent-purple); }
.mode-badge.technical { background: var(--accent-orange); }

.header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
}

.header-btn {
    padding: 6px 10px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all var(--transition-fast);
}

.header-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--accent-blue);
}

.header-btn.active {
    background: var(--accent-blue);
    color: white;
    border-color: var(--accent-blue);
}

/* ============================================
   MAIN CONTENT AREA
   ============================================ */

.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    gap: 16px;
    overflow-y: auto;
}

/* ============================================
   TRANSCRIPT SECTION
   ============================================ */

.section {
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    overflow: hidden;
    animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

.section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
}

.section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.section-title .icon {
    font-size: 14px;
}

.section-title.transcript { color: var(--color-transcript); }
.section-title.flash { color: var(--color-flash); }
.section-title.deep { color: var(--color-deep); }
.section-title.question { color: var(--color-question); }

.section-badge {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
}

.section-content {
    padding: 16px;
}

/* ============================================
   TRANSCRIPT BOX
   ============================================ */

.transcript-box {
    font-size: 15px;
    line-height: 1.6;
    color: var(--text-primary);
    min-height: 40px;
}

.transcript-box .highlight {
    color: var(--accent-cyan);
    font-weight: 500;
}

.transcript-empty {
    color: var(--text-muted);
    font-style: italic;
}

/* ============================================
   FLASH BULLETS - QUICK SUGGESTIONS
   ============================================ */

.flash-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.flash-summary {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px;
    background: rgba(88, 166, 255, 0.1);
    border-radius: var(--radius-md);
    border-left: 3px solid var(--color-flash);
}

.flash-type {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    border-radius: var(--radius-full);
    white-space: nowrap;
}

.flash-type.question { background: rgba(57, 197, 207, 0.2); color: var(--color-question); }
.flash-type.objection { background: rgba(248, 81, 73, 0.2); color: var(--color-objection); }
.flash-type.buying_signal { background: rgba(63, 185, 80, 0.2); color: var(--color-buying-signal); }
.flash-type.technical { background: rgba(219, 109, 40, 0.2); color: var(--color-technical); }
.flash-type.statement { background: rgba(139, 148, 158, 0.2); color: var(--text-secondary); }
.flash-type.small_talk { background: rgba(72, 79, 88, 0.2); color: var(--text-muted); }

.flash-summary-text {
    flex: 1;
    font-size: 14px;
    color: var(--text-primary);
}

.bullets-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.bullet-item {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border-left: 3px solid transparent;
    transition: all var(--transition-fast);
    cursor: pointer;
}

.bullet-item:hover {
    background: var(--bg-hover);
    border-left-color: var(--accent-blue);
}

.bullet-priority {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    font-size: 11px;
    font-weight: 700;
    border-radius: var(--radius-full);
    flex-shrink: 0;
}

.bullet-priority.p1 { background: var(--accent-blue); color: white; }
.bullet-priority.p2 { background: var(--accent-purple); color: white; }
.bullet-priority.p3 { background: var(--bg-hover); color: var(--text-secondary); }
.bullet-priority.p4 { background: var(--bg-hover); color: var(--text-muted); }

.bullet-text {
    flex: 1;
    font-size: 14px;
    line-height: 1.5;
}

.bullet-item.starred {
    border-left-color: var(--accent-yellow);
}

.bullet-item.starred .bullet-priority {
    background: var(--accent-yellow);
}

/* ============================================
   DEEP RESPONSE - DETAILED CONTENT
   ============================================ */

.deep-container {
    position: relative;
}

.deep-content {
    font-size: 14px;
    line-height: 1.7;
    color: var(--text-primary);
    white-space: pre-wrap;
}

.deep-content .highlight {
    color: var(--accent-purple);
    font-weight: 500;
}

.deep-streaming {
    position: relative;
}

.deep-streaming::after {
    content: '';
    display: inline-block;
    width: 2px;
    height: 16px;
    background: var(--accent-purple);
    margin-left: 4px;
    animation: blink 1s infinite;
}

@keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
}

/* ============================================
   QUESTION TO ASK
   ============================================ */

.question-box {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 16px;
    background: linear-gradient(135deg, rgba(57, 197, 207, 0.15), rgba(57, 197, 207, 0.05));
    border-radius: var(--radius-md);
    border: 1px solid rgba(57, 197, 207, 0.3);
}

.question-icon {
    font-size: 18px;
    flex-shrink: 0;
}

.question-text {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-question);
    line-height: 1.5;
}

/* ============================================
   STATUS INDICATORS
   ============================================ */

.status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
}

.status-left {
    display: flex;
    align-items: center;
    gap: 12px;
}

.status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
}

.status-dot.listening {
    background: var(--accent-green);
    animation: pulse 2s infinite;
}

.status-dot.processing {
    background: var(--accent-blue);
    animation: pulse 1s infinite;
}

.status-dot.error {
    background: var(--accent-red);
}

@keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.6; transform: scale(1.1); }
}

.status-right {
    display: flex;
    align-items: center;
    gap: 16px;
}

.ai-provider {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-muted);
}

.ai-provider.local { color: var(--accent-green); }
.ai-provider.cloud { color: var(--accent-blue); }

/* ============================================
   URGENCY INDICATOR
   ============================================ */

.urgency-bar {
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 600;
    text-align: center;
    animation: slideDown 0.3s ease;
}

@keyframes slideDown {
    from { opacity: 0; transform: translateY(-100%); }
    to { opacity: 1; transform: translateY(0); }
}

.urgency-bar.answer_now {
    background: linear-gradient(90deg, var(--accent-red), #ff6b6b);
    color: white;
}

.urgency-bar.can_elaborate {
    background: linear-gradient(90deg, var(--accent-yellow), #ffc107);
    color: #1a1a1a;
}

.urgency-bar.just_listening {
    background: linear-gradient(90deg, var(--accent-green), #4caf50);
    color: white;
}

/* ============================================
   OVERLAY MODE STYLES
   ============================================ */

.overlay-mode {
    background: var(--bg-glass);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-radius: var(--radius-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: var(--shadow-lg), var(--shadow-glow);
}

.overlay-mode .section {
    background: rgba(22, 27, 34, 0.6);
    border-color: rgba(255, 255, 255, 0.1);
}

/* ============================================
   COMPACT MODE
   ============================================ */

.compact-mode .section-header {
    padding: 8px 12px;
}

.compact-mode .section-content {
    padding: 10px 12px;
}

.compact-mode .bullet-item {
    padding: 6px 10px;
}

.compact-mode .bullet-priority {
    width: 18px;
    height: 18px;
    font-size: 10px;
}

/* ============================================
   ANIMATIONS
   ============================================ */

.fade-in {
    animation: fadeIn 0.3s ease;
}

.slide-up {
    animation: slideUp 0.3s ease;
}

@keyframes slideUp {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
}

.bounce-in {
    animation: bounceIn 0.4s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

@keyframes bounceIn {
    from { opacity: 0; transform: scale(0.8); }
    to { opacity: 1; transform: scale(1); }
}

/* ============================================
   SCROLLBAR
   ============================================ */

::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: var(--bg-primary);
}

::-webkit-scrollbar-thumb {
    background: var(--bg-hover);
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: var(--border-color);
}

/* ============================================
   STEALTH MODE INDICATOR
   ============================================ */

.stealth-indicator {
    position: fixed;
    top: 8px;
    right: 8px;
    padding: 4px 8px;
    font-size: 10px;
    background: rgba(248, 81, 73, 0.2);
    color: var(--accent-red);
    border-radius: var(--radius-sm);
    opacity: 0.6;
    pointer-events: none;
}

/* ============================================
   HOTKEY HINTS
   ============================================ */

.hotkey-hint {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    background: var(--bg-tertiary);
    border-radius: 4px;
    color: var(--text-muted);
}
"##;

/// Get the full CSS including theme overrides
pub fn get_themed_css(theme_name: &str) -> String {
    let theme_vars = match theme_name {
        "light" => LIGHT_THEME_VARS,
        "high_contrast" => HIGH_CONTRAST_VARS,
        "cyberpunk" => CYBERPUNK_VARS,
        _ => "", // Dark is default
    };

    format!("{}\n\n{}", theme_vars, POLISHED_CSS)
}

const LIGHT_THEME_VARS: &str = r#"
:root {
    --bg-primary: #ffffff;
    --bg-secondary: #f6f8fa;
    --bg-tertiary: #eaeef2;
    --bg-hover: #d0d7de;
    --bg-glass: rgba(246, 248, 250, 0.9);
    --text-primary: #1f2328;
    --text-secondary: #656d76;
    --text-muted: #8c959f;
    --accent-blue: #0969da;
    --accent-green: #1a7f37;
    --accent-yellow: #9a6700;
    --accent-orange: #bc4c00;
    --accent-red: #cf222e;
    --accent-purple: #8250df;
    --accent-cyan: #0969da;
    --accent-pink: #bf3989;
    --border-color: #d0d7de;
}
"#;

const HIGH_CONTRAST_VARS: &str = r#"
:root {
    --bg-primary: #000000;
    --bg-secondary: #0a0a0a;
    --bg-tertiary: #141414;
    --text-primary: #ffffff;
    --accent-blue: #00d4ff;
    --accent-green: #00ff7f;
    --accent-red: #ff4444;
}
"#;

const CYBERPUNK_VARS: &str = r#"
:root {
    --bg-primary: #0a0a12;
    --bg-secondary: #12121f;
    --bg-tertiary: #1a1a2e;
    --accent-blue: #00f0ff;
    --accent-green: #00ff9f;
    --accent-red: #ff0055;
    --accent-purple: #bf00ff;
}
"#;
