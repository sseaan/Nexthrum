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
 * Flat Mail-like row: no card, no border, hairline separators come from the
 * list container. Unread state uses three redundant cues (dot, weight and the
 * accent-tinted metadata) so it does not depend on colour alone.
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

  // The row is clickable, so it must also be reachable from the keyboard.
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
        className="group relative cursor-pointer rounded-lg px-2 py-2.5 transition-colors hover:bg-[var(--color-surface-hover)]"
        aria-label={`${isUnread ? t.message.unread : t.message.read}: ${
          message.title || t.message.noTitle
        }, ${formatFullDate(message.date)}`}
      >
        <div className="flex items-start gap-2">
          {/* Unread dot (cue 1 of 3) */}
          <span
            className={cx(
              "mt-[6px] h-1.5 w-1.5 flex-shrink-0 rounded-full",
              isUnread ? "bg-[var(--color-primary)]" : "bg-transparent",
            )}
            aria-hidden="true"
          />

          <div className="min-w-0 flex-1">
            <div className="flex items-baseline gap-2">
              <h3
                className={cx(
                  "min-w-0 flex-1 truncate text-[13px]",
                  isUnread
                    ? "font-semibold text-[var(--color-text)]"
                    : "font-normal text-[var(--color-text-secondary)]",
                )}
              >
                {message.title || t.message.noTitle}
              </h3>
              <time
                className={cx(
                  "flex-shrink-0 whitespace-nowrap text-[11px] tabular-nums",
                  isUnread
                    ? "text-[var(--color-primary)]"
                    : "text-[var(--color-text-muted)]",
                )}
                dateTime={message.date}
                title={formatFullDate(message.date)}
              >
                {formatDate(message.date)}
              </time>
            </div>

            <p
              className={cx(
                "mt-0.5 line-clamp-2 text-xs leading-relaxed",
                isUnread
                  ? "text-[var(--color-text-secondary)]"
                  : "text-[var(--color-text-muted)]",
              )}
            >
              {truncate(message.message, 300)}
            </p>

            <div className="mt-1 flex items-center gap-1.5 text-[11px] text-[var(--color-text-muted)]">
              <span className="max-w-[11rem] truncate" title={appLabel(message.appid)}>
                {appLabel(message.appid)}
              </span>
              {message.priority > 0 && (
                <>
                  <span className="opacity-40">·</span>
                  <span className="tabular-nums">P{message.priority}</span>
                </>
              )}
            </div>
          </div>
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
            className="fixed z-50 min-w-[8.5rem] rounded-lg bg-[var(--color-surface)] p-1 shadow-[var(--shadow-popover)]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            {isUnread && (
              <button
                type="button"
                role="menuitem"
                onClick={handleMarkRead}
                className="w-full rounded-md px-2 py-1 text-left text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
              >
                {t.message.markRead}
              </button>
            )}
            <button
              type="button"
              role="menuitem"
              onClick={handleDelete}
              className="w-full rounded-md px-2 py-1 text-left text-xs text-[var(--color-danger)] transition-colors hover:bg-[var(--color-surface-hover)]"
            >
              {t.message.delete}
            </button>
          </div>
        </>
      )}
    </>
  );
}
