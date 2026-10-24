import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n, fmt } from "../../i18n";
import { useTheme } from "../../theme/useTheme";
import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";
import { Input } from "../ui/Input";
import { PageHeader } from "../ui/PageHeader";
import {
  AlertIcon,
  CloseIcon,
  MoonIcon,
  RefreshIcon,
  SearchIcon,
  SunIcon,
} from "../ui/icons";
import { cx } from "../../utils/cx";
import type { MessageSnapshot } from "../../types";

/**
 * Toolbar above the message list: title, unread count, search, connection state
 * and three distinct actions — refresh, theme toggle (icon) and settings (text).
 * Settings deliberately carries no icon: a gear reads as a theme switch.
 */
export function MessageHeader() {
  const { t } = useI18n();
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const messages = useAppStore((s) => s.messages);
  const unreadCount = useAppStore((s) => s.unreadCount);
  const connectionState = useAppStore((s) => s.connectionState);
  const connectionError = useAppStore((s) => s.connectionError);
  const searchQuery = useAppStore((s) => s.searchQuery);
  const setSearchQuery = useAppStore((s) => s.setSearchQuery);
  const { resolved: resolvedTheme, setMode } = useTheme();
  const { applyMutationSnapshot, currentRevision } = useMessageSnapshot();
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [syncServer, setSyncServer] = useState(false);
  const [reconnecting, setReconnecting] = useState(false);

  const isDark = resolvedTheme === "dark";
  const toggleTheme = () => setMode(isDark ? "light" : "dark");

  const handleClear = async () => {
    try {
      const revision = currentRevision();
      const snapshot = await invoke<MessageSnapshot>("clear_messages", { syncServer });
      await applyMutationSnapshot(snapshot, revision);
      setShowClearConfirm(false);
    } catch (e) {
      console.error("Failed to clear messages:", e);
    }
  };

  const handleReconnect = async () => {
    setReconnecting(true);
    try {
      await invoke("connect_websocket");
    } catch (e) {
      console.error("Failed to reconnect:", e);
    } finally {
      setReconnecting(false);
    }
  };

  const dotClass =
    connectionState === "connected"
      ? "bg-[var(--color-success)]"
      : connectionState === "connecting"
        ? "bg-[var(--color-primary)] animate-pulse"
        : "bg-[var(--color-danger)]";

  return (
    <>
      <PageHeader
        title={t.header.title}
        badge={
          unreadCount > 0 ? (
            <span className="flex-shrink-0 text-xs font-medium tabular-nums text-[var(--color-text-muted)]">
              {unreadCount}
            </span>
          ) : null
        }
        actions={
          <>
            <IconButton
              label={t.header.reconnect}
              onClick={handleReconnect}
              disabled={reconnecting || connectionState === "connecting"}
            >
              <RefreshIcon size={15} />
            </IconButton>
            <IconButton
              label={isDark ? t.appearance.light : t.appearance.dark}
              onClick={toggleTheme}
            >
              {isDark ? <SunIcon size={15} /> : <MoonIcon size={15} />}
            </IconButton>
            <Button
              size="md"
              variant="ghost"
              onClick={() => setCurrentView("settings")}
              title={t.header.settings}
              className="px-2.5 font-normal"
            >
              {t.header.settings}
            </Button>
          </>
        }
      >
        <div className="relative mt-1.5">
          <span className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
            <SearchIcon size={13} />
          </span>
          <Input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t.search.placeholder}
            aria-label={t.search.placeholder}
            size="sm"
            leading
            trailing
          />
          {searchQuery !== "" && (
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              title={t.search.clear}
              aria-label={t.search.clear}
              className="absolute right-1 top-1/2 inline-flex h-5 w-5 -translate-y-1/2 items-center justify-center rounded-md text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-border)]/60 hover:text-[var(--color-text)]"
            >
              <CloseIcon size={11} />
            </button>
          )}
        </div>

        <div className="mt-1.5 flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-[11px] text-[var(--color-text-muted)]">
          <span
            className={cx("h-1.5 w-1.5 flex-shrink-0 rounded-full", dotClass)}
            role="status"
            aria-label={t.connection[connectionState]}
            title={t.connection[connectionState]}
          />
          <span>{t.connection[connectionState]}</span>
          <span className="opacity-50">·</span>
          <span className="tabular-nums">{fmt(t.header.cached, { n: messages.length })}</span>
          <div className="flex-1" />
          {messages.length > 0 && (
            <button
              type="button"
              onClick={() => setShowClearConfirm(true)}
              className="rounded-md px-1 py-0.5 text-[11px] transition-colors hover:bg-[var(--color-danger-soft)] hover:text-[var(--color-danger)]"
            >
              {t.header.clearAll}
            </button>
          )}
        </div>

        {connectionError && (
          <div
            role="alert"
            className="mt-1.5 flex items-start gap-1.5 rounded-lg bg-[var(--color-danger-soft)] px-2 py-1 text-[11px] text-[var(--color-danger)]"
          >
            <span className="mt-px flex-shrink-0">
              <AlertIcon size={12} />
            </span>
            <span className="min-w-0 flex-1 break-words">{connectionError}</span>
            <button
              type="button"
              onClick={handleReconnect}
              disabled={reconnecting || connectionState === "connecting"}
              className="flex-shrink-0 underline underline-offset-2 transition-opacity hover:opacity-80 disabled:opacity-40"
            >
              {t.header.reconnect}
            </button>
          </div>
        )}
      </PageHeader>

      {showClearConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-overlay)] p-4">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="clear-all-title"
            className="w-full max-w-xs rounded-xl bg-[var(--color-surface)] p-4 shadow-[var(--shadow-popover)]"
          >
            <h2
              id="clear-all-title"
              className="text-[13px] font-semibold text-[var(--color-text)]"
            >
              {t.clearAll.title}
            </h2>
            <p className="mt-1.5 text-xs leading-relaxed text-[var(--color-text-secondary)]">
              {t.clearAll.confirm}
            </p>
            <label className="mt-3 flex cursor-pointer items-start gap-2 text-xs text-[var(--color-text-secondary)]">
              <input
                type="checkbox"
                checked={syncServer}
                onChange={(e) => setSyncServer(e.target.checked)}
                className="mt-0.5 h-3.5 w-3.5 rounded accent-[var(--color-primary)]"
              />
              <span>{t.clearAll.syncServer}</span>
            </label>
            <div className="mt-4 flex justify-end gap-1.5">
              <Button variant="ghost" size="sm" onClick={() => setShowClearConfirm(false)}>
                {t.clearAll.cancel}
              </Button>
              <Button variant="danger" size="sm" onClick={() => void handleClear()}>
                {t.clearAll.confirmButton}
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
