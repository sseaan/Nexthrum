import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import { Button } from "../ui/Button";
import { Field, Input } from "../ui/Input";
import { AlertIcon, BellIcon, CheckIcon } from "../ui/icons";
import { cx } from "../../utils/cx";
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
      setShowToken(true);
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
      // The host reports "saved but not connected" the same way as settings.
      if (message.includes("Settings were saved, but")) {
        setSettings(form);
      }
      setError(message);
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
      <div className="mx-auto w-full max-w-xl space-y-5 px-5 py-6">
        <header className="space-y-2">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)]">
            <BellIcon size={20} />
          </div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">
            {t.onboarding.welcome}
          </h1>
          <p className="text-sm leading-relaxed text-[var(--color-text-secondary)]">
            {t.onboarding.intro}
          </p>
        </header>

        <ol className="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-card)]">
          {steps.map((step, index) => (
            <li key={step.title} className="flex items-start gap-2.5">
              <span
                className="mt-0.5 inline-flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full bg-[var(--color-primary-soft)] text-[11px] font-bold text-[var(--color-primary)]"
                aria-hidden="true"
              >
                {index + 1}
              </span>
              <div className="min-w-0 space-y-0.5">
                <p className="text-sm font-semibold text-[var(--color-text)]">
                  {step.title}
                </p>
                <p className="text-xs leading-relaxed text-[var(--color-text-secondary)]">
                  {step.detail}
                </p>
              </div>
            </li>
          ))}
        </ol>

        <div className="space-y-3">
          <Field label={t.onboarding.serverUrl} htmlFor="onboarding-server-url">
            <Input
              id="onboarding-server-url"
              type="text"
              value={form.server_url}
              onChange={(e) =>
                setForm((prev) => ({ ...prev, server_url: e.target.value }))
              }
              placeholder={t.onboarding.serverUrlPlaceholder}
              spellCheck={false}
              autoComplete="url"
              autoFocus
            />
          </Field>

          {showToken ? (
            <Field label={t.onboarding.clientToken} htmlFor="onboarding-client-token">
              <Input
                id="onboarding-client-token"
                type="password"
                value={form.client_token}
                onChange={(e) =>
                  setForm((prev) => ({ ...prev, client_token: e.target.value }))
                }
                placeholder={t.onboarding.tokenPlaceholder}
                autoComplete="off"
              />
            </Field>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setShowToken(true)}
              className="-ml-0.5"
            >
              {t.onboarding.showToken}
            </Button>
          )}

          {testResult && (
            <div
              role="status"
              className={cx(
                "flex items-start gap-2 rounded-lg border px-2.5 py-1.5 text-xs",
                testSucceeded
                  ? "border-[var(--color-success)]/25 bg-[var(--color-success-soft)] text-[var(--color-success)]"
                  : "border-[var(--color-danger)]/25 bg-[var(--color-danger-soft)] text-[var(--color-danger)]",
              )}
            >
              <span className="mt-px flex-shrink-0">
                {testSucceeded ? <CheckIcon size={13} /> : <AlertIcon size={13} />}
              </span>
              <span className="break-words">{testResult}</span>
            </div>
          )}

          {error && (
            <div
              role="alert"
              className="flex items-start gap-2 rounded-lg border border-[var(--color-danger)]/25 bg-[var(--color-danger-soft)] px-2.5 py-1.5 text-xs text-[var(--color-danger)]"
            >
              <span className="mt-px flex-shrink-0">
                <AlertIcon size={13} />
              </span>
              <span className="break-words">{error}</span>
            </div>
          )}
        </div>

        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="primary" onClick={handleSave} disabled={saving}>
              {saving ? t.settings.saving : t.onboarding.save}
            </Button>
            <Button variant="secondary" onClick={handleTest} disabled={testing}>
              {testing ? t.onboarding.testing : t.onboarding.test}
            </Button>
          </div>
          <div className="flex flex-wrap items-center justify-end gap-1.5">
            <Button variant="ghost" size="sm" onClick={onSkip}>
              {t.onboarding.skip}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setCurrentView("settings")}
            >
              {t.onboarding.advanced}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
