# Nexthrum macOS app

The macOS app uses SwiftUI with a shared Rust bridge. Settings exposes Server URL, Client Token, retention days (1–365), DND start/end, notification permission, and connection status. If the Keychain token is absent, locally stored messages still load and remain available, while the Gotify connection stays stopped.

`CredentialSettingsTransaction` does not treat a successful settings commit followed by a connection-start failure as a rollback: it keeps the persisted settings/token and reports the connection error separately. XCTest covers settings-save rollback, rollback failure reporting, credential-write failure (settings are not attempted), and successful commit.

## Generated bridge

From the repository root:

```bash
scripts/generate-swift-bindings.sh
```

The generated files are in `Generated/`:

- `nexthrum_ffi.swift`
- `nexthrum_ffiFFI.h`
- `nexthrum_ffiFFI.modulemap`

## Xcode project build

A shared `Nexthrum.xcodeproj` and `Nexthrum` scheme now build the app target. Run:

```bash
scripts/build-macos-xcode.sh
```

The shared `Nexthrum.xcconfig` is the single source for target architecture and Swift/linker settings. The Xcode build phase builds the Rust FFI crate and regenerates UniFFI bindings; the wrapper validates the resulting arm64 executable, embedded dylib, `@rpath`, and Info.plist. Output is `target/xcode-derived/Build/Products/Debug/Nexthrum.app`.

The Xcode project intentionally targets Apple Silicon arm64 only; Intel/x86_64 and universal binaries are out of scope. The current deployment target is macOS 14.0. `Sparkle.framework` (2.10.0) is vendored under `Vendor/` and drives automatic updates from the release `appcast.xml`.

Builds are signed with the stable self-signed identity (`scripts/create-local-signing-identity.sh`); Gatekeeper still warns on first install because the app is not notarized.

The app validates settings through `BridgeService`, tests the Gotify `/version` endpoint through `nexthrum-gotify`, and exposes a cancellable WebSocket lifecycle. UniFFI exposes `startConnection`, non-blocking `pollEvent`, and `stopConnection`; Swift polls on the main run loop and updates connection state and the in-memory message list. Tokio runtime/channels remain inside Rust and are not passed across FFI.

Notification delivery uses `MacNotificationPolicy` to require both Core's `should_notify` decision and an authorized/provisional/ephemeral system status. Notification authorization and click-to-message-ID parsing have an Xcode unit test target; run it with:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project apps/macos/Nexthrum.xcodeproj -scheme Nexthrum -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test
```

Actual notification delivery requires granting permission in macOS System Settings.

## Updates

Sparkle 2 drives automatic updates from the release `appcast.xml`. `CFBundleVersion`
is a monotonically increasing build number (currently `2000`); bump it for every
release. `scripts/generate-swift-bindings.sh` and the Xcode build read the same
`Vendor/Sparkle.framework`, so no package resolution is needed on CI.

## Localisation

UI strings live in `Resources/Localizable.xcstrings` (en + zh-Hans). Static SwiftUI
literals are used as keys; dynamic strings go through `L10n.t(key)`. `LocalizationTests`
fails if the two languages drift apart.

The macOS app uses a native SwiftUI `MenuBarExtra` window panel: unread count, the three most recent messages (open / mark read / delete), Open Messages, Settings, and Quit. Closing the last main window keeps the app and Gotify connection running; quitting is explicit from the menu. Clicking a notification reopens the main window and navigates to its message.

The macOS app uses `SMAppService.mainApp` for Launch at login. Settings shows the stored preference and actual system status, including when approval is needed in System Settings. Registration errors are reported independently and do not undo server settings or Keychain credentials.
