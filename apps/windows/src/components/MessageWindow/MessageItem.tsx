import { useState, useCallback, type KeyboardEvent, type MouseEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n } from "../../i18n";
import { useAppStore } from "../../store/useAppStore";
import { formatDate, formatFullDate, truncate } from "../../utils/format";
import { cx } from "../../utils/cx";
import type { MessageListItem, MessageSnapshot } from "../../types";

interface MessageItemProps {
  message: MessageListItem;
}

/**
 * One row in the message list.
 *
 * Unread state uses three redundant cues (left accent bar, dot, font weight) so
 * it does not depend on colour alone; read rows drop to secondary colours.
 */
export function MessageItem({ message }: MessageItemProps) {
  const { t } = useI18n();
  const { applyMutationSnapshot, currentRevision } = useMessageSnapshot();
  const appLabel = useAppStore((s) => s.appLabel);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const handleClick = useCallback(async () => {
    // Copy message content to clipboard
    try {
      await navigator.clipboard.writeText(message.message);
    } catch {
      // Fallback for environments without clipboard API
    }

    // Mark as read if unread
    if (!message.read) {
      try {
        const revision = currentRevision();
        const snapshot = await invoke<MessageSnapshot>("mark_message_read", {
          messageId: message.id,
        });
        await applyMutationSnapshot(snapshot, revision);
      } catch (e) {
        console.error("Failed to mark read:", e);
      }
    }
  }, [message, applyMutationSnapshot, currentRevision]);

  const handleContextMenu = useCallback((e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  }, []);

  const handleDelete = useCallback(async () => {
    try {
      const revision = currentRevision();
      const snapshot = await invoke<MessageSnapshot>("delete_message", {
        messageId: message.id,
      });
      await applyMutationSnapshot(snapshot, revision);
    } catch (e) {
      console.error("Failed to delete message:", e);
    }
    setContextMenu(null);
  }, [message.id, applyMutationSnapshot, currentRevision]);

  const handleMarkRead = useCallback(async () => {
    try {
      const revision = currentRevision();
      const snapshot = await invoke<MessageSnapshot>("mark_message_read", {
        messageId: message.id,
      });
      await applyMutationSnapshot(snapshot, revision);
    } catch (e) {
      console.error("Failed to mark read:", e);
    }
    setContextMenu(null);
  }, [message.id, applyMutationSnapshot, currentRevision]);

  const handleCloseContext = useCallback(() => {
    setContextMenu(null);
  }, []);

  // The card is clickable, so it must also be reachable from the keyboard.
  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLElement>) => {
      if (e.key !== "Enter" && e.key !== " ") return;
      e.preventDefault();
      void handleClick();
    },
    [handleClick],
  );

  const isUnread = !message.read;

  return (
    <>
      <article
        role="button"
        tabIndex={0}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        onContextMenu={handleContextMenu}
        className={cx(
          "group relative cursor-pointer overflow-hidden rounded-xl border py-2.5 pl-3.5 pr-3 transition-colors",
          "hover:shadow-[var(--shadow-card)]",
          isUnread
            ? "border-[var(--color-primary)]/35 bg-[var(--color-primary-soft)] hover:border-[var(--color-primary)]/55"
            : "border-[var(--color-border)] bg-[var(--color-surface)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-hover)]",
        )}
        aria-label={`${isUnread ? t.message.unread : t.message.read}: ${
          message.title || t.message.noTitle
        }, ${formatFullDate(message.date)}`}
      >
        {/* Unread accent bar (cue 1 of 3) */}
        <span
          className={cx(
            "absolute bottom-2 left-0 top-2 w-[3px] rounded-r-full transition-colors",
            isUnread ? "bg-[var(--color-primary)]" : "bg-transparent",
          )}
          aria-hidden="true"
        />

        <div className="flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex min-w-0 items-center gap-1.5">
              {/* Unread dot (cue 2) */}
              {isUnread && (
                <span
                  className="h-1.5 w-1.5 flex-shrink-0 rounded-full bg-[var(--color-primary)]"
                  aria-hidden="true"
                />
              )}
              <h3
                className={cx(
                  "min-w-0 truncate text-sm",
                  isUnread
                    ? "font-semibold text-[var(--color-text)]"
                    : "font-normal text-[var(--color-text-secondary)]",
                )}
              >
                {message.title || t.message.noTitle}
              </h3>
            </div>

            <p
              className={cx(
                "mt-1 line-clamp-3 text-xs leading-relaxed",
                isUnread
                  ? "text-[var(--color-text-secondary)]"
                  : "text-[var(--color-text-muted)]",
              )}
            >
              {truncate(message.message, 300)}
            </p>

            <div className="mt-2 flex flex-wrap items-center gap-1.5">
              <span
                title={appLabel(message.appid)}
                className={cx(
                  "max-w-[12rem] truncate rounded-full px-1.5 py-0.5 text-[10px] font-medium",
                  isUnread
                    ? "bg-[var(--color-primary)]/15 text-[var(--color-primary)]"
                    : "bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]",
                )}
              >
                {appLabel(message.appid)}
              </span>
              {message.priority > 0 && (
                <span className="rounded-full bg-[var(--color-surface-hover)] px-1.5 py-0.5 text-[10px] tabular-nums text-[var(--color-text-muted)]">
                  P{message.priority}
                </span>
              )}
            </div>
          </div>

          <time
            className="mt-0.5 flex-shrink-0 whitespace-nowrap text-[11px] tabular-nums text-[var(--color-text-muted)]"
            dateTime={message.date}
            title={formatFullDate(message.date)}
          >
            {formatDate(message.date)}
          </time>
        </div>
      </article>

      {/* Context Menu */}
      {contextMenu && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={handleCloseContext}
            onContextMenu={(e) => {
              e.preventDefault();
              handleCloseContext();
            }}
          />
          <div
            role="menu"
            className="fixed z-50 min-w-[9rem] rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] py-1 shadow-[var(--shadow-popover)]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            {isUnread && (
              <button
                type="button"
                role="menuitem"
                onClick={handleMarkRead}
                className="w-full px-3 py-1.5 text-left text-sm text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
              >
                {t.message.markRead}
              </button>
            )}
            <button
              type="button"
              role="menuitem"
              onClick={handleDelete}
              className="w-full px-3 py-1.5 text-left text-sm text-[var(--color-danger)] transition-colors hover:bg-[var(--color-surface-hover)]"
            >
              {t.message.delete}
            </button>
          </div>
        </>
      )}
    </>
  );
}
