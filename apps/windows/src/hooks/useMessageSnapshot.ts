import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { MessageSnapshot } from "../types";
import {
  applyMutationSnapshotSafely,
  fetchStableSnapshot,
} from "../utils/snapshotSync";

/**
 * Applies backend-owned message snapshots without letting a stale response
 * overwrite state that changed while the request was in flight.
 *
 * The race handling lives in `utils/snapshotSync` so it can be unit-tested;
 * this hook only wires it to the Tauri command and the Zustand store.
 */
export function useMessageSnapshot() {
  const applyMessageSnapshot = useAppStore((s) => s.applyMessageSnapshot);

  const currentRevision = useCallback(
    () => useAppStore.getState().backendRevision,
    [],
  );

  const deps = useCallback(
    () => ({
      currentRevision,
      fetchSnapshot: () => invoke<MessageSnapshot>("get_message_snapshot"),
      applySnapshot: applyMessageSnapshot,
    }),
    [applyMessageSnapshot, currentRevision],
  );

  const refresh = useCallback(async () => {
    await fetchStableSnapshot(deps());
  }, [deps]);

  const applyMutationSnapshot = useCallback(
    async (snapshot: MessageSnapshot, revisionAtStart: number) => {
      await applyMutationSnapshotSafely(snapshot, revisionAtStart, deps());
    },
    [deps],
  );

  return { applyMutationSnapshot, refresh, currentRevision };
}
