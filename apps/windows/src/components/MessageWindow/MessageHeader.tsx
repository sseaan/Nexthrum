import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n, fmt } from "../../i18n";
import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";
import { Input } from "../ui/Input";
import { PageHeader } from "../ui/PageHeader";
import { AlertIcon, CloseIcon, RefreshIcon, SearchIcon, SettingsIcon } from "../ui/icons";
import { cx } from "../../utils/cx";
import type { MessageSnapshot } from "../../types";

/**
 * Top bar for the message list: title, unread pill, search, connection status
 * and the primary actions. Mirrors the macOS header so both platforms read the
 * same while keeping a Windows-shaped layout.
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
  const { applyMutationSnapshot, currentRevision } = useMessageSnapshot();
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [syncServer, setSyncServer] = useState(false);
  const [reconnecting, setReconnecting] = useState(false);

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
            <span className="flex-shrink-0 rounded-full bg-[var(--color-primary-soft)] px-2 py-0.5 text-[11px] font-semibold text-[var(--color-primary)] tabular-nums">
              {fmt(t.header.unread, { n: unreadCount })}
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
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setCurrentView("settings")}
              title={t.header.settings}
              aria-label={t.header.settings}
            >
              <SettingsIcon size={15} />
              <span className="hidden min-[560px]:inline">{t.header.settings}</span>
            </Button>
          </>
        }
      >
        <div className="relative mt-2">
          <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
            <SearchIcon size={14} />
          </span>
          <Input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t.search.placeholder}
            aria-label={t.search.placeholder}
            className="h-8 pl-8 pr-8 text-xs"
          />
          {searchQuery !== "" && (
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              title={t.search.clear}
              aria-label={t.search.clear}
              className="absolute right-1.5 top-1/2 inline-flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-md text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              <CloseIcon size={12} />
            </button>
          )}
        </div>

        <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-[var(--color-text-secondary)]">
          <span
            className={cx("h-2 w-2 flex-shrink-0 rounded-full", dotClass)}
            role="status"
            aria-label={t.connection[connectionState]}
            title={t.connection[connectionState]}
          />
          <span>{t.connection[connectionState]}</span>
          <span className="text-[var(--color-text-muted)]">·</span>
          <span className="tabular-nums">{fmt(t.header.cached, { n: messages.length })}</span>
          <div className="flex-1" />
          {messages.length > 0 && (
            <button
              type="button"
              onClick={() => setShowClearConfirm(true)}
              className="rounded-md px-1.5 py-0.5 text-xs text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-danger-soft)] hover:text-[var(--color-danger)]"
            >
              {t.header.clearAll}
            </button>
          )}
        </div>

        {connectionError && (
          <div
            role="alert"
            className="mt-2 flex items-start gap-2 rounded-lg border border-[var(--color-danger)]/25 bg-[var(--color-danger-soft)] px-2.5 py-1.5 text-xs text-[var(--color-danger)]"
          >
            <span className="mt-px flex-shrink-0">
              <AlertIcon size={13} />
            </span>
            <span className="min-w-0 flex-1 break-words">{connectionError}</span>
            <button
              type="button"
              onClick={handleReconnect}
              disabled={reconnecting || connectionState === "connecting"}
              className="flex-shrink-0 rounded underline underline-offset-2 transition-opacity hover:opacity-80 disabled:opacity-40"
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
            className="w-full max-w-sm rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-popover)]"
          >
            <h2
              id="clear-all-title"
              className="text-base font-semibold text-[var(--color-text)]"
            >
              {t.clearAll.title}
            </h2>
            <p className="mt-2 text-sm leading-relaxed text-[var(--color-text-secondary)]">
              {t.clearAll.confirm}
            </p>
            <label className="mt-4 flex cursor-pointer items-start gap-2 text-sm text-[var(--color-text-secondary)]">
              <input
                type="checkbox"
                checked={syncServer}
                onChange={(e) => setSyncServer(e.target.checked)}
                className="mt-0.5 h-4 w-4 rounded border-[var(--color-border)] accent-[var(--color-primary)]"
              />
              <span>{t.clearAll.syncServer}</span>
            </label>
            <div className="mt-5 flex justify-end gap-2">
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
