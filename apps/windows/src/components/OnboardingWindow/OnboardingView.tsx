import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import type { AppSettings } from "../../types";

/**
 * First-run guide. Shown while no server is configured so a fresh install
 * explains what to enter and where the token comes from.
 */
export function OnboardingView({ onSkip }: { onSkip: () => void }) {
  const { t } = useI18n();
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);
  const setCurrentView = useAppStore((s) => s.setCurrentView);

  const [form, setForm] = useState<AppSettings>({ ...settings });
  const [showToken, setShowToken] = useState(false);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [testSucceeded, setTestSucceeded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canSubmit = form.server_url.trim() !== "" && form.client_token.trim() !== "";

  const handleTest = async () => {
    setShowToken(true);
    if (!canSubmit) {
      setError(t.onboarding.missing);
      return;
    }
    setError(null);
    setTesting(true);
    setTestResult(null);
    try {
      const result = await invoke<string>("test_connection", {
        serverUrl: form.server_url,
        clientToken: form.client_token,
      });
      setTestResult(result);
      setTestSucceeded(true);
    } catch (e) {
      setTestResult(String(e));
      setTestSucceeded(false);
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    if (!canSubmit) {
      setError(t.onboarding.missing);
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await invoke("save_settings", { settings: form });
      setSettings(form);
      // App.tsx picks the message view up from the saved settings.
    } catch (e) {
      const message = String(e);
      if (message.includes("Settings were saved, but")) {
        setSettings(form);
        setError(message);
      } else {
        setError(message);
      }
    } finally {
      setSaving(false);
    }
  };

  const steps = [
    { title: t.onboarding.step1Title, detail: t.onboarding.step1Detail },
    { title: t.onboarding.step2Title, detail: t.onboarding.step2Detail },
    { title: t.onboarding.step3Title, detail: t.onboarding.step3Detail },
  ];

  return (
    <div className="h-full overflow-y-auto bg-[var(--color-bg)]">
      <div className="mx-auto w-full max-w-[560px] px-6 py-8 space-y-5">
        <div className="space-y-2">
          <div className="text-3xl" aria-hidden="true">🔔</div>
          <h1 className="text-xl font-semibold text-[var(--color-text)]">
            {t.onboarding.welcome}
          </h1>
          <p className="text-sm text-[var(--color-text-secondary)] leading-relaxed">
            {t.onboarding.intro}
          </p>
        </div>

        <ol className="space-y-2.5 rounded-xl bg-[var(--color-surface)] p-4">
          {steps.map((step, index) => (
            <li key={step.title} className="flex items-start gap-2.5">
              <span
                className="mt-0.5 w-[18px] h-[18px] rounded-full bg-[var(--color-primary)]/15 text-[var(--color-primary)] text-[11px] font-bold inline-flex items-center justify-center flex-shrink-0"
                aria-hidden="true"
              >
                {index + 1}
              </span>
              <div className="space-y-0.5">
                <p className="text-sm font-semibold text-[var(--color-text)]">{step.title}</p>
                <p className="text-xs text-[var(--color-text-secondary)] leading-relaxed">
                  {step.detail}
                </p>
              </div>
            </li>
          ))}
        </ol>

        <div className="space-y-2.5">
          <input
            type="text"
            value={form.server_url}
            onChange={(e) => setForm((prev) => ({ ...prev, server_url: e.target.value }))}
            placeholder={t.onboarding.serverUrlPlaceholder}
            aria-label={t.onboarding.serverUrl}
            className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] placeholder-[var(--color-text-secondary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
          />

          {showToken ? (
            <input
              type="password"
              value={form.client_token}
              onChange={(e) => setForm((prev) => ({ ...prev, client_token: e.target.value }))}
              placeholder={t.onboarding.tokenPlaceholder}
              aria-label={t.onboarding.clientToken}
              className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] placeholder-[var(--color-text-secondary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
            />
          ) : (
            <button
              onClick={() => setShowToken(true)}
              className="text-xs text-[var(--color-primary)] hover:underline"
            >
              {t.onboarding.showToken}
            </button>
          )}

          {testResult && (
            <p
              className={`text-xs flex items-start gap-1.5 ${
                testSucceeded ? "text-[var(--color-success)]" : "text-[var(--color-danger)]"
              }`}
            >
              <span aria-hidden="true">{testSucceeded ? "✓" : "✕"}</span>
              <span className="break-words">{testResult}</span>
            </p>
          )}
          {error && <p className="text-xs text-[var(--color-danger)] break-words">{error}</p>}
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            onClick={handleTest}
            disabled={testing}
            className="px-3 py-1.5 text-sm rounded-md border border-[var(--color-border)] text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] disabled:opacity-50 transition-colors"
          >
            {testing ? t.onboarding.testing : t.onboarding.test}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-3 py-1.5 text-sm rounded-md bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] disabled:opacity-50 transition-colors"
          >
            {t.onboarding.save}
          </button>
          <div className="flex-1" />
          <button onClick={onSkip} className="text-xs text-[var(--color-text-secondary)] hover:underline">
            {t.onboarding.skip}
          </button>
          <button
            onClick={() => setCurrentView("settings")}
            className="text-xs text-[var(--color-text-secondary)] hover:underline"
          >
            {t.onboarding.advanced}
          </button>
        </div>
      </div>
    </div>
  );
}
