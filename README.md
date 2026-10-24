<div align="center">

<img alt="Nexthrum" src="./docs/images/logo.png" width="140">

# Nexthrum

**Gotify desktop client for Windows and macOS — one Rust core, two native frontends.**

<img src="https://img.shields.io/badge/-English-155EEF?style=for-the-badge" alt="English"> <a title="简体中文" href="./README.zh.md"><img src="https://img.shields.io/badge/-简体中文-545759?style=for-the-badge" alt="简体中文"></a>

[![Release](https://img.shields.io/github/v/release/sseaan/Nexthrum?logo=github&logoColor=white&labelColor=24292f&color=155eef)](https://github.com/sseaan/Nexthrum/releases)
[![License](https://img.shields.io/github/license/sseaan/Nexthrum?labelColor=7d89b0&color=5d6b98)](./LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/sseaan/Nexthrum/ci.yml?logo=github&logoColor=white&label=CI&labelColor=32b583&color=12b76a)](https://github.com/sseaan/Nexthrum/actions/workflows/ci.yml)
[![Release build](https://img.shields.io/github/actions/workflow/status/sseaan/Nexthrum/build.yml?logo=github&logoColor=white&label=release&labelColor=7a5af8&color=9b8afb)](https://github.com/sseaan/Nexthrum/actions/workflows/build.yml)
<br>
![Platforms](https://img.shields.io/badge/Windows-10%20%7C%2011-5cadf6?logo=windows&logoColor=white&labelColor=2496ed&color=1570ef)
![macOS](https://img.shields.io/badge/macOS-14%2B%20arm64-000000?logo=apple&logoColor=white&labelColor=545759&color=1d1d1f)
![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white&labelColor=1d1d1f&color=24c8db)
![Rust](https://img.shields.io/badge/Rust-shared%20core-000000?logo=rust&logoColor=white&labelColor=545759&color=000000)
![React](https://img.shields.io/badge/React-18-61dafb?logo=react&logoColor=white&labelColor=1d1d1f&color=61dafb)
![SwiftUI](https://img.shields.io/badge/SwiftUI-native-f05138?logo=swift&logoColor=white&labelColor=1d1d1f&color=f05138)
[![Last commit](https://img.shields.io/github/last-commit/sseaan/Nexthrum?logo=github&logoColor=white&labelColor=8098f9&color=444ce7)](https://github.com/sseaan/Nexthrum/commits/main)

</div>

> A desktop client for your self-hosted [Gotify](https://gotify.net/) server: messages arrive over WebSocket, land in a local SQLite history, and surface as native notifications and tray/menu-bar badges — with no telemetry and no third-party service.

Nexthrum is built twice on purpose: Windows uses Tauri 2 + React, macOS uses SwiftUI, and both call the **same Rust crates** for message rules, storage and the Gotify protocol. Fix a message rule once and both platforms get it.

**Great fit if you want to:**
- read and search your Gotify history on the desktop instead of the web UI
- get native notifications with a real do-not-disturb window
- keep the client token in the OS credential vault rather than a config file
- run a small, auditable client that only talks to your own server

**Probably not for you if you need:**
- a mobile app (there is none)
- a hosted or multi-account service
- a generic notification hub for services other than Gotify

<div align="center">

### Screenshots

<table>
  <tr>
    <td align="center" width="50%"><img src="./docs/images/macos-messages.png" alt="macOS message list" width="420"><br><sub>macOS · message list</sub></td>
    <td align="center" width="50%"><img src="./docs/images/windows-messages.png" alt="Windows message list" width="420"><br><sub>Windows · message list</sub></td>
  </tr>
  <tr>
    <td align="center"><img src="./docs/images/macos-settings.png" alt="macOS settings" width="420"><br><sub>macOS · settings</sub></td>
    <td align="center"><img src="./docs/images/windows-settings.png" alt="Windows settings" width="420"><br><sub>Windows · settings</sub></td>
  </tr>
</table>

</div>

---

<details>
<summary><strong>Table of contents</strong></summary>

- [Features](#features)
- [Install](#install)
- [Getting started](#getting-started)
- [Build from source](#build-from-source)
- [Architecture](#architecture)
- [Data and privacy](#data-and-privacy)
- [Documentation](#documentation)
- [Version history](#version-history)
- [License](#license)

</details>

---

## Features

<details open>
<summary><strong>Messaging</strong></summary>

- **Real-time stream** — Gotify WebSocket client with automatic reconnect and backoff, cancel-on-stop and a 10 s handshake timeout, so the UI never hangs on "connecting".
- **Local history** — SQLite with configurable retention (1–365 days, default 7). Re-delivered messages keep their local read state and never re-notify.
- **Flat, readable list** — rows separated by hairlines instead of nested cards; unread state uses three redundant cues (dot, weight, accent-coloured time) plus source and priority chips.
- **Search** — debounced, case-insensitive match over title and body; the global unread count stays untouched while filtering.
- **Source filter** — appears once messages come from two or more Gotify applications; names come from `/application`, falling back to `App <id>`.
</details>

<details>
<summary><strong>Desktop integration</strong></summary>

- **Native notifications** — Windows toast notifications and macOS `UserNotifications`, both gated by the shared do-not-disturb window.
- **Always-on entry points** — Windows tray icon with unread badge and tooltip; macOS menu bar panel with the three most recent messages and quick actions.
- **Window behaviour you expect** — Windows uses a custom caption bar (logo, product name, minimise/maximise/close, double-click to maximise); macOS keeps the native toolbar and can hide its Dock icon to live in the menu bar only.
- **Start at login** — Windows autostart entry and macOS `SMAppService`, with the real system approval state shown in Settings.
</details>

<details>
<summary><strong>Appearance, security and updates</strong></summary>

- **Light and dark** — one set of semantic tokens for both themes; Windows offers follow-system / light / dark in Settings, macOS follows the system appearance.
- **Localized UI** — English and Simplified Chinese; Windows has an in-app switch, macOS follows the system language (switchable in Settings, applied after relaunch).
- **Secure credentials** — the client token never touches the settings JSON: Windows Credential Manager and macOS Keychain, with idempotent migration of old plaintext tokens.
- **Signed updates** — Windows uses `tauri-plugin-updater` with a minisign-signed `latest.json`; macOS uses Sparkle with an EdDSA-signed `appcast.xml`. Both are visible in Settings, including an "automatically check" toggle on macOS.
</details>

---

## Install

Download from [GitHub Releases](https://github.com/sseaan/Nexthrum/releases).

### Windows

| File | Description |
|---|---|
| `Nexthrum_2.0.1_x64-setup.exe` | NSIS installer (per-user, no admin prompt) |
| `Nexthrum_2.0.1_x64-portable.zip` | Portable build — extract and run |

The installer is **not** Authenticode-signed, so SmartScreen shows "Windows protected your PC" on first run: choose **More info → Run anyway**.

### macOS

| File | Description |
|---|---|
| `Nexthrum-2.0.1-arm64.dmg` | Drag-and-drop installer (Apple Silicon, macOS 14+) |
| `Nexthrum-2.0.1.zip` | App archive used by Sparkle for updates |

The app is **not notarized** (no paid Apple Developer account), so the first launch needs **right-click → Open**, or:

```bash
xattr -dr com.apple.quarantine /Applications/Nexthrum.app
```

Updates installed by Sparkle carry no quarantine flag, so later upgrades no longer prompt.

---

## Getting started

1. Launch Nexthrum — the window opens and a tray icon / menu bar item appears.
2. Open **Settings** and enter your **Gotify server URL** (for example `https://gotify.example.com`) plus a **client token** (Gotify web UI → *Clients*).
3. Press **Test connection**, then **Save** — the WebSocket stream starts automatically.
4. Send a message from Gotify: it appears in the list and raises a notification unless do-not-disturb is active.

Clicking a row copies its body to the clipboard and marks it read; right-click offers *Mark as read* and *Delete*.

---

## Build from source

### Prerequisites

| Platform | Requirements |
|---|---|
| Windows | Node.js 20+, Rust stable (MSVC toolchain), Visual Studio C++ Build Tools, WebView2 |
| macOS | Node.js 20+, Rust stable, Xcode 26+ (arm64) |
| Both | Git, with `cargo` and `npm` on `PATH` |

### Windows client

```bash
git clone https://github.com/sseaan/Nexthrum.git
cd Nexthrum

npm --prefix apps/windows ci
npm --prefix apps/windows run tauri dev     # development with hot reload
npm --prefix apps/windows run tauri build   # NSIS installer + updater artifacts
```

The dev server needs a fixed port that matches `build.devUrl`; Windows reserves ranges for Hyper-V, so run `powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1` if Vite reports the port as taken.

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

macOS builds are signed with a **stable self-signed identity** created by `scripts/create-local-signing-identity.sh`. It keeps the code signature (and therefore Keychain grants) stable between rebuilds; it does not replace notarization.

> Build against a current SDK. Apps linked against an old SDK fall back to the legacy interface design on macOS 26/27, which makes them look different from other native apps. Releases are built on a `macos-26` runner; verify with `DTSDKName` in the built app's `Info.plist`.

### Checks

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/windows run typecheck
npm --prefix apps/windows test
npm --prefix apps/windows run build
npm --prefix apps/windows run check:tauri-versions
```

CI (`.github/workflows/ci.yml`) runs the same gates on Windows and macOS arm64, plus UniFFI generation, the Xcode build and XCTest.

---

## Architecture

```
apps/windows (React)  ──Tauri──┐
                               ├── nexthrum-core ── nexthrum-storage (SQLite)
apps/macos (SwiftUI) ──UniFFI──┘        │            nexthrum-gotify (HTTP/WS)
                                        └── host adapters: tray, notifications,
                                            credentials, window lifecycle, updates
```

- `nexthrum-core` owns every business rule: message intake, deduplication, read state, DND, retention. It has no UI, Tauri or platform-path dependencies.
- `nexthrum-gotify` owns the HTTP API and the WebSocket client, including reconnect/backoff, cancellation, typed errors and the rustls crypto-provider setup.
- `nexthrum-storage` owns SQLite migrations, queries and settings I/O; the host injects the paths.
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
scripts/              Bindings, icons, macOS build/package, signing identity, dev port
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
- Tokens live in the platform credential vault, never in `settings.json`. Logs are redacted.
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
| [`docs/release.md`](docs/release.md) | Version sources, signing, updates, test builds, checklist |
| [`docs/development-guide.md`](docs/development-guide.md) | Environment setup and workflow |
| [`docs/windows-testing.md`](docs/windows-testing.md) | Real-machine test checklist for Windows |
| [`docs/history.md`](docs/history.md) | Version history and archived planning records |
| [`AGENTS.md`](AGENTS.md) | Contribution conventions for automated agents |

---

## Version history

| Version | Date | Highlights |
|---|---|---|
| **2.0.1** | 2026-10-24 | Windows UI rebuilt (portrait window, flat macOS-like language, theme toggle, clearer row hierarchy), macOS Dock-icon/language preferences, update controls in Settings, dependency and release-tooling fixes |
| 2.0.0 | 2026-10-23 | First cross-platform release on the shared Rust core: card list, search, source filter, tray badge, menu bar panel, credentials in the OS vault, per-platform auto-update |
| 1.0.1 | 2026-07-02 | Windows-only maintenance release |
| 1.0.0 | 2026-07-02 | Windows-only first release |

Details and the retired planning documents are archived in [`docs/history.md`](docs/history.md).

---

## License

Copyright © 2026 Sean.

Nexthrum is free software licensed under the **GNU Affero General Public License v3.0** (AGPL-3.0). See [LICENSE](LICENSE) for the full text.

You may use, study, modify and redistribute it under the terms of that license. Because AGPL-3.0 also covers software offered over a network, if you deploy a modified Nexthrum as a network service you must offer the corresponding source to its users. There is **no warranty**, as stated in the license.
