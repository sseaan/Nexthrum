#!/usr/bin/env bash
# Regenerate Windows (Tauri) and macOS (Xcode) app icons from the master image.
#
# Usage: scripts/generate-icons.sh [source.png]
# Default source: ./Nexthrum.png (square PNG, 8-bit RGB/RGBA, ideally >= 1024px).
#
# The master artwork is assumed to carry its own transparent padding, so each
# platform is re-padded to its own convention instead of sharing one margin:
#   Windows app/taskbar icon : 92% of the canvas
#   Windows tray icon        : 94% (legibility at 16px)
#   macOS AppIcon            : 80.5% (Apple's 824/1024 icon grid)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOURCE="${1:-$ROOT/Nexthrum.png}"
if [[ ! -f "$SOURCE" ]]; then
  echo "Icon source not found: $SOURCE" >&2
  exit 1
fi

read -r WIDTH HEIGHT <<<"$(sips -g pixelWidth -g pixelHeight "$SOURCE" | awk '/pixel/ {print $2}' | paste -sd' ' -)"
if [[ "$WIDTH" != "$HEIGHT" ]]; then
  echo "Icon source must be square, got ${WIDTH}x${HEIGHT}" >&2
  exit 1
fi
if (( WIDTH < 1024 )); then
  echo "Icon source should be at least 1024x1024, got ${WIDTH}" >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

normalize() { # canvas content-ratio output
  python3 "$ROOT/scripts/icon_normalize.py" "$SOURCE" "$3" --canvas "$1" --content "$2"
}

echo "==> Windows icons (apps/windows/src-tauri/icons)"
normalize 1024 0.92 "$WORK/windows-master.png"
(
  cd apps/windows
  # Mobile assets and Microsoft Store logos are not shipped by this desktop app.
  npx tauri icon "$WORK/windows-master.png" -o src-tauri/icons >/dev/null
  rm -rf src-tauri/icons/android src-tauri/icons/ios
  rm -f src-tauri/icons/Square*Logo.png src-tauri/icons/StoreLogo.png
  rm -f src-tauri/icons/64x64.png src-tauri/icons/icon.png
)
normalize 32 0.94 apps/windows/src-tauri/icons/tray-icon.png
ls -1 apps/windows/src-tauri/icons

echo "==> macOS icon (apps/macos/AppIcon.icns)"
normalize 1024 0.805 "$WORK/macos-master.png"
ICONSET="$WORK/AppIcon.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$WORK/macos-master.png" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" "$WORK/macos-master.png" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o apps/macos/AppIcon.icns
ls -l apps/macos/AppIcon.icns

echo "Icons regenerated. Rebuild both apps to verify."
