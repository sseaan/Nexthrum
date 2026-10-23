#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
if [[ ! -d "$DEVELOPER_DIR/Platforms/MacOSX.platform" ]]; then
  echo "Xcode toolchain not found at: $DEVELOPER_DIR" >&2
  exit 1
fi

SDK_PATH="$(xcrun --sdk macosx --show-sdk-path)"
OUT="${DERIVED_DATA_PATH:-$ROOT/target/xcode-derived}"

xcodebuild \
  -project apps/macos/NexthrumMac.xcodeproj \
  -scheme NexthrumMac \
  -configuration Debug \
  -sdk macosx \
  -derivedDataPath "$OUT" \
  CODE_SIGNING_ALLOWED=NO \
  ALWAYS_SEARCH_USER_PATHS=NO \
  SDKROOT="$SDK_PATH" \
  build

APP="$OUT/Build/Products/Debug/NexthrumMac.app"
EXECUTABLE="$APP/Contents/MacOS/NexthrumMac"
EMBEDDED_FFI="$APP/Contents/Frameworks/libnexthrum_ffi.dylib"

if [[ ! -x "$EXECUTABLE" ]]; then
  echo "Xcode app executable missing: $EXECUTABLE" >&2
  exit 1
fi
if [[ ! -f "$EMBEDDED_FFI" ]]; then
  echo "Embedded Rust FFI library missing: $EMBEDDED_FFI" >&2
  exit 1
fi
if [[ "$(lipo -archs "$EXECUTABLE")" != "arm64" ]]; then
  echo "Expected an arm64-only app executable" >&2
  exit 1
fi
if ! otool -L "$EXECUTABLE" | grep -Fq '@rpath/libnexthrum_ffi.dylib'; then
  echo "App executable is not linked against the embedded Rust FFI library" >&2
  exit 1
fi
ICON="$APP/Contents/Resources/AppIcon.icns"
if [[ ! -f "$ICON" ]]; then
  echo "App icon missing from the bundle: $ICON" >&2
  exit 1
fi
plutil -lint "$APP/Contents/Info.plist"
printf 'Built and validated arm64 Xcode app: %s\n' "$APP"
