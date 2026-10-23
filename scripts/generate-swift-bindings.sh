#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO_BIN="${CARGO_BIN:-${HOME}/.cargo/bin/cargo}"

"$CARGO_BIN" build -p nexthrum-ffi
mkdir -p apps/macos/Generated
"$CARGO_BIN" run -p uniffi-bindgen-nexthrum -- generate \
  --library target/debug/libnexthrum_ffi.dylib \
  --language swift \
  --out-dir apps/macos/Generated
