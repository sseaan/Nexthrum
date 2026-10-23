import { create } from "zustand";
import zh from "./zh-CN";
import en from "./en-US";

type DeepString<T> = {
  [K in keyof T]: T[K] extends string
    ? string
    : T[K] extends object
      ? DeepString<T[K]>
      : never;
};

type Translations = DeepString<typeof zh>;

const locales: Record<string, Translations> = {
  "zh-CN": zh as Translations,
  "en-US": en as Translations,
};

interface I18nState {
  locale: string;
  setLocale: (locale: string) => void;
  t: Translations;
}

/**
 * Returns the translation object for the current locale.
 * Usage: const { t } = useI18n(); →  t.message.empty
 */
const LOCALE_STORAGE_KEY = "nexthrum.locale";

/** Best guess before the user picks a language: follow the system, defaulting to Chinese. */
function detectLocale(): string {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored && locales[stored]) return stored;
  } catch {
    // Storage can be unavailable (private mode); fall through to detection.
  }
  const system = navigator.language?.toLowerCase() ?? "";
  if (system.startsWith("zh")) return "zh-CN";
  if (system.startsWith("en")) return "en-US";
  return "zh-CN";
}

const initialLocale = detectLocale();

export const useI18n = create<I18nState>(() => ({
  locale: initialLocale,
  setLocale: (_locale: string) => {},
  t: locales[initialLocale] ?? zh,
}));

// Initialize immediately from the persisted/system locale.
useI18n.setState({
  locale: initialLocale,
  t: locales[initialLocale] ?? zh,
  setLocale: (locale: string) => {
    const resolved = locales[locale] ? locale : "zh-CN";
    try {
      localStorage.setItem(LOCALE_STORAGE_KEY, resolved);
    } catch {
      // Persisting the preference is best-effort.
    }
    useI18n.setState({
      locale: resolved,
      t: locales[resolved] ?? zh,
    });
  },
});

/** Simple string interpolation: replace {key} with values */
export function fmt(
  template: string,
  values: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (_, key) =>
    String(values[key] ?? `{${key}}`),
  );
}
