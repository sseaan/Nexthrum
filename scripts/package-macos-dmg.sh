#!/usr/bin/env bash
# Build an ad-hoc signed macOS DMG without an Apple Developer account.
#
# Usage: scripts/package-macos-dmg.sh [version]
#
# What this does:
#   1. builds the app in Release with `CODE_SIGN_IDENTITY=-` (ad-hoc signature)
#   2. verifies the signature and the nested Rust dylib signature
#   3. packs a DMG with an /Applications shortcut
#
# What this cannot do without a paid Apple Developer account:
#   - Developer ID signing (Gatekeeper "unidentified developer" instead)
#   - notarization/stapling (`spctl` will reject the downloaded DMG)
# Recipients must right-click -> Open, or run:
#   xattr -dr com.apple.quarantine /Applications/Nexthrum.app
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
if [[ ! -d "$DEVELOPER_DIR/Platforms/MacOSX.platform" ]]; then
  echo "Xcode toolchain not found at: $DEVELOPER_DIR" >&2
  exit 1
fi

VERSION="${1:-$(node -p "require('./apps/windows/package.json').version")}"
OUT="$ROOT/target/macos-dmg"
APP="$OUT/Build/Products/Release/NexthrumMac.app"
DMG="$OUT/Nexthrum-${VERSION}-arm64.dmg"

rm -rf "$OUT"
mkdir -p "$OUT"

LOCAL_IDENTITY="Nexthrum Local Signing"
# Prefer the stable self-signed identity when it exists: it keeps the Keychain
# ACL and TCC grants across rebuilds. Fall back to ad-hoc signing otherwise.
if [[ -z "${SIGN_IDENTITY:-}" ]] && security find-identity -v -p codesigning 2>/dev/null | grep -Fq "$LOCAL_IDENTITY"; then
  SIGN_IDENTITY="$LOCAL_IDENTITY"
fi
SIGN_IDENTITY="${SIGN_IDENTITY:--}"

if [[ "$SIGN_IDENTITY" == "-" ]]; then
  echo "==> Building Release app (ad-hoc signed; Keychain grants are lost on every rebuild)"
  echo "    Run scripts/create-local-signing-identity.sh for a stable identity."
else
  echo "==> Building Release app (signed with: $SIGN_IDENTITY)"
fi
xcodebuild \
  -project apps/macos/NexthrumMac.xcodeproj \
  -scheme NexthrumMac \
  -configuration Release \
  -sdk macosx \
  -derivedDataPath "$OUT" \
  CODE_SIGNING_ALLOWED=YES \
  CODE_SIGN_IDENTITY="$SIGN_IDENTITY" \
  CODE_SIGN_STYLE=Manual \
  DEVELOPMENT_TEAM="" \
  OTHER_CODE_SIGN_FLAGS="--timestamp=none" \
  build >/dev/null

if [[ ! -d "$APP" ]]; then
  echo "Release app not found: $APP" >&2
  exit 1
fi

echo "==> Verifying signatures"
codesign --verify --deep --strict "$APP" || {
  echo "Signature verification failed for $APP" >&2
  exit 1
}
codesign -dv --verbose=2 "$APP" 2>&1 | grep -E "Signature|Identifier|Format" || true
DYLD="$APP/Contents/Frameworks/libnexthrum_ffi.dylib"
if [[ -f "$DYLD" ]]; then
  codesign --verify --strict "$DYLD"
  echo "nested dylib signature ok"
else
  echo "expected nested dylib missing: $DYLD" >&2
  exit 1
fi

echo "==> Building DMG"
STAGE="$OUT/stage"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "Nexthrum" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"

echo
echo "Created: $DMG"
echo "Size:    $(du -h "$DMG" | cut -f1)"
echo
echo "Distribution notes (no Apple Developer account):"
echo "  - The app is ad-hoc signed, NOT notarized. Gatekeeper will warn recipients."
echo "  - Install: open the DMG, drag Nexthrum to Applications, then right-click -> Open."
echo "  - Or clear the quarantine flag: xattr -dr com.apple.quarantine /Applications/NexthrumMac.app"
echo "  - For notarized builds see docs/release.md (requires a paid Developer ID)."
