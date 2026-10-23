import { describe, expect, it } from "vitest";
import { downloadProgress, isAttentionState, type UpdateStatus } from "./updateState";

describe("downloadProgress", () => {
  it("returns a fraction when the total is known", () => {
    expect(downloadProgress(50, 200)).toBe(0.25);
    expect(downloadProgress(200, 200)).toBe(1);
  });

  it("clamps out-of-range values", () => {
    expect(downloadProgress(300, 200)).toBe(1);
    expect(downloadProgress(-10, 200)).toBe(0);
  });

  it("returns null when the total is unknown", () => {
    expect(downloadProgress(50)).toBeNull();
    expect(downloadProgress(50, 0)).toBeNull();
  });
});

describe("isAttentionState", () => {
  it("shows the banner only for actionable states", () => {
    const shown: UpdateStatus[] = [
      { kind: "available", version: "2.1.0" },
      { kind: "downloading", version: "2.1.0", progress: 0.4 },
      { kind: "ready", version: "2.1.0" },
    ];
    const hidden: UpdateStatus[] = [
      { kind: "idle" },
      { kind: "checking" },
      { kind: "up-to-date" },
    ];
    expect(shown.every(isAttentionState)).toBe(true);
    expect(hidden.some(isAttentionState)).toBe(false);
  });
});
