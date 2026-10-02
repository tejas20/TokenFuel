import { expect, test } from "vitest";
import { isStale, displayStatus, selectLimit } from "./freshness";
import type { Limit, Snapshot } from "./types";
const now = Date.parse("2026-10-02T12:00:00Z");
test("missing pinned quota never silently substitutes a different pool", () => {
  const snapshot = { limits: [quota()] } as Snapshot;
  expect(selectLimit(snapshot, "removed-pool")).toBeUndefined();
  expect(selectLimit(snapshot, "x")?.id).toBe("x");
  expect(selectLimit(snapshot, null)?.id).toBe("x");
});
const quota = (change: Partial<Limit> = {}): Limit => ({
  id: "x",
  name: "Weekly",
  product: "Codex",
  scope: "account",
  unit: "percent",
  period: "weekly",
  resetsAt: null,
  used: null,
  total: null,
  remaining: null,
  remainingPercent: 87,
  unlimited: false,
  source: "documented",
  observedAt: new Date(now - 120000).toISOString(),
  ...change,
});
test("per-account freshness rejects old counters, expired resets and invalid dates", () => {
  expect(isStale(quota(), now, 120)).toBe(false);
  expect(
    isStale(
      quota({ observedAt: new Date(now - 300000).toISOString() }),
      now,
      120,
    ),
  ).toBe(true);
  expect(
    isStale(quota({ resetsAt: new Date(now).toISOString() }), now, 120),
  ).toBe(true);
  expect(isStale(quota({ observedAt: "bad" }), now, 120)).toBe(true);
});
test("capture and manual freshness does not mask connection errors", () => {
  const q = quota({
    source: "manual",
    observedAt: new Date(now - 3600000).toISOString(),
  });
  expect(isStale(q, now, 120)).toBe(false);
  expect(isStale({ ...q, source: "experimental" }, now, 120, true)).toBe(true);
  expect(
    displayStatus(
      { status: "offline" } as Snapshot,
      { ...q, resetsAt: new Date(now).toISOString() },
      now,
      120,
    ),
  ).toBe("offline");
});
