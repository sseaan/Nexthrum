import { useI18n, fmt } from "../../i18n";
import { isAttentionState, type UpdateStatus } from "../../utils/updateState";

interface UpdateBannerProps {
  status: UpdateStatus;
  onInstall: () => void;
  onRestart: () => void;
  onDismiss: () => void;
}

/**
 * Non-blocking banner shown when a signed update is available, downloading or
 * ready to install. Renders nothing for idle/up-to-date/checking states.
 */
export function UpdateBanner({ status, onInstall, onRestart, onDismiss }: UpdateBannerProps) {
  const { t } = useI18n();
  if (!isAttentionState(status) && status.kind !== "error") return null;

  const percent =
    status.kind === "downloading" && status.progress !== null
      ? Math.round(status.progress * 100)
      : null;

  const message =
    status.kind === "available"
      ? fmt(t.update.available, { version: status.version })
      : status.kind === "downloading"
        ? percent === null
          ? fmt(t.update.downloading, { version: status.version })
          : fmt(t.update.downloadingPercent, { version: status.version, percent })
        : status.kind === "ready"
          ? t.update.ready
          : t.update.failed;

  return (
    <div
      role="status"
      aria-live="polite"
      className="flex items-center gap-2 px-4 py-2 text-xs border-b border-[var(--color-border)] bg-[var(--color-primary)]/10 text-[var(--color-text)]"
    >
      <span aria-hidden="true">⤓</span>
      <span className="flex-1 break-words">{message}</span>
      {status.kind === "available" && (
        <button
          onClick={onInstall}
          className="px-2 py-1 rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] transition-colors"
        >
          {t.update.install}
        </button>
      )}
      {status.kind === "ready" && (
        <button
          onClick={onRestart}
          className="px-2 py-1 rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] transition-colors"
        >
          {t.update.restart}
        </button>
      )}
      {(status.kind === "available" || status.kind === "error") && (
        <button
          onClick={onDismiss}
          className="text-[var(--color-text-secondary)] hover:text-[var(--color-text)]"
          aria-label={t.update.dismiss}
          title={t.update.dismiss}
        >
          ✕
        </button>
      )}
    </div>
  );
}
