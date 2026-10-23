/** Explicit states for the update flow so the UI cannot show a stale label. */
export type UpdateStatus =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "up-to-date" }
  | { kind: "available"; version: string; notes?: string }
  | { kind: "downloading"; version: string; progress: number | null }
  | { kind: "ready"; version: string }
  | { kind: "error"; message: string };

export const initialUpdateStatus: UpdateStatus = { kind: "idle" };

/** Bytes downloaded / total, or `null` when the total is unknown. */
export function downloadProgress(downloaded: number, total?: number | null): number | null {
  if (!total || total <= 0) return null;
  return Math.min(1, Math.max(0, downloaded / total));
}

/** True when the banner should stay visible. */
export function isAttentionState(status: UpdateStatus): boolean {
  return status.kind === "available" || status.kind === "downloading" || status.kind === "ready";
}
