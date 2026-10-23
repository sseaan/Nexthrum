import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n, fmt } from "../../i18n";
import type { MessageSnapshot } from "../../types";

/**
 * Top bar for the message list: title, unread pill, connection status and the
 * primary actions. Mirrors the macOS header so both platforms read the same.
 */
export function MessageHeader() {
  const { t } = useI18n();
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const messages = useAppStore((s) => s.messages);
  const unreadCount = useAppStore((s) => s.unreadCount);
  const connectionState = useAppStore((s) => s.connectionState);
  const connectionError = useAppStore((s) => s.connectionError);
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
    <div className="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="flex items-center gap-3 px-4 pt-3">
        <h1 className="text-lg font-semibold text-[var(--color-text)]">
          {t.header.title}
        </h1>
        {unreadCount > 0 && (
          <span className="text-xs font-semibold bg-[var(--color-primary)]/15 text-[var(--color-primary)] px-2 py-0.5 rounded-full">
            {fmt(t.header.unread, { n: unreadCount })}
          </span>
        )}
        <div className="flex-1" />
        <button
          onClick={handleReconnect}
          disabled={reconnecting || connectionState === "connecting"}
          title={t.header.reconnect}
          aria-label={t.header.reconnect}
          className="w-7 h-7 inline-flex items-center justify-center rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] disabled:opacity-40 transition-colors"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path
              d="M13.5 8a5.5 5.5 0 1 1-1.8-4.08M13.5 2v3.5H10"
              stroke="currentColor"
              strokeWidth="1.4"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <button
          onClick={() => setCurrentView("settings")}
          title={t.header.settings}
          aria-label={t.header.settings}
          className="w-7 h-7 inline-flex items-center justify-center rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] transition-colors"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <circle cx="8" cy="8" r="2.2" stroke="currentColor" strokeWidth="1.4" />
            <path
              d="M8 1.6v1.6M8 12.8v1.6M1.6 8h1.6M12.8 8h1.6M3.5 3.5l1.1 1.1M11.4 11.4l1.1 1.1M12.5 3.5l-1.1 1.1M4.6 11.4l-1.1 1.1"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      <div className="flex items-center gap-2 px-4 pb-2 pt-1 text-xs text-[var(--color-text-secondary)]">
        <span
          className={`w-2 h-2 rounded-full ${dotClass}`}
          role="status"
          aria-label={t.connection[connectionState]}
          title={t.connection[connectionState]}
        />
        <span>{t.connection[connectionState]}</span>
        <span className="opacity-60">·</span>
        <span>{fmt(t.header.cached, { n: messages.length })}</span>
        <div className="flex-1" />
        {messages.length > 0 && (
          <button
            onClick={() => setShowClearConfirm(true)}
            className="underline-offset-2 hover:underline hover:text-[var(--color-text)] transition-colors"
          >
            {t.header.clearAll}
          </button>
        )}
      </div>

      {connectionError && (
        <div
          role="alert"
          className="flex items-start gap-2 px-4 py-1.5 text-xs text-[var(--color-danger)] bg-[var(--color-danger)]/10 border-t border-[var(--color-danger)]/20"
        >
          <span aria-hidden="true">⚠</span>
          <span className="break-words">{connectionError}</span>
        </div>
      )}

      {showClearConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-[var(--color-surface)] rounded-lg p-6 max-w-sm w-full mx-4 shadow-xl border border-[var(--color-border)]">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-2">
              {t.clearAll.title}
            </h2>
            <p className="text-sm text-[var(--color-text-secondary)] mb-4">
              {t.clearAll.confirm}
            </p>
            <label className="flex items-center gap-2 mb-4 text-sm text-[var(--color-text-secondary)] cursor-pointer">
              <input
                type="checkbox"
                checked={syncServer}
                onChange={(e) => setSyncServer(e.target.checked)}
                className="rounded border-[var(--color-border)] bg-[var(--color-bg)]"
              />
              {t.clearAll.syncServer}
            </label>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-2 text-sm rounded-md text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
              >
                {t.clearAll.cancel}
              </button>
              <button
                onClick={handleClear}
                className="px-4 py-2 text-sm rounded-md bg-[var(--color-danger)] text-white hover:bg-red-600 transition-colors"
              >
                {t.clearAll.confirmButton}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
