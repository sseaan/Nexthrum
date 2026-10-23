import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { MessageSnapshot } from "../types";

const MAX_SYNC_ATTEMPTS = 3;

/**
 * Applies backend-owned message snapshots without letting a stale response
 * overwrite state that changed while the request was in flight.
 *
 * `backendRevision` is bumped by every backend-originated update (new message,
 * unread count, snapshot). A mutation snapshot is only applied when no such
 * event arrived meanwhile; otherwise the snapshot is re-fetched, bounded so a
 * busy server cannot cause an endless loop.
 */
export function useMessageSnapshot() {
  const applyMessageSnapshot = useAppStore((s) => s.applyMessageSnapshot);

  const currentRevision = useCallback(
    () => useAppStore.getState().backendRevision,
    [],
  );

  const refresh = useCallback(async () => {
    let lastSnapshot: MessageSnapshot | null = null;
    for (let attempt = 0; attempt < MAX_SYNC_ATTEMPTS; attempt += 1) {
      const revision = currentRevision();
      lastSnapshot = await invoke<MessageSnapshot>("get_message_snapshot");
      if (currentRevision() === revision) {
        applyMessageSnapshot(lastSnapshot);
        return;
      }
    }
    // Live traffic kept arriving. Apply the newest copy we fetched; the next
    // event or snapshot re-syncs again, so this cannot drift permanently.
    if (lastSnapshot) {
      applyMessageSnapshot(lastSnapshot);
    }
  }, [applyMessageSnapshot, currentRevision]);

  const applyMutationSnapshot = useCallback(
    async (snapshot: MessageSnapshot, revisionAtStart: number) => {
      if (currentRevision() === revisionAtStart) {
        applyMessageSnapshot(snapshot);
        return;
      }
      await refresh();
    },
    [applyMessageSnapshot, currentRevision, refresh],
  );

  return { applyMutationSnapshot, refresh, currentRevision };
}
