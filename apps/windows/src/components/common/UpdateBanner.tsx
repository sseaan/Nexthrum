import { useI18n, fmt } from "../../i18n";
import { isAttentionState, type UpdateStatus } from "../../utils/updateState";
import { Button } from "../ui/Button";
import { AlertIcon, CloseIcon, DownloadIcon } from "../ui/icons";
import { cx } from "../../utils/cx";

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

  const isError = status.kind === "error";

  return (
    <div
      role="status"
      aria-live="polite"
      className={cx(
        "flex-shrink-0 border-b",
        isError
          ? "border-[var(--color-danger)]/25 bg-[var(--color-danger-soft)]"
          : "border-[var(--color-border)] bg-[var(--color-primary-soft)]",
      )}
    >
      <div className="mx-auto flex w-full max-w-3xl items-center gap-2 px-4 py-2 text-xs">
        <span
          className={cx(
            "flex-shrink-0",
            isError ? "text-[var(--color-danger)]" : "text-[var(--color-primary)]",
          )}
          aria-hidden="true"
        >
          {isError ? <AlertIcon size={14} /> : <DownloadIcon size={14} />}
        </span>
        <span className="min-w-0 flex-1 break-words text-[var(--color-text)]">{message}</span>

        {status.kind === "available" && (
          <Button size="sm" variant="primary" onClick={onInstall} className="h-7 px-2 text-[11px]">
            {t.update.install}
          </Button>
        )}
        {status.kind === "ready" && (
          <Button size="sm" variant="primary" onClick={onRestart} className="h-7 px-2 text-[11px]">
            {t.update.restart}
          </Button>
        )}
        {(status.kind === "available" || isError) && (
          <button
            type="button"
            onClick={onDismiss}
            className="inline-flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-md text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            aria-label={t.update.dismiss}
            title={t.update.dismiss}
          >
            <CloseIcon size={12} />
          </button>
        )}
      </div>
    </div>
  );
}
