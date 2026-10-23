import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import type { AppSettings } from "../../types";

export function SettingsWindow() {
  const { t } = useI18n();
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);
  const setCurrentView = useAppStore((s) => s.setCurrentView);

  const [form, setForm] = useState<AppSettings>({ ...settings });
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  useEffect(() => {
    setForm({ ...settings });
  }, [settings]);

  const handleSave = async () => {
    setSaving(true);
    setStatusMessage(null);
    try {
      await invoke("save_settings", { settings: form });
      setSettings(form);
      setStatusMessage(t.settings.saved);
      setTimeout(() => setStatusMessage(null), 3000);
    } catch (e) {
      const error = String(e);
      if (error.includes("Settings were saved, but")) {
        setSettings(form);
        setStatusMessage(error);
      } else {
        setStatusMessage(`Failed to save: ${error}`);
      }
    } finally {
      setSaving(false);
    }
  };

  const handleTestConnection = async () => {
    if (!form.server_url || !form.client_token) {
      setTestResult(t.settings.fillRequired);
      return;
    }

    setTesting(true);
    setTestResult(null);
    try {
      const result = await invoke("test_connection", {
        serverUrl: form.server_url,
        clientToken: form.client_token,
      });
      setTestResult(result as string);
      await invoke("save_settings", { settings: form });
      setSettings(form);
    } catch (e) {
      const error = String(e);
      if (error.includes("Settings were saved, but")) {
        setSettings(form);
        setTestResult(error);
      } else {
        setTestResult(`Connection failed: ${error}`);
      }
    } finally {
      setTesting(false);
    }
  };

  const handleBack = () => {
    setCurrentView("messages");
  };

  const updateField = <K extends keyof AppSettings>(
    field: K,
    value: AppSettings[K],
  ) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg)]">
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
        <div className="flex items-center gap-3">
          <button
            onClick={handleBack}
            className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
          >
            {t.settings.back}
          </button>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">
            {t.settings.title}
          </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-lg mx-auto space-y-6">
          <section>
            <h2 className="text-sm font-semibold text-[var(--color-text)] mb-3 uppercase tracking-wide">
              {t.settings.serverConnection}
            </h2>
            <div className="space-y-4">
              <div>
                <label className="block text-xs text-[var(--color-text-secondary)] mb-1">
                  {t.settings.serverUrl}
                </label>
                <input
                  type="text"
                  value={form.server_url}
                  onChange={(e) => updateField("server_url", e.target.value)}
                  placeholder="https://gotify.example.com"
                  className="w-full px-3 py-2 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] placeholder-[var(--color-text-secondary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                />
              </div>
              <div>
                <label className="block text-xs text-[var(--color-text-secondary)] mb-1">
                  {t.settings.clientToken}
                </label>
                <input
                  type="password"
                  value={form.client_token}
                  onChange={(e) => updateField("client_token", e.target.value)}
                  placeholder="Your Gotify client token"
                  className="w-full px-3 py-2 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] placeholder-[var(--color-text-secondary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                />
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleTestConnection}
                  disabled={testing}
                  className="px-4 py-2 text-sm rounded-md bg-[var(--color-surface-hover)] text-[var(--color-text)] hover:bg-[var(--color-border)] disabled:opacity-50 transition-colors"
                >
                  {testing ? t.settings.testing : t.settings.testConnection}
                </button>
              </div>
              {testResult && (
                <p
                  className={`text-xs ${
                    testResult.startsWith("Connected")
                      ? "text-[var(--color-success)]"
                      : "text-[var(--color-danger)]"
                  }`}
                >
                  {testResult}
                </p>
              )}
            </div>
          </section>

          <section>
            <h2 className="text-sm font-semibold text-[var(--color-text)] mb-3 uppercase tracking-wide">
              {t.settings.general}
            </h2>
            <div className="space-y-4">
              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--color-text)]">
                  {t.settings.autoStart}
                </span>
                <input
                  type="checkbox"
                  checked={form.auto_start}
                  onChange={(e) => updateField("auto_start", e.target.checked)}
                  className="w-5 h-5 rounded border-[var(--color-border)] bg-[var(--color-surface)] accent-[var(--color-primary)]"
                />
              </label>

              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--color-text)]">
                  {t.settings.startHidden}
                </span>
                <input
                  type="checkbox"
                  checked={form.start_hidden}
                  onChange={(e) =>
                    updateField("start_hidden", e.target.checked)
                  }
                  className="w-5 h-5 rounded border-[var(--color-border)] bg-[var(--color-surface)] accent-[var(--color-primary)]"
                />
              </label>

              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--color-text)]">
                  {t.settings.minimizeToTray}
                </span>
                <input
                  type="checkbox"
                  checked={form.minimize_to_tray}
                  onChange={(e) =>
                    updateField("minimize_to_tray", e.target.checked)
                  }
                  className="w-5 h-5 rounded border-[var(--color-border)] bg-[var(--color-surface)] accent-[var(--color-primary)]"
                />
              </label>

              <div>
                <label className="block text-xs text-[var(--color-text-secondary)] mb-1">
                  {t.settings.retention}
                </label>
                <input
                  type="number"
                  min={1}
                  max={365}
                  value={form.history_retention_days}
                  onChange={(e) =>
                    updateField(
                      "history_retention_days",
                      parseInt(e.target.value) || 7,
                    )
                  }
                  className="w-full px-3 py-2 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                />
              </div>
            </div>
          </section>

          {/* Language selector */}
          <section>
            <h2 className="text-sm font-semibold text-[var(--color-text)] mb-3 uppercase tracking-wide">
              {t.language.label}
            </h2>
            <div className="flex gap-2">
              {(["zh-CN", "en-US"] as const).map((locale) => (
                <button
                  key={locale}
                  onClick={() => {
                    const { setLocale } = useI18n.getState();
                    setLocale(locale);
                  }}
                  className={`px-4 py-2 text-sm rounded-md border transition-colors ${
                    useI18n((s) => s.locale) === locale
                      ? "border-[var(--color-primary)] bg-[var(--color-primary)]/10 text-[var(--color-primary)]"
                      : "border-[var(--color-border)] text-[var(--color-text-secondary)] hover:border-[var(--color-text-secondary)]"
                  }`}
                >
                  {locale === "zh-CN" ? t.language.zhCN : t.language.enUS}
                </button>
              ))}
            </div>
          </section>

          <section>
            <h2 className="text-sm font-semibold text-[var(--color-text)] mb-3 uppercase tracking-wide">
              {t.settings.dnd}
            </h2>
            <p className="text-xs text-[var(--color-text-secondary)] mb-3">
              {t.settings.dndHint}
            </p>
            <div className="flex items-center gap-3">
              <div className="flex-1">
                <label className="block text-xs text-[var(--color-text-secondary)] mb-1">
                  {t.settings.dndStart}
                </label>
                <input
                  type="time"
                  value={form.do_not_disturb_start}
                  onChange={(e) =>
                    updateField("do_not_disturb_start", e.target.value)
                  }
                  className="w-full px-3 py-2 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                />
              </div>
              <span className="text-[var(--color-text-secondary)] mt-5">
                {t.settings.dndTo}
              </span>
              <div className="flex-1">
                <label className="block text-xs text-[var(--color-text-secondary)] mb-1">
                  {t.settings.dndEnd}
                </label>
                <input
                  type="time"
                  value={form.do_not_disturb_end}
                  onChange={(e) =>
                    updateField("do_not_disturb_end", e.target.value)
                  }
                  className="w-full px-3 py-2 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                />
              </div>
            </div>
          </section>

          <div className="pt-4">
            <button
              onClick={handleSave}
              disabled={saving}
              className="w-full py-2.5 text-sm font-medium rounded-md bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] disabled:opacity-50 transition-colors"
            >
              {saving ? t.settings.saving : t.settings.save}
            </button>
            {statusMessage && (
              <p
                className={`text-xs mt-2 text-center ${
                  statusMessage.startsWith("Failed")
                    ? "text-[var(--color-danger)]"
                    : "text-[var(--color-success)]"
                }`}
              >
                {statusMessage}
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
