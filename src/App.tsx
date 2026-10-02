import React, { useEffect, useState } from "react";

import {
  GearSix,
  DotsSix,
  ArrowClockwise,
  PushPin,
  X,
  Plus,
  Trash,
  GasPump,
  WarningCircle,
} from "@phosphor-icons/react";
import { CircularProgressbar, buildStyles } from "react-circular-progressbar";
import "react-circular-progressbar/dist/styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen } from "@tauri-apps/api/event";
import { command, desktop, demo, initial } from "./bridge";
import { countdown, percent } from "./format";
import type { Account, Config, Limit, Provider } from "./types";
import "./style.css";
const names = {
  claude: "Claude",
  openai: "OpenAI",
  gemini: "Gemini",
  grok: "Grok",
};
export default function App() {
  const [config, setConfig] = useState<Config>(initial),
    [settings, setSettings] = useState(false),
    [expanded, setExpanded] = useState<string | null>(null),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [now, setNow] = useState(Date.now());
  useEffect(() => {
    command<Config>("read_config")
      .then(setConfig)
      .catch((e) => setError(String(e)));
    const timer = setInterval(() => setNow(Date.now()), 30000);
    let dispose: (() => void) | undefined;
    if (desktop)
      listen<Config>("usage-updated", (e) => setConfig(e.payload)).then(
        (f) => (dispose = f),
      );
    return () => {
      clearInterval(timer);
      dispose?.();
    };
  }, []);
  async function run(name: string, args: Record<string, unknown> = {}) {
    setError("");
    setBusy(true);
    try {
      await command(name, args);
      setConfig({ ...(await command<Config>("read_config")) });
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setBusy(false);
    }
  }
  async function preference(change: Partial<Config["settings"]>) {
    await run("save_settings", { settings: { ...config.settings, ...change } });
  }
  // Content expansion changes only the local widget size; provider windows never receive this capability.
  useEffect(() => {
    if (!desktop) return;
    const dock = document.querySelector("main")!;
    const observer = new ResizeObserver(() => {
      getCurrentWindow()
        .setSize(
          new LogicalSize(
            window.innerWidth,
            Math.min(950, Math.ceil(dock.getBoundingClientRect().height) + 28),
          ),
        )
        .catch(() => {});
    });
    observer.observe(dock);
    return () => observer.disconnect();
  }, []);
  async function drag() {
    if (!desktop) return;
    await getCurrentWindow().startDragging();
    await command("snap_window");
  }
  const latest = Object.values(config.cached)
    .map((s) => (s.fetchedAt ? Date.parse(s.fetchedAt) : 0))
    .filter(Boolean);
  return (
    <main
      className={`dock ${config.settings.opaque ? "opaque" : ""} view-${config.settings.view}`}
      data-theme={config.settings.theme}
    >
      <header>
        <div className="brand">
          <GasPump weight="duotone" size={19} />
          <span>TokenFuel</span>
        </div>
        <div className="switch" aria-label="Widget view">
          {(["bars", "rings"] as const).map((view) => (
            <button
              key={view}
              aria-pressed={config.settings.view === view}
              className={config.settings.view === view ? "active" : ""}
              onClick={() => preference({ view })}
            >
              {view === "bars" ? "Bars" : "Rings"}
            </button>
          ))}
        </div>
        <div className="tools">
          <button
            aria-label="Always on top"
            aria-pressed={config.settings.alwaysOnTop}
            onClick={() =>
              preference({ alwaysOnTop: !config.settings.alwaysOnTop })
            }
          >
            <PushPin
              weight={config.settings.alwaysOnTop ? "fill" : "regular"}
            />
          </button>
          <button
            aria-label="Settings"
            onClick={() => {
              setSettings(!settings);
              setExpanded(null);
            }}
          >
            <GearSix />
          </button>
          <button aria-label="Drag widget" onPointerDown={drag}>
            <DotsSix />
          </button>
        </div>
      </header>
      {(demo || !desktop) && (
        <div className="preview-label">
          {demo
            ? "Design preview · sample data"
            : "Browser preview · connections require the Windows app"}
        </div>
      )}
      <section className="accounts" aria-label="Remaining allowances">
        {config.accounts.map((a) => {
          const snapshot = config.cached[a.id];
          const q =
            snapshot?.limits.find((q) => q.id === a.pinnedLimit) ||
            snapshot?.limits[0];
          const p = q?.remainingPercent ?? null;
          const state = snapshot?.status || "disconnected";
          const warning = p !== null && p < 20;
          return (
            <article
              key={a.id}
              className={`provider ${a.provider} ${p !== null && p <= 50 ? "deeper" : ""} ${warning ? "low" : ""}`}
            >
              <button
                className="account-heading"
                aria-expanded={expanded === a.id}
                onClick={() => {
                  setExpanded(expanded === a.id ? null : a.id);
                  setSettings(false);
                }}
              >
                <img src={`/providers/${a.provider}.svg`} alt="" />
                <span>
                  <strong>{names[a.provider]}</strong>
                  <small>
                    {a.label}
                    {a.workspace ? ` · ${a.workspace}` : ""}
                    {q
                      ? ` · ${q.product && q.product !== "Manual snapshot" ? q.product + " · " : ""}${q.name}`
                      : ""}
                  </small>
                </span>
              </button>
              <div className="quota-name">
                {q
                  ? `${q.product && q.product !== "Manual snapshot" ? q.product + " · " : ""}${q.name}`
                  : "Allowance not connected"}
              </div>
              {config.settings.view === "rings" ? (
                <div className="ring-row">
                  <div className="ring">
                    <CircularProgressbar
                      value={p ?? 0}
                      text={q?.unlimited ? "∞" : percent(p)}
                      strokeWidth={10}
                      styles={buildStyles({
                        pathColor: "var(--accent)",
                        trailColor: "var(--track)",
                        textColor: "var(--text)",
                        textSize: "24px",
                        pathTransitionDuration: 0.2,
                      })}
                    />
                  </div>
                  <span>
                    <strong>
                      {q?.unlimited
                        ? "Unlimited"
                        : p === null
                          ? "No reading"
                          : "remaining"}
                    </strong>
                    <small>{countdown(q?.resetsAt ?? null, now)}</small>
                  </span>
                </div>
              ) : (
                <>
                  <>
                    {p !== null ? (
                      <progress
                        max={100}
                        value={p}
                        aria-label={`${names[a.provider]} remaining ${percent(p)}`}
                      />
                    ) : (
                      <div className="unknown-track" aria-hidden="true" />
                    )}
                  </>
                  <div className="remaining">
                    <strong>
                      {q?.unlimited
                        ? "Unlimited"
                        : p === null
                          ? "—"
                          : percent(p)}{" "}
                      <span>{p !== null ? "remaining" : ""}</span>
                    </strong>
                    <small>{countdown(q?.resetsAt ?? null, now)}</small>
                  </div>
                </>
              )}
              <div className="source-line">
                {warning ? (
                  <span>
                    <WarningCircle /> Low remaining
                  </span>
                ) : q?.remaining ? (
                  `${q.remaining} ${q.unit} left`
                ) : q?.source === "manual" ? (
                  "Manual snapshot"
                ) : q?.source === "experimental" ? (
                  "Experimental snapshot"
                ) : q ? (
                  "Documented source"
                ) : (
                  <button
                    onClick={() => {
                      setSettings(true);
                      setExpanded(null);
                    }}
                  >
                    {state === "disconnected"
                      ? "Connect account"
                      : state === "loginRequired"
                        ? "Sign-in required"
                        : state === "rateLimited"
                          ? "Waiting to retry"
                          : state === "offline"
                            ? "Offline"
                            : "Reading unavailable"}
                  </button>
                )}
                {state !== "available" && q && (
                  <span className="status">{state} · cached</span>
                )}
              </div>
            </article>
          );
        })}
      </section>
      {expanded && (
        <section className="details">
          {(() => {
            const a = config.accounts.find((a) => a.id === expanded)!;
            const s = config.cached[a.id];
            return (
              <>
                <div className="section-title">
                  <strong>All limits · {names[a.provider]}</strong>
                  <button
                    aria-label="Close details"
                    onClick={() => setExpanded(null)}
                  >
                    <X />
                  </button>
                </div>
                {s?.identity && (
                  <p>
                    Signed in as {s.identity.displayName}
                    {s.identity.plan ? ` · ${s.identity.plan}` : ""}
                  </p>
                )}
                <p>
                  {s?.message ||
                    "Connect this account in settings to read its allowance."}
                </p>
                {s?.limits.map((q) => (
                  <div className="limit-row" key={q.id}>
                    <div>
                      <strong>{q.name}</strong>
                      <small>
                        {q.product} · {q.scope} · {q.period} · {q.source}
                      </small>
                      {q.used !== null && (
                        <small>
                          {q.used} {q.unit} used
                          {q.total !== null
                            ? ` / ${q.total} ${q.unit}`
                            : " · limit not reported"}
                        </small>
                      )}
                    </div>
                    <span>
                      {q.unlimited
                        ? "Unlimited"
                        : q.remainingPercent === null
                          ? q.remaining
                            ? `${q.remaining} ${q.unit}`
                            : "Limit not reported"
                          : percent(q.remainingPercent)}
                    </span>
                    <button
                      aria-label={`Pin ${q.name}`}
                      aria-pressed={a.pinnedLimit === q.id}
                      onClick={() =>
                        run("save_account", {
                          account: { ...a, pinnedLimit: q.id },
                        })
                      }
                    >
                      <PushPin
                        weight={a.pinnedLimit === q.id ? "fill" : "regular"}
                      />
                    </button>
                  </div>
                ))}
                {s?.retryAt && (
                  <small>
                    Next attempt: {new Date(s.retryAt).toLocaleTimeString()}
                  </small>
                )}
              </>
            );
          })()}
        </section>
      )}
      {settings && (
        <section className="settings">
          <div className="section-title">
            <strong>Settings</strong>
            <button
              aria-label="Close settings"
              onClick={() => setSettings(false)}
            >
              <X />
            </button>
          </div>
          <div className="preferences">
            <label>
              Theme
              <select
                value={config.settings.theme}
                onChange={(e) =>
                  preference({
                    theme: e.target.value as Config["settings"]["theme"],
                  })
                }
              >
                <option value="system">System</option>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </label>
            {(["opaque", "startup", "alerts", "snapToEdges"] as const).map(
              (key) => (
                <label key={key}>
                  <input
                    type="checkbox"
                    checked={config.settings[key]}
                    onChange={(e) => preference({ [key]: e.target.checked })}
                  />
                  {
                    {
                      opaque: "Opaque background",
                      startup: "Start with Windows",
                      alerts: "Alerts at 20% and 10%",
                      snapToEdges: "Snap to edges",
                    }[key]
                  }
                </label>
              ),
            )}
            <label>
              Poll every{" "}
              <select
                value={config.settings.intervalSecs}
                onChange={(e) =>
                  preference({ intervalSecs: Number(e.target.value) })
                }
              >
                <option value={120}>2 minutes</option>
                <option value={300}>5 minutes</option>
                <option value={600}>10 minutes</option>
              </select>
            </label>
          </div>
          <p className="hint">
            Connections stay on this device. Browser sign-in windows are
            isolated and session-only. Experimental browser readings require
            explicit capture.
          </p>
          {config.accounts.map((a) => (
            <AccountEditor key={a.id} account={a} run={run} />
          ))}
          <button
            className="add"
            onClick={() =>
              run("save_account", {
                account: {
                  ...initial.accounts[0],
                  id: "new",
                  provider: "claude",
                  label: "Additional account",
                  enabled: false,
                },
              })
            }
          >
            <Plus /> Add account
          </button>
          <small className="hint">
            Grok is planned next. Claude Enterprise monthly limits require
            office verification. Ordinary ChatGPT counters and Gemini automatic
            tracking remain unverified.
          </small>
        </section>
      )}
      {error && (
        <div role="alert" className="error">
          {error}
          <button aria-label="Dismiss error" onClick={() => setError("")}>
            <X />
          </button>
        </div>
      )}
      <footer>
        <button onClick={() => run("refresh")} disabled={busy}>
          <ArrowClockwise className={busy ? "spinning" : ""} />
          {busy ? "Refreshing…" : "Refresh"}
        </button>
        <span>
          {latest.length
            ? `Updated ${Math.max(0, Math.floor((now - Math.max(...latest)) / 60000))}m ago`
            : "No readings yet"}
        </span>
        <span className="local">Local only</span>
      </footer>
    </main>
  );
}
function AccountEditor({
  account: a,
  run,
}: {
  account: Account;
  run: (n: string, args?: Record<string, unknown>) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState(a),
    [manual, setManual] = useState(false);
  useEffect(() => setDraft(a), [a.id, a.revision]);
  return (
    <div className="account-editor">
      <div className="edit-row">
        <img src={`/providers/${draft.provider}.svg`} alt="" />
        <select
          aria-label="Provider"
          value={draft.provider}
          onChange={(e) =>
            setDraft({
              ...draft,
              provider: e.target.value as Provider,
              connection:
                e.target.value === "openai"
                  ? "codexCli"
                  : e.target.value === "claude"
                    ? "claudeCli"
                    : "browser",
              enabled: false,
            })
          }
        >
          {(["claude", "openai", "gemini"] as const).map((p) => (
            <option key={p} value={p}>
              {names[p]}
            </option>
          ))}
        </select>
        <input
          aria-label="Account name"
          placeholder="Account name"
          value={draft.label}
          maxLength={80}
          onChange={(e) => setDraft({ ...draft, label: e.target.value })}
        />
        <input
          aria-label="Workspace"
          placeholder="Workspace (optional)"
          value={draft.workspace}
          maxLength={120}
          onChange={(e) => setDraft({ ...draft, workspace: e.target.value })}
        />
        <button
          aria-label="Remove account"
          onClick={() => run("remove_account", { id: a.id })}
        >
          <Trash />
        </button>
      </div>
      <div className="edit-row">
        <select
          aria-label="Connection source"
          value={draft.connection}
          onChange={(e) =>
            setDraft({
              ...draft,
              connection: e.target.value as Account["connection"],
              enabled: false,
            })
          }
        >
          {draft.provider === "openai" && (
            <option value="codexCli">Codex · documented</option>
          )}
          {draft.provider === "claude" && (
            <option value="claudeCli">Claude Code · experimental</option>
          )}
          <option value="browser">Usage view · experimental capture</option>
          <option value="manual">Manual snapshot</option>
        </select>
        {draft.connection !== "manual" && (
          <label>
            <input
              type="checkbox"
              checked={draft.enabled}
              onChange={(e) =>
                setDraft({ ...draft, enabled: e.target.checked })
              }
            />
            Allow connection{" "}
            {draft.connection.endsWith("Cli") ? "and local session access" : ""}
          </label>
        )}
        {(draft.connection === "browser" ||
          draft.connection === "claudeCli") && (
          <label>
            <input
              type="checkbox"
              checked={draft.experimental}
              onChange={(e) =>
                setDraft({ ...draft, experimental: e.target.checked })
              }
            />
            Experimental opt-in
          </label>
        )}
        <button
          className="primary"
          onClick={async () => {
            if (await run("save_account", { account: draft }))
              await run("refresh");
          }}
        >
          Save
        </button>
      </div>
      {draft.connection === "browser" && (
        <button onClick={() => run("open_provider", { id: a.id })}>
          Open isolated sign-in / Usage view
        </button>
      )}
      {draft.connection === "manual" && (
        <button onClick={() => setManual(!manual)}>
          Enter manual snapshot
        </button>
      )}
      {manual && (
        <form
          className="manual"
          onSubmit={async (e) => {
            e.preventDefault();
            const f = new FormData(e.currentTarget);
            const pct = String(f.get("percent"));
            const saved = await run("set_manual", {
              id: a.id,
              input: {
                name: f.get("name"),
                unit: f.get("unit"),
                period: f.get("period"),
                used: f.get("used"),
                total: f.get("total"),
                remainingPercent: pct === "" ? null : Number(pct),
                unlimited: f.get("unlimited") === "on",
                resetsAt: f.get("reset") || null,
              },
            });
            if (saved) await run("refresh");
          }}
        >
          <label>
            Quota name
            <input name="name" required defaultValue="Monthly budget" />
          </label>
          <label>
            Period
            <input name="period" defaultValue="monthly" required />
          </label>
          <label>
            Unit
            <input name="unit" defaultValue="USD" required />
          </label>
          <label>
            Used
            <input name="used" inputMode="decimal" defaultValue="0" />
          </label>
          <label>
            Limit
            <input
              name="total"
              inputMode="decimal"
              placeholder="Unknown if empty"
            />
          </label>
          <label>
            Or remaining %
            <input name="percent" type="number" min="0" max="100" step="any" />
          </label>
          <label>
            Reset (with timezone)
            <input name="reset" placeholder="2026-10-17T00:00:00Z" />
          </label>
          <label>
            <input name="unlimited" type="checkbox" />
            Provider explicitly says unlimited
          </label>
          <button className="primary">Save manual snapshot</button>
        </form>
      )}
    </div>
  );
}
