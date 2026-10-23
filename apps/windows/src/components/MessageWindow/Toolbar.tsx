import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n, fmt } from "../../i18n";
import type { MessageSnapshot } from "../../types";

export function Toolbar() {
  const { t } = useI18n();
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const unreadCount = useAppStore((s) => s.unreadCount);
  const connectionState = useAppStore((s) => s.connectionState);
  const { applyMutationSnapshot, currentRevision } = useMessageSnapshot();
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [syncServer, setSyncServer] = useState(false);

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

  return (
    <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="flex items-center gap-3">
        <span
          className={`w-2 h-2 rounded-full ${
            connectionState === "connected"
              ? "bg-[var(--color-success)]"
              : connectionState === "connecting"
                ? "bg-[var(--color-primary)] animate-pulse"
                : "bg-[var(--color-danger)]"
          }`}
          role="status"
          aria-label={t.connection[connectionState]}
          title={t.connection[connectionState]}
        />
      </div>

      <div className="flex items-center gap-2">
        {unreadCount > 0 && (
          <span className="text-xs bg-[var(--color-primary)] text-white px-2 py-0.5 rounded-full">
            {fmt(t.toolbar.unread, { n: unreadCount })}
          </span>
        )}

        <button
          onClick={() => setShowClearConfirm(true)}
          className="px-3 py-1 text-sm rounded-md bg-[var(--color-surface-hover)] hover:bg-[var(--color-border)] text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
          title={t.toolbar.clearAll}
        >
          {t.toolbar.clearAll}
        </button>

        <button
          onClick={() => setCurrentView("settings")}
          className="px-3 py-1 text-sm rounded-md bg-[var(--color-surface-hover)] hover:bg-[var(--color-border)] text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
          title={t.toolbar.settings}
        >
          {t.toolbar.settings}
        </button>
      </div>

      {/* Clear confirmation dialog */}
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
