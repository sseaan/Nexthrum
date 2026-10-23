import type { MessageSnapshot } from "../types";

/** Attempts allowed when re-fetching a snapshot that raced with live events. */
export const MAX_SYNC_ATTEMPTS = 3;

export interface SnapshotSyncDeps {
  /** Reads the current backend revision (bumped by every backend event). */
  currentRevision: () => number;
  /** Fetches a fresh authoritative snapshot. */
  fetchSnapshot: () => Promise<MessageSnapshot>;
  /** Applies a snapshot to the UI store. */
  applySnapshot: (snapshot: MessageSnapshot) => void;
  maxAttempts?: number;
}

/**
 * Fetches a snapshot and applies it only when no backend event arrived while the
 * request was in flight; otherwise it retries a bounded number of times and
 * finally applies the newest copy so the UI cannot drift permanently.
 *
 * Extracted from the React hook so the race handling is unit-testable.
 */
export async function fetchStableSnapshot({
  currentRevision,
  fetchSnapshot,
  applySnapshot,
  maxAttempts = MAX_SYNC_ATTEMPTS,
}: SnapshotSyncDeps): Promise<MessageSnapshot | null> {
  let lastSnapshot: MessageSnapshot | null = null;

  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const revision = currentRevision();
    lastSnapshot = await fetchSnapshot();
    if (currentRevision() === revision) {
      applySnapshot(lastSnapshot);
      return lastSnapshot;
    }
  }

  if (lastSnapshot) {
    applySnapshot(lastSnapshot);
  }
  return lastSnapshot;
}

/**
 * Applies a snapshot returned by a mutation command, falling back to a re-fetch
 * when the response is stale.
 */
export async function applyMutationSnapshotSafely(
  snapshot: MessageSnapshot,
  revisionAtStart: number,
  deps: SnapshotSyncDeps,
): Promise<void> {
  if (deps.currentRevision() === revisionAtStart) {
    deps.applySnapshot(snapshot);
    return;
  }
  await fetchStableSnapshot(deps);
}
