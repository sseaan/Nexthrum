#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
if [[ ! -d "$DEVELOPER_DIR/Platforms/MacOSX.platform" ]]; then
  echo "Xcode toolchain not found at: $DEVELOPER_DIR" >&2
  echo "Install Xcode and set DEVELOPER_DIR to its Contents/Developer directory." >&2
  exit 1
fi

scripts/generate-swift-bindings.sh
SDK_PATH="$(xcrun --sdk macosx --show-sdk-path)"
OUTPUT_DIR="${OUTPUT_DIR:-$ROOT/target/macos}"
APP_PATH="$OUTPUT_DIR/Nexthrum.app"
FRAMEWORKS="$APP_PATH/Contents/Frameworks"
MACOS="$APP_PATH/Contents/MacOS"

rm -rf "$APP_PATH"
mkdir -p "$FRAMEWORKS" "$MACOS" "$APP_PATH/Contents/Resources"
cp apps/macos/Info.plist "$APP_PATH/Contents/Info.plist"
cp target/debug/deps/libnexthrum_ffi.dylib "$FRAMEWORKS/libnexthrum_ffi.dylib"
install_name_tool -id @rpath/libnexthrum_ffi.dylib "$FRAMEWORKS/libnexthrum_ffi.dylib"

xcrun swiftc \
  -sdk "$SDK_PATH" \
  -I apps/macos/Generated \
  -Xcc -fmodule-map-file=apps/macos/Generated/nexthrum_ffiFFI.modulemap \
  apps/macos/Generated/nexthrum_ffi.swift \
  apps/macos/KeychainCredentialStore.swift \
  apps/macos/AppModel.swift \
  apps/macos/ContentView.swift \
  apps/macos/SettingsView.swift \
  apps/macos/NexthrumApp.swift \
  -L target/debug/deps -lnexthrum_ffi \
  -Xlinker -rpath -Xlinker @executable_path/../Frameworks \
  -framework SwiftUI -framework AppKit -framework Security \
  -o "$MACOS/Nexthrum"

install_name_tool \
  -change "$ROOT/target/debug/deps/libnexthrum_ffi.dylib" \
  @rpath/libnexthrum_ffi.dylib \
  "$MACOS/Nexthrum"

/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP_PATH/Contents/Info.plist" >/dev/null
codesign --force --deep --sign - "$APP_PATH"
echo "Built ad-hoc signed macOS app bundle: $APP_PATH"
