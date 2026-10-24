# Nexthrum

**English** · [简体中文](README.zh.md)

A cross-platform desktop client for [Gotify](https://gotify.net/): receive your self-hosted push messages as native notifications, with a shared Rust core behind both native frontends.

| Platform | UI | Status |
|---|---|---|
| Windows 10 1809+ / 11 | Tauri 2 + React 18 + TypeScript | Shipping |
| macOS 14+ (Apple Silicon) | SwiftUI + Sparkle | Shipping |

---

## Features

- **Real-time stream** — Gotify WebSocket client with automatic reconnect, cancel-on-stop and a 10 s handshake timeout, so the UI never hangs on "connecting".
- **Native notifications** — Windows toast notifications and macOS `UserNotifications`; both respect the shared do-not-disturb policy from the Rust core.
- **Local history** — SQLite with configurable retention (1–365 days, default 7). Duplicate deliveries keep the local read state and never re-notify.
- **Message list as cards** — unread state is shown with three redundant cues (accent bar, dot, weight), source and priority badges, and hover elevation.
- **Search** — case-insensitive match over title and body, debounced, with the global unread count preserved while filtering.
- **Source filter** — appears when messages come from two or more Gotify applications; names are resolved from `/application` and fall back to `App <id>`.
- **Always-on entry points** — Windows tray with unread badge and tooltip; macOS menu bar popover with the three most recent messages and quick actions.
- **Secure credentials** — the client token never touches the settings JSON: Windows Credential Manager and macOS Keychain, with idempotent migration of old plaintext tokens.
- **Automatic updates** — signed update channel per platform: Windows `tauri-plugin-updater`, macOS Sparkle (EdDSA-signed appcast).
- **Localized UI** — English and Simplified Chinese; Windows has an in-app switch, macOS follows the system language.
- **Launch at login** — Windows autostart entry and macOS `SMAppService`, with the real system approval state surfaced in Settings.

---

## Installation

Download from [GitHub Releases](https://github.com/sseaan/Nexthrum/releases).

### Windows

| File | Description |
|---|---|
| `Nexthrum_2.0.1_x64-setup.exe` | NSIS installer (per-user, no admin prompt) |
| `Nexthrum_2.0.1_x64-portable.zip` | Portable build — extract and run |

The installer is **not** Authenticode-signed yet, so SmartScreen shows "Windows protected your PC" on first run: choose **More info → Run anyway**.

### macOS

| File | Description |
|---|---|
| `Nexthrum-2.0.1-arm64.dmg` | Drag-and-drop installer (Apple Silicon, macOS 14+) |
| `Nexthrum-2.0.1.zip` | App archive used by Sparkle for updates |

The app is **not notarized** (no paid Apple Developer account), so the first launch needs **right-click → Open**, or:

```bash
xattr -dr com.apple.quarantine /Applications/Nexthrum.app
```

Upgrades delivered by Sparkle are installed without the quarantine flag, so later updates no longer prompt. If you are upgrading from an early 2.0 build, delete the old `NexthrumMac.app` first — the bundle id, settings and Keychain entry are unchanged.

---

## Getting started

1. Launch Nexthrum — the window opens and a tray icon / menu bar item appears.
2. Open **Settings** and enter your **Gotify server URL** (for example `https://gotify.example.com`) and a **client token** (Gotify web UI → *Clients*).
3. Press **Test connection** to check the HTTP endpoint, then **Save** — the WebSocket stream starts automatically.
4. Send a message from Gotify; it shows up in the list and raises a notification unless do-not-disturb is active.

---

## Build from source

### Prerequisites

| Platform | Requirements |
|---|---|
| Windows | Node.js 20+, Rust stable, Visual Studio C++ Build Tools, WebView2 |
| macOS | Node.js 20+, Rust stable, Xcode 15.4+ (arm64), macOS 14 SDK |
| Both | Git; `cargo`, `npm` on `PATH` |

### Windows client

```bash
git clone https://github.com/sseaan/Nexthrum.git
cd Nexthrum

npm --prefix apps/windows ci
npm --prefix apps/windows run tauri dev     # development with hot reload
npm --prefix apps/windows run tauri build   # NSIS installer + portable zip
```

### macOS client

```bash
scripts/generate-swift-bindings.sh          # UniFFI bindings for Swift
scripts/build-macos-xcode.sh                # builds and validates Nexthrum.app
scripts/package-macos-dmg.sh                # signed DMG for local distribution

# unit tests
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project apps/macos/Nexthrum.xcodeproj -scheme Nexthrum \
  -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test
```

macOS builds are signed with a **stable self-signed identity** created by `scripts/create-local-signing-identity.sh`. This keeps the code signature (and therefore Keychain grants) stable between rebuilds; it does not replace notarization.

### Checks

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/windows run typecheck
npm --prefix apps/windows test
```

CI (`.github/workflows/ci.yml`) runs the same checks on Windows and macOS arm64, plus UniFFI generation, the Xcode build and XCTest.

---

## Architecture

```
apps/windows (React)  ──Tauri──┐
                               ├── nexthrum-core ── nexthrum-storage (SQLite)
apps/macos (SwiftUI) ──UniFFI──┘        │            nexthrum-gotify (HTTP/WS)
                                        └── host adapters: tray, notifications,
                                            credentials, window lifecycle
```

- `nexthrum-core` owns every business rule: message intake, deduplication, read state, DND, retention. It has no UI, Tauri or platform-path dependencies.
- `nexthrum-gotify` owns the HTTP API and the WebSocket client, including reconnect/backoff, cancellation, typed errors and the rustls crypto-provider setup.
- `nexthrum-storage` owns SQLite schema migrations, queries and settings I/O; the host injects the paths.
- `nexthrum-ffi` exposes a stable UniFFI surface (`BridgeService`, DTOs, typed errors) for Swift.
- Host adapters stay thin: Tauri commands and the SwiftUI bridge only validate input, call core use cases and surface events.

The backend owns state: unread counts and message lists always come from an atomic snapshot, and mutation commands return the post-mutation snapshot.

### Repository layout

```
apps/
  windows/            Windows client (React + Vite) and Tauri host
  macos/              SwiftUI app, Xcode project, XCTest, Sparkle (Vendor/)
crates/
  nexthrum-core/      Domain models, use cases, DND and notification policy
  nexthrum-gotify/    Gotify HTTP/WebSocket client
  nexthrum-storage/   SQLite repository, migrations, settings
  nexthrum-ffi/       UniFFI bridge
scripts/              Bindings, icons, macOS build/package, signing identity
tools/uniffi-bindgen/ Local Swift bindings generator
docs/                 Requirements, architecture, design, tech spec, release runbook
devlog/               Daily development log
```

### Tech stack

| Layer | Windows | macOS |
|---|---|---|
| UI | React 18 + TypeScript + Tailwind CSS v4 + Zustand | SwiftUI |
| Host | Tauri 2 (Rust) | AppKit bridge + UniFFI |
| Core | Rust workspace (`nexthrum-core`, `nexthrum-gotify`, `nexthrum-storage`) | same crates |
| Storage | SQLite (rusqlite) | same repository |
| Streaming | tokio-tungstenite + rustls | same client |
| Notifications | `tauri-plugin-notification` | `UserNotifications` |
| Updates | `tauri-plugin-updater` (`latest.json`) | Sparkle (`appcast.xml`) |
| Credentials | Windows Credential Manager | Keychain |

---

## Data and privacy

- Nexthrum talks only to the Gotify server you configure; there is no telemetry and no third-party service.
- Tokens are stored in the platform credential vault, never in `settings.json`. Logs are redacted.
- Local data:
  - Windows: `%APPDATA%\Nexthrum\nexthrum_messages.db` and `nexthrum_data\settings.json`
  - macOS: `~/Library/Application Support/Nexthrum/`
- Deleting messages locally never touches the server unless you enable "sync deletion" in the clear dialog.

---

## Documentation

| Document | Contents |
|---|---|
| [`docs/requirements.md`](docs/requirements.md) | Product requirements and acceptance criteria |
| [`docs/architecture.md`](docs/architecture.md) | Module responsibilities, events, persistence, update channels |
| [`docs/tech-spec.md`](docs/tech-spec.md) | Layering rules, dependencies, quality gates |
| [`docs/design-spec.md`](docs/design-spec.md) | Visual language, layout, accessibility rules |
| [`docs/release.md`](docs/release.md) | Version sources, signing, notarization, updates, checklist |
| [`docs/development-guide.md`](docs/development-guide.md) | Environment setup and workflow |
| [`AGENTS.md`](AGENTS.md) | Contribution conventions for automated agents |

---

## License

Copyright © 2026 Sean.

Nexthrum is free software licensed under the **GNU Affero General Public License v3.0** (AGPL-3.0). See [LICENSE](LICENSE) for the full text.

You may use, study, modify and redistribute it under the terms of that license. Because AGPL-3.0 also covers software offered over a network, if you deploy a modified Nexthrum as a network service you must offer the corresponding source to its users. There is **no warranty**, as stated in the license.
