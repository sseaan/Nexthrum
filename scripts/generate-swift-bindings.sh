#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Resolve cargo instead of assuming ~/.cargo: CI images may install rustup
# elsewhere (e.g. /opt/homebrew/opt/rustup/bin) and Xcode trims PATH.
if [[ -z "${CARGO_BIN:-}" ]]; then
  CARGO_BIN="$(command -v cargo || true)"
fi
if [[ -z "${CARGO_BIN:-}" || ! -x "$CARGO_BIN" ]]; then
  for candidate in \
    "$HOME/.cargo/bin/cargo" \
    /usr/local/cargo/bin/cargo \
    /opt/homebrew/opt/rustup/bin/cargo \
    /opt/homebrew/bin/cargo \
    /usr/local/bin/cargo; do
    if [[ -x "$candidate" ]]; then CARGO_BIN="$candidate"; break; fi
  done
fi
if [[ -z "${CARGO_BIN:-}" || ! -x "$CARGO_BIN" ]]; then
  echo "error: cargo not found; install Rust or set CARGO_BIN" >&2
  exit 1
fi

"$CARGO_BIN" build -p nexthrum-ffi
mkdir -p apps/macos/Generated
"$CARGO_BIN" run -p uniffi-bindgen-nexthrum -- generate \
  --library target/debug/libnexthrum_ffi.dylib \
  --language swift \
  --out-dir apps/macos/Generated
