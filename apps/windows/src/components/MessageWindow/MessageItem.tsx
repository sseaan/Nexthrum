import { useState, useCallback, type MouseEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMessageSnapshot } from "../../hooks/useMessageSnapshot";
import { useI18n } from "../../i18n";
import { useAppStore } from "../../store/useAppStore";
import { formatDate, formatFullDate, truncate } from "../../utils/format";
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

  const isUnread = !message.read;

  return (
    <>
      <div
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        className={`group relative pl-4 pr-4 py-3 border-b border-[var(--color-border)] cursor-pointer transition-colors hover:bg-[var(--color-surface-hover)] ${
          isUnread ? "bg-[var(--color-surface)]" : "bg-[var(--color-bg)]"
        }`}
        aria-label={`${isUnread ? t.message.unread : t.message.read}: ${
          message.title || t.message.noTitle
        }, ${formatFullDate(message.date)}`}
      >
        {/* Unread accent bar (cue 1 of 3) */}
        <span
          className={`absolute left-0 top-0 bottom-0 w-[3px] ${
            isUnread ? "bg-[var(--color-primary)]" : "bg-transparent"
          }`}
          aria-hidden="true"
        />

        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              {/* Unread dot (cue 2) */}
              {isUnread && (
                <span
                  className="w-1.5 h-1.5 rounded-full bg-[var(--color-primary)] flex-shrink-0"
                  aria-hidden="true"
                />
              )}
              <h3
                className={`text-sm truncate ${
                  isUnread
                    ? "font-bold text-[var(--color-text)]"
                    : "font-normal text-[var(--color-text-secondary)]"
                }`}
              >
                {message.title || t.message.noTitle}
              </h3>
            </div>
            <p
              className={`text-xs leading-relaxed line-clamp-3 ${
                isUnread
                  ? "text-[var(--color-text)]"
                  : "text-[var(--color-text-secondary)]"
              }`}
            >
              {truncate(message.message, 300)}
            </p>
            <div className="flex items-center gap-1.5 mt-1.5">
              <span
                className={`text-[10px] px-1.5 py-0.5 rounded-full ${
                  isUnread
                    ? "bg-[var(--color-primary)]/15 text-[var(--color-primary)]"
                    : "bg-[var(--color-surface-hover)] text-[var(--color-text-secondary)]"
                }`}
              >
                {appLabel(message.appid)}
              </span>
              {message.priority > 0 && (
                <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--color-surface-hover)] text-[var(--color-text-secondary)] tabular-nums">
                  P{message.priority}
                </span>
              )}
            </div>
          </div>
          <time
            className="text-xs text-[var(--color-text-secondary)] whitespace-nowrap flex-shrink-0 mt-0.5 tabular-nums"
            dateTime={message.date}
            title={formatFullDate(message.date)}
          >
            {formatDate(message.date)}
          </time>
        </div>
      </div>

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
            className="fixed z-50 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-md shadow-xl py-1 min-w-[140px]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            {isUnread && (
              <button
                onClick={handleMarkRead}
                className="w-full text-left px-4 py-2 text-sm text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-colors"
              >
                {t.message.markRead}
              </button>
            )}
            <button
              onClick={handleDelete}
              className="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-surface-hover)] transition-colors"
            >
              {t.message.delete}
            </button>
          </div>
        </>
      )}
    </>
  );
}
