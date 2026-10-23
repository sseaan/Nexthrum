import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import { cx } from "../../utils/cx";

/**
 * Source filter chips. Shown only when messages come from two or more Gotify
 * applications, so a single-source setup keeps the list uncluttered.
 */
export function AppFilter() {
  const { t } = useI18n();
  const apps = useAppStore((s) => s.apps);
  const messages = useAppStore((s) => s.messages);
  const selectedAppId = useAppStore((s) => s.selectedAppId);
  const setSelectedAppId = useAppStore((s) => s.setSelectedAppId);

  if (apps.length < 2) return null;

  const unreadByApp = new Map<number, number>();
  for (const message of messages) {
    if (!message.read) {
      unreadByApp.set(message.appid, (unreadByApp.get(message.appid) ?? 0) + 1);
    }
  }
  const unreadTotal = messages.reduce((sum, m) => sum + (m.read ? 0 : 1), 0);

  const chip = (
    key: string,
    label: string,
    total: number,
    unread: number,
    selected: boolean,
    onClick: () => void,
  ) => (
    <button
      key={key}
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      aria-label={
        unread > 0
          ? `${label}, ${total} messages, ${unread} unread`
          : `${label}, ${total} messages`
      }
      className={cx(
        "flex flex-shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full border px-2.5 py-1 text-xs transition-colors",
        selected
          ? "border-[var(--color-primary)] bg-[var(--color-primary)] text-white"
          : "border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-secondary)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
      )}
    >
      <span className={cx("max-w-[10rem] truncate", selected && "font-semibold")}>
        {label}
      </span>
      <span className="tabular-nums opacity-75">{total}</span>
      {unread > 0 && (
        <span
          className={cx(
            "rounded-full px-1 font-bold tabular-nums",
            selected
              ? "bg-white/25"
              : "bg-[var(--color-primary-soft)] text-[var(--color-primary)]",
          )}
        >
          {unread}
        </span>
      )}
    </button>
  );

  return (
    <div className="flex-shrink-0 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div
        className="mx-auto flex w-full max-w-3xl items-center gap-1.5 overflow-x-auto px-4 py-2"
        aria-label={t.filter.label}
        role="group"
      >
        {chip(
          "all",
          t.filter.allApps,
          messages.length,
          unreadTotal,
          selectedAppId === null,
          () => setSelectedAppId(null),
        )}
        {apps.map((app) =>
          chip(
            String(app.appid),
            app.appname,
            messages.filter((m) => m.appid === app.appid).length,
            unreadByApp.get(app.appid) ?? 0,
            selectedAppId === app.appid,
            () => setSelectedAppId(app.appid),
          ),
        )}
      </div>
    </div>
  );
}
