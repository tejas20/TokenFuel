import type { Limit, Snapshot } from "./types";
export function selectLimit(
  snapshot: Snapshot | undefined,
  pinned: string | null,
) {
  return pinned
    ? snapshot?.limits.find((q) => q.id === pinned)
    : snapshot?.limits[0];
}

// Freshness is per quota, not the most recently updated account in the footer.
export function isStale(
  q: Limit,
  now: number,
  intervalSecs: number,
  captured = false,
): boolean {
  const observed = Date.parse(q.observedAt);
  const maxAge =
    q.source === "manual"
      ? 86400000
      : captured
        ? 600000
        : Math.max(300000, intervalSecs * 2000);
  const reset = q.resetsAt ? Date.parse(q.resetsAt) : NaN;
  return (
    !Number.isFinite(observed) ||
    now < observed - 60000 ||
    now - observed >= maxAge ||
    (Number.isFinite(reset) && reset <= now)
  );
}

export function displayStatus(
  snapshot: Snapshot | undefined,
  q: Limit | undefined,
  now: number,
  intervalSecs: number,
  captured = false,
) {
  const status = snapshot?.status ?? "disconnected";
  return status === "available" && q && isStale(q, now, intervalSecs, captured)
    ? "stale"
    : status;
}
