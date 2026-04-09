# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MFA Manager — a local-first desktop TOTP authenticator app prototype. The project has three components:

1. **UI prototype** (`index.html`) — A single-file HTML/CSS/JS mockup of the MFA Manager desktop app. No build tools, no framework. Open directly in a browser.
2. **Slint UI** (`slint_ui/`) — A Rust+Slint port of the HTML prototype. Uses `slint 1.9` and Rust edition 2024. Mirrors the same layout and demo data but is missing the theme switcher and add/edit account sheet.
3. **Multi-agent design system** (`agents.py`) — A Python script using the Anthropic SDK to run a 4-agent pipeline (Architect → Reviewer → Coder → Tester) that designs and implements the core TOTP engine in Rust.

## Running

- **UI prototype**: Open `index.html` in any browser. No server needed.
- **Slint UI**: `cd slint_ui && cargo run` (debug) or `cargo run --release` (release). Build only: `cargo build` / `cargo build --release`.
- **Agent pipeline**: `python3 agents.py` — requires `anthropic` pip package and `ANTHROPIC_API_KEY` env var. Uses `claude-opus-4-6` with adaptive thinking. Produces `design.md`, `review.md`, `code.md`, `tests.md` artifacts.
- **Color scheme reference**: `colors.html` — standalone page showing all 7 theme palettes (a–g) with design tokens.

## Architecture

### index.html — UI Prototype

Self-contained single-file app with no external dependencies beyond Google Fonts (Inter, Roboto Mono, Material Icons Round).

- **Theming**: 7 color schemes (a–g) via `data-theme` attribute on `.app`. CSS custom properties define all tokens. Theme switcher at bottom of the page.
- **TOTP display**: Demo OTP codes in `OTP_CODES` array, 30-second countdown timers with circular SVG progress rings. Codes auto-mask after 8s reveal. Copy-to-clipboard with auto-clear after 25s.
- **Lock screen**: 6-digit PIN entry with keypad, attempt counting, shake animation on error. Demo PIN is `123456`, auto-entered on load.
- **Layout**: Fixed 400×720px app shell (macOS-style traffic lights, search bar, card list, bottom sheet for adding entries, snackbar notifications).

### agents.py — Multi-Agent Pipeline

4-phase sequential pipeline, each phase is a streaming call to Claude via `client.messages.stream()`:

1. **Architect** — designs Rust module structure, data structures, traits, dependencies, security model
2. **Reviewer** — audits the design for security/correctness, outputs P0/P1/P2 issues
3. **Coder** — implements the TOTP engine based on reviewed design
4. **Tester** — writes RFC 4226 test vectors, property tests, security tests, integration tests

Agent prompts are in Chinese and target Rust implementation with: AES-256-GCM encryption, Argon2id key derivation, `zeroize` for memory wiping, `thiserror` for error types.

### slint_ui/ — Slint Desktop UI

Rust + Slint 1.9 port of the HTML prototype. The build script (`build.rs`) compiles `ui/app.slint` via `slint_build::compile()`.

- `ui/app.slint` (505 lines) defines the full UI: Glacier Blue theme, account data model, component library (TrafficLights, TimerRing, CopyChip, TotpCard, PasswordCard, SearchBar, BottomNav, LockScreen), and the MainWindow with hardcoded demo accounts.
- `src/main.rs` is a minimal entry point — `slint::include_modules!()` + `MainWindow::new()?.run()`.
- Uses the "G" Glacier Blue palette, while `colors.html` recommends "C" Carbon Dark as the optimal scheme.
- Currently lacks the theme switcher and add/edit account bottom sheet from the HTML prototype.

### colors.html — Theme Reference

Standalone palette showcase. Not connected to the app — purely a design tool for exploring color scheme options.

## Key Technical Details

- TOTP timers use a 30-second period (RFC 6238 standard)
- SVG circular progress rings track remaining time (`strokeDashoffset` animation)
- Only one card can show a revealed code at a time (auto-closes others)
- Clipboard is auto-cleared 25s after copy (security feature in prototype)
- The agent pipeline outputs go to `/home/oak/mfa_manager/` (hardcoded path in `agents.py:231`)
