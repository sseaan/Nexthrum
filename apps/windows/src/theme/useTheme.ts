import { create } from "zustand";

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "nexthrum.theme";

const MODES: ThemeMode[] = ["system", "light", "dark"];

function isThemeMode(value: string | null): value is ThemeMode {
  return value !== null && (MODES as string[]).includes(value);
}

function systemPrefersDark(): boolean {
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch {
    // Older runtimes may not implement matchMedia; dark is the baseline skin.
    return true;
  }
}

function readStoredMode(): ThemeMode {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (isThemeMode(stored)) return stored;
  } catch {
    // Storage can be unavailable; fall back to following the system.
  }
  return "system";
}

/** Resolve a user preference into the theme that is actually painted. */
export function resolveTheme(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") return systemPrefersDark() ? "dark" : "light";
  return mode;
}

/** Write the resolved theme to the document so CSS tokens can switch. */
export function applyTheme(mode: ThemeMode): ResolvedTheme {
  const resolved = resolveTheme(mode);
  const root = document.documentElement;
  root.dataset.theme = resolved;
  root.style.colorScheme = resolved;
  return resolved;
}

interface ThemeState {
  mode: ThemeMode;
  /** Theme currently painted, after resolving "system". */
  resolved: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}

const initialMode = readStoredMode();

export const useTheme = create<ThemeState>((set) => ({
  mode: initialMode,
  resolved: resolveTheme(initialMode),
  setMode: (mode) => {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    } catch {
      // Persisting the preference is best-effort.
    }
    set({ mode, resolved: applyTheme(mode) });
  },
}));

/**
 * Paint the stored theme and keep "system" in sync with OS changes.
 * Called once from main.tsx before the first render.
 */
export function initTheme(): void {
  applyTheme(useTheme.getState().mode);
  useTheme.setState({ resolved: resolveTheme(useTheme.getState().mode) });

  try {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    media.addEventListener("change", () => {
      if (useTheme.getState().mode !== "system") return;
      const resolved = applyTheme("system");
      useTheme.setState({ resolved });
    });
  } catch {
    // Without matchMedia the theme simply stays on the initial resolution.
  }
}
