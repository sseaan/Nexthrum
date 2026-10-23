import { describe, expect, it } from "vitest";
import { formatDate, formatFullDate, truncate } from "./format";

const now = new Date(2026, 9, 23, 9, 0, 0); // 2026-10-23 09:00 local

function localIso(year: number, month: number, day: number, hour: number, minute: number) {
  return new Date(year, month - 1, day, hour, minute).toISOString();
}

describe("formatDate", () => {
  it("shows only the time for messages from today", () => {
    expect(formatDate(localIso(2026, 10, 23, 14, 5), now)).toBe("14:05");
  });

  it("shows month-day and time within the same year", () => {
    expect(formatDate(localIso(2026, 3, 4, 16, 45), now)).toBe("03-04 16:45");
  });

  it("shows the full date for older messages", () => {
    expect(formatDate(localIso(2024, 12, 31, 23, 59), now)).toBe("2024-12-31");
  });

  it("falls back to the raw value when the timestamp cannot be parsed", () => {
    expect(formatDate("not-a-date", now)).toBe("not-a-date");
    expect(formatDate("", now)).toBe("");
  });
});

describe("formatFullDate", () => {
  it("includes date and time components", () => {
    // Pin a 24-hour locale so the assertion does not depend on the host locale.
    const value = formatFullDate(localIso(2026, 10, 23, 14, 5), "en-GB");
    expect(value).toContain("2026");
    expect(value).toContain("14:05");
  });

  it("returns the raw string for invalid input", () => {
    expect(formatFullDate("nope")).toBe("nope");
  });
});

describe("truncate", () => {
  it("keeps short text untouched and clips long text", () => {
    expect(truncate("short", 10)).toBe("short");
    expect(truncate("0123456789abc", 10)).toBe("0123456789...");
  });
});
