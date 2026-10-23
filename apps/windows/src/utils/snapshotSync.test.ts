import { describe, expect, it, vi } from "vitest";
import { applyMutationSnapshotSafely, fetchStableSnapshot } from "./snapshotSync";
import type { MessageSnapshot } from "../types";

function snapshot(unread: number): MessageSnapshot {
  return {
    messages: [
      {
        id: unread,
        appid: 1,
        message: `m${unread}`,
        title: "t",
        priority: 0,
        date: "2026-10-23T02:05:00Z",
        read: false,
        received_at: "2026-10-23T02:05:00Z",
      },
    ],
    unread_count: unread,
  };
}

describe("fetchStableSnapshot", () => {
  it("applies the snapshot when no event arrives in flight", async () => {
    const applySnapshot = vi.fn();
    const result = await fetchStableSnapshot({
      currentRevision: () => 4,
      fetchSnapshot: async () => snapshot(1),
      applySnapshot,
    });

    expect(applySnapshot).toHaveBeenCalledTimes(1);
    expect(result?.unread_count).toBe(1);
  });

  it("retries when a backend event bumps the revision during the fetch", async () => {
    let revision = 0;
    const applySnapshot = vi.fn();
    const fetchSnapshot = vi.fn(async () => {
      revision += 1; // simulate a live event arriving mid-request
      return snapshot(revision);
    });

    const result = await fetchStableSnapshot({
      currentRevision: () => revision,
      fetchSnapshot,
      applySnapshot,
    });

    // Revision keeps changing, so all attempts are used and the last one applies.
    expect(fetchSnapshot).toHaveBeenCalledTimes(3);
    expect(applySnapshot).toHaveBeenCalledTimes(1);
    expect(result).not.toBeNull();
  });

  it("stops retrying as soon as the revision is stable", async () => {
    let revision = 0;
    const applySnapshot = vi.fn();
    const fetchSnapshot = vi.fn(async () => {
      if (fetchSnapshot.mock.calls.length === 1) revision += 1;
      return snapshot(1);
    });

    await fetchStableSnapshot({
      currentRevision: () => revision,
      fetchSnapshot,
      applySnapshot,
    });

    expect(fetchSnapshot).toHaveBeenCalledTimes(2);
    expect(applySnapshot).toHaveBeenCalledTimes(1);
  });

  it("never applies anything if no snapshot could be fetched", async () => {
    const applySnapshot = vi.fn();
    await expect(
      fetchStableSnapshot({
        currentRevision: () => 0,
        fetchSnapshot: async () => {
          throw new Error("offline");
        },
        applySnapshot,
      }),
    ).rejects.toThrow("offline");
    expect(applySnapshot).not.toHaveBeenCalled();
  });
});

describe("applyMutationSnapshotSafely", () => {
  it("applies a mutation snapshot when nothing changed in between", async () => {
    const applySnapshot = vi.fn();
    await applyMutationSnapshotSafely(snapshot(2), 7, {
      currentRevision: () => 7,
      fetchSnapshot: async () => snapshot(0),
      applySnapshot,
    });

    expect(applySnapshot).toHaveBeenCalledTimes(1);
    expect(applySnapshot.mock.calls[0][0].unread_count).toBe(2);
  });

  it("re-fetches instead of applying a stale mutation snapshot", async () => {
    const applySnapshot = vi.fn();
    const fetchSnapshot = vi.fn(async () => snapshot(9));

    await applyMutationSnapshotSafely(snapshot(2), 7, {
      currentRevision: () => 8,
      fetchSnapshot,
      applySnapshot,
    });

    expect(fetchSnapshot).toHaveBeenCalledTimes(1);
    expect(applySnapshot.mock.calls[0][0].unread_count).toBe(9);
  });
});
