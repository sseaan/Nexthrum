import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useI18n, fmt } from "../../i18n";
import { useTheme, type ThemeMode } from "../../theme/useTheme";
import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";
import { Field, Input } from "../ui/Input";
import { PageHeader } from "../ui/PageHeader";
import { Section } from "../ui/Section";
import { SegmentedControl } from "../ui/SegmentedControl";
import { Switch } from "../ui/Switch";
import { AlertIcon, CheckIcon, ChevronLeftIcon } from "../ui/icons";
import { cx } from "../../utils/cx";
import type { AppSettings } from "../../types";

export function SettingsWindow() {
  const { t, locale, setLocale } = useI18n();
  const themeMode = useTheme((s) => s.mode);
  const setThemeMode = useTheme((s) => s.setMode);
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
        setStatusMessage(fmt(t.settings.saveFailed, { error }));
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

  const updateField = <K extends keyof AppSettings>(
    field: K,
    value: AppSettings[K],
  ) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  const statusIsError =
    statusMessage !== null &&
    !statusMessage.includes(t.settings.saved);

  return (
    <div className="flex h-full min-h-0 flex-col bg-[var(--color-bg)]">
      <PageHeader
        title={t.settings.title}
        leading={
          <IconButton
            label={t.settings.back}
            onClick={() => setCurrentView("messages")}
            className="-ml-1.5"
          >
            <ChevronLeftIcon size={16} />
          </IconButton>
        }
        actions={
          <Button size="sm" variant="primary" onClick={handleSave} disabled={saving}>
            {saving ? t.settings.saving : t.settings.save}
          </Button>
        }
      >
        {statusMessage && (
          <div
            role="status"
            className={cx(
              "mt-2 flex items-start gap-2 rounded-lg border px-2.5 py-1.5 text-xs",
              statusIsError
                ? "border-[var(--color-danger)]/25 bg-[var(--color-danger-soft)] text-[var(--color-danger)]"
                : "border-[var(--color-success)]/25 bg-[var(--color-success-soft)] text-[var(--color-success)]",
            )}
          >
            <span className="mt-px flex-shrink-0">
              {statusIsError ? <AlertIcon size={13} /> : <CheckIcon size={13} />}
            </span>
            <span className="break-words">{statusMessage}</span>
          </div>
        )}
      </PageHeader>

      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-2xl space-y-6 px-4 py-5">
          <Section title={t.settings.serverConnection}>
            <Field label={t.settings.serverUrl} htmlFor="settings-server-url">
              <Input
                id="settings-server-url"
                type="text"
                value={form.server_url}
                onChange={(e) => updateField("server_url", e.target.value)}
                placeholder="https://gotify.example.com"
                spellCheck={false}
                autoComplete="url"
              />
            </Field>

            <Field
              label={t.settings.clientToken}
              htmlFor="settings-client-token"
              hint={t.settings.tokenHint}
            >
              <Input
                id="settings-client-token"
                type="password"
                value={form.client_token}
                onChange={(e) => updateField("client_token", e.target.value)}
                placeholder="Axxxxxxxxxxxxxx"
                autoComplete="off"
              />
            </Field>

            <div className="flex flex-wrap items-center gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={handleTestConnection}
                disabled={testing}
              >
                {testing ? t.settings.testing : t.settings.testConnection}
              </Button>
              {testResult && (
                <p
                  className={cx(
                    "min-w-0 flex-1 text-xs leading-relaxed",
                    testResult.startsWith("Connected")
                      ? "text-[var(--color-success)]"
                      : "text-[var(--color-danger)]",
                  )}
                >
                  {testResult}
                </p>
              )}
            </div>
          </Section>

          <Section title={t.settings.general}>
            <Switch
              id="settings-auto-start"
              label={t.settings.autoStart}
              description={t.settings.autoStartHint}
              checked={form.auto_start}
              onChange={(checked) => updateField("auto_start", checked)}
            />
            <Switch
              id="settings-start-hidden"
              label={t.settings.startHidden}
              description={t.settings.startHiddenHint}
              checked={form.start_hidden}
              onChange={(checked) => updateField("start_hidden", checked)}
            />
            <Switch
              id="settings-minimize-to-tray"
              label={t.settings.minimizeToTray}
              description={t.settings.minimizeToTrayHint}
              checked={form.minimize_to_tray}
              onChange={(checked) => updateField("minimize_to_tray", checked)}
            />

            <Field
              label={t.settings.retention}
              htmlFor="settings-retention"
              hint={t.settings.retentionHint}
              className="max-w-[12rem]"
            >
              <Input
                id="settings-retention"
                type="number"
                min={1}
                max={365}
                value={form.history_retention_days}
                onChange={(e) =>
                  updateField(
                    "history_retention_days",
                    parseInt(e.target.value, 10) || 7,
                  )
                }
                className="tabular-nums"
              />
            </Field>
          </Section>

          <Section title={t.appearance.title} description={t.appearance.hint}>
            <Field label={t.appearance.theme}>
              <SegmentedControl
                label={t.appearance.theme}
                value={themeMode}
                onChange={(value: ThemeMode) => setThemeMode(value)}
                options={[
                  { value: "system", label: t.appearance.system },
                  { value: "light", label: t.appearance.light },
                  { value: "dark", label: t.appearance.dark },
                ]}
              />
            </Field>

            <Field label={t.language.label}>
              <SegmentedControl
                label={t.language.label}
                value={locale === "en-US" ? "en-US" : "zh-CN"}
                onChange={(value) => setLocale(value)}
                options={[
                  { value: "zh-CN", label: t.language.zhCN },
                  { value: "en-US", label: t.language.enUS },
                ]}
              />
            </Field>
          </Section>

          <Section title={t.settings.dnd} description={t.settings.dndHint}>
            <div className="flex items-end gap-3">
              <Field label={t.settings.dndStart} htmlFor="settings-dnd-start" className="flex-1">
                <Input
                  id="settings-dnd-start"
                  type="time"
                  value={form.do_not_disturb_start}
                  onChange={(e) =>
                    updateField("do_not_disturb_start", e.target.value)
                  }
                />
              </Field>
              <span className="pb-2.5 text-xs text-[var(--color-text-muted)]">
                {t.settings.dndTo}
              </span>
              <Field label={t.settings.dndEnd} htmlFor="settings-dnd-end" className="flex-1">
                <Input
                  id="settings-dnd-end"
                  type="time"
                  value={form.do_not_disturb_end}
                  onChange={(e) =>
                    updateField("do_not_disturb_end", e.target.value)
                  }
                />
              </Field>
            </div>
          </Section>

          <div className="pb-2">
            <Button
              variant="primary"
              className="w-full"
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? t.settings.saving : t.settings.save}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
