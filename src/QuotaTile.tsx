import { PushPin } from "@phosphor-icons/react";
import { CircularProgressbar, buildStyles } from "react-circular-progressbar";
import { countdown, percent } from "./format";
import { displayStatus } from "./freshness";
import { accountName, quotaLabel, visibleLimits } from "./widget";
import type { Account, Limit, Settings, Snapshot } from "./types";

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
  const limits = a.enabled ? visibleLimits(snapshot, a.pinnedLimit) : [];
  const identity = [a.label, a.workspace].filter(Boolean).join(" · ");
  const missingPin =
    a.enabled && a.pinnedLimit && !limits.some((q) => q.id === a.pinnedLimit);
  return (
    <button
      className={`quota-tile provider ${a.provider}`}
      aria-label={`${accountName(a)} · ${identity} · All remaining allowances`}
      aria-expanded={expanded}
      onClick={onClick}
      title={`${accountName(a)} · ${identity}\nClick for connection details and quota pinning`}
    >
      <span className="tile-heading">
        <img src={`/providers/${a.provider}.svg`} alt="" />
        <strong>{accountName(a)}</strong>
      </span>
      <span className="tile-identity" title={identity}>
        {identity}
      </span>
      <span className="tile-limits">
        {(limits.length ? limits : [undefined]).map((q) => (
          <QuotaWindow
            key={q?.id ?? "unavailable"}
            account={a}
            snapshot={snapshot}
            quota={q}
            settings={settings}
            now={now}
          />
        ))}
      </span>
      {missingPin && (
        <span className="tile-caption attention">Pinned limit unavailable</span>
      )}
    </button>
  );
}

function QuotaWindow({
  account: a,
  snapshot,
  quota: q,
  settings,
  now,
}: {
  account: Account;
  snapshot?: Snapshot;
  quota?: Limit;
  settings: Settings;
  now: number;
}) {
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
  const status =
    state !== "available"
      ? `${states[state] || "Unavailable"}${q ? " · cached" : ""}`
      : !q
        ? "Unavailable"
        : `${p !== null && p < 20 ? "Low remaining · " : ""}${q.source === "manual" ? "Manual snapshot" : q.source === "experimental" ? "Experimental" : "Checked"}`;
  const balance =
    q?.remaining !== null && q?.remaining !== undefined
      ? `${q.remaining} ${q.unit} left`
      : null;
  const spending =
    q?.used !== null && q?.used !== undefined
      ? `${q.used} ${q.unit} used`
      : null;
  const amount = q?.unlimited
    ? "Unlimited"
    : p !== null
      ? `${value} remaining${balance ? ` · ${balance}` : ""}`
      : balance
        ? balance
        : spending
          ? `${spending} (uncapped)`
          : "Limit not reported";
  const label = q ? quotaLabel(q) : "Allowance";
  const caption =
    state !== "available" || !q
      ? status
      : `${q.source === "manual" ? "Manual · " : q.source === "experimental" ? "Experimental · " : ""}${p !== null && p < 20 ? "Low · " : ""}${reset.replace("Resets in ", "Reset ")}`;
  return (
    <span
      className={`quota-window provider ${a.provider} ${p !== null && p <= 50 ? "deeper" : ""} ${p !== null && p < 20 ? "low" : ""} ${state !== "available" ? "unverified" : ""}`}
      aria-label={`${label} · ${amount} · ${status} · ${reset}`}
      title={`${q?.product ? `${q.product} · ` : ""}${label} · ${q?.scope || "account"}\n${amount}\n${reset}\n${status} · ${Number.isFinite(age) ? age : "?"}m ago`}
    >
      <span className="tile-quota">
        {q && a.pinnedLimit === q.id && (
          <PushPin weight="fill" aria-label="Pinned quota" />
        )}
        {label}
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
              aria-label={`${accountName(a)} ${label} ${value} remaining`}
            />
          ) : (
            <span className="unknown-track" aria-hidden="true" />
          )}
          <span className="tile-value">{value}</span>
        </span>
      )}
      {!q?.unlimited && balance && (
        <span className="tile-amount">{balance}</span>
      )}
      {!q?.unlimited && !balance && spending && (
        <span className="tile-amount">{spending} (uncapped)</span>
      )}
      {q?.unlimited && <span className="tile-amount">Unlimited</span>}
      <span
        className={`tile-caption ${state !== "available" || (p !== null && p < 20) ? "attention" : ""}`}
      >
        {caption}
      </span>
    </span>
  );
}
