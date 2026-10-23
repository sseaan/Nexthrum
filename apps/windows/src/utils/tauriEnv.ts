/**
 * Runtime detection helpers.
 *
 * `@tauri-apps/api` reads `window.__TAURI_INTERNALS__`, which only exists inside
 * the Tauri webview. Opening the dev server in a plain browser is a legitimate
 * debugging path (see docs/windows-testing.md), so every Tauri API call must be
 * guarded instead of throwing on load.
 */
export function isTauriRuntime(): boolean {
  if (typeof window === "undefined") return false;
  const bridge = (window as unknown as { __TAURI_INTERNALS__?: unknown })
    .__TAURI_INTERNALS__;
  return typeof bridge !== "undefined";
}
