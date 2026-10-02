import { CircularProgressbar, buildStyles } from "react-circular-progressbar";
import { countdown, percent } from "./format";
import { displayStatus, selectLimit } from "./freshness";
import { accountName } from "./widget";
import type { Account, Settings, Snapshot } from "./types";

const states: Record<string, string> = {
  disconnected: "Connect",
  loginRequired: "Sign in",
  offline: "Offline",
  stale: "Stale",
  rateLimited: "Waiting to retry",
  unavailable: "Unavailable",
};

export function QuotaTile({
  account: a,
  snapshot,
  settings,
  now,
  expanded,
  onClick,
}: {
  account: Account;
  snapshot?: Snapshot;
  settings: Settings;
  now: number;
  expanded: boolean;
  onClick: () => void;
}) {
  const q = a.enabled ? selectLimit(snapshot, a.pinnedLimit) : undefined;
  const p = q?.remainingPercent ?? null;
  const state = a.enabled
    ? displayStatus(
        snapshot,
        q,
        now,
        settings.intervalSecs,
        a.connection === "browser",
      )
    : "disconnected";
  const reset = q?.resetsAt
    ? countdown(q.resetsAt, now)
    : q?.resetLabel || "Reset not reported";
  const value = q?.unlimited ? "∞" : percent(p);
  const age = q
    ? Math.max(0, Math.floor((now - Date.parse(q.observedAt)) / 60000))
    : null;
  const source =
    q?.source === "manual"
      ? "Manual"
      : q?.source === "experimental"
        ? "Experimental"
        : "Checked";
  const status =
    state !== "available"
      ? `${states[state] || "Unavailable"}${q ? " · cached" : ""}`
      : !q
        ? "Pinned limit unavailable"
        : p !== null && p < 20
          ? q?.source === "manual"
            ? "Low · manual"
            : "Low remaining"
          : q.source === "manual"
            ? "Manual snapshot"
            : `${source} ${Number.isFinite(age) ? age : "?"}m ago`;
  const identity = [a.label, a.workspace].filter(Boolean).join(" · ");
  const amount = q?.unlimited
    ? "Unlimited"
    : p !== null
      ? `${value} remaining`
      : q?.remaining !== null && q?.remaining !== undefined
        ? `${q.remaining} ${q.unit} left`
        : "No reading";
  return (
    <button
      className={`quota-tile provider ${a.provider} ${p !== null && p <= 50 ? "deeper" : ""} ${p !== null && p < 20 ? "low" : ""} ${state !== "available" ? "unverified" : ""}`}
      aria-label={`${accountName(a)} · ${identity} · ${q?.name || "Allowance"} · ${amount} · ${status}`}
      aria-expanded={expanded}
      onClick={onClick}
      title={`${accountName(a)} · ${identity}\n${q?.name || "Allowance not connected"} · ${amount}\n${reset}\n${status}${q?.source === "experimental" ? " · experimental source" : ""}`}
    >
      <span className="tile-heading">
        <img src={`/providers/${a.provider}.svg`} alt="" />
        <strong>{accountName(a)}</strong>
      </span>
      {settings.view === "rings" ? (
        <span className="mini-ring">
          <CircularProgressbar
            value={p ?? 0}
            text={value}
            strokeWidth={9}
            styles={buildStyles({
              pathColor: "var(--accent)",
              trailColor: "var(--track)",
              textColor: "var(--text)",
              textSize: "25px",
              pathTransitionDuration: 0.2,
            })}
          />
        </span>
      ) : (
        <span className="tile-meter">
          {p !== null ? (
            <progress
              max={100}
              value={p}
              aria-label={`${accountName(a)} ${value} remaining`}
            />
          ) : (
            <span className="unknown-track" aria-hidden="true" />
          )}
          <span className="tile-value">{value}</span>
        </span>
      )}
      <span className="tile-quota">{q?.name || identity || "Allowance"}</span>
      <span
        className={`tile-caption ${state !== "available" || (p !== null && p < 20) ? "attention" : ""}`}
      >
        {state !== "available" ||
        !q ||
        (p !== null && p < 20) ||
        q.source === "manual"
          ? status
          : q.unlimited
            ? "Unlimited"
            : p === null
              ? amount
              : reset.replace("Resets in ", "Reset ")}
      </span>
    </button>
  );
}
