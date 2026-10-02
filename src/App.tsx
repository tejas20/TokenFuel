import React, { useEffect, useState } from "react";

import {
  GearSix,
  DotsSix,
  ArrowClockwise,
  PushPin,
  X,
  Plus,
  Trash,
  Circle,
  ChartBar,
} from "@phosphor-icons/react";
import "react-circular-progressbar/dist/styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen } from "@tauri-apps/api/event";
import { command, desktop, demo, initial } from "./bridge";
import { countdown, percent } from "./format";
import { QuotaTile } from "./QuotaTile";
import { accountName, visibleAccounts, widgetWidth } from "./widget";
import type { Account, Config, Limit, Provider } from "./types";
import "./style.css";
import "./compact.css";
import { getVersion } from "@tauri-apps/api/app";
const names: Record<Provider, string> = {
  claude: "Claude",
  openai: "OpenAI",
  gemini: "Gemini",
  grok: "Grok",
  opencode: "OpenCode Go",
  cursor: "Cursor",
  copilot: "Copilot",
  antigravity: "Antigravity",
  unknown: "Unknown",
};
const connectionHints: Record<Account["connection"], string> = {
  codexCli:
    "Automatically finds the installed Codex app or CLI and uses its current sign-in after you allow access. No token to copy. Tracks Codex allowances; ordinary ChatGPT counters are unavailable.",
  claudeCli:
    "Automatically uses the current Claude Code sign-in after you allow access. No token to copy. Experimental; availability depends on your plan.",
  cursorLocal:
    "Automatically finds the current Cursor sign-in after you allow access. No token to copy. Experimental; sign in to Cursor first.",
  copilotCli:
    "Automatically looks for an existing Copilot CLI or GitHub CLI sign-in after you allow access. No token to copy. Live account verification is still needed.",
  grokCli:
    "Automatically looks for an existing Grok CLI sign-in after you allow access. Tracks Grok Build credits; ordinary Grok chat allowances are unavailable.",
  antigravityLocal:
    "Automatically looks for an existing Antigravity sign-in after you allow access. Experimental; live account verification is still needed.",
  opencodeGo:
    "Uses an existing OpenCode Go monitor configuration if available. Otherwise enter the workspace ID and session cookie in Advanced connection settings. Experimental.",
  geminiWeb:
    "Save both permissions, then open the isolated Usage window and sign in. Keep it open for automatic refresh. Sign-in is temporary; some SSO accounts need the manual option.",
  browser:
    "Save both permissions, then open the isolated Usage window and sign in. Refresh reads a snapshot of visible usage. Sign-in is temporary; automatic tracking is unavailable with this source.",
  manual:
    "Enter the allowance yourself. This snapshot does not update automatically.",
  unknown: "Choose a supported connection source.",
};
const secretConnections: Account["connection"][] = [
  "opencodeGo",
  "cursorLocal",
  "grokCli",
  "copilotCli",
  "antigravityLocal",
];
export default function App() {
  const [config, setConfig] = useState<Config>(initial),
    [version, setVersion] = useState(""),
    [settings, setSettings] = useState(false),
    [expanded, setExpanded] = useState<string | null>(null),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [now, setNow] = useState(Date.now());
  useEffect(() => {
    command<Config>("read_config")
      .then((loaded) => {
        setConfig(loaded);
        if (!demo && !loaded.accounts.some((account) => account.enabled))
          setSettings(true);
      })
      .catch((e) => setError(String(e)));
    if (desktop)
      getVersion()
        .then(setVersion)
        .catch(() => {});
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
  const accounts = visibleAccounts(config.accounts);
  const panel = settings || expanded !== null;
  const width = widgetWidth(accounts.length, config.settings.view, panel);
  // Observe content dimensions, not the current native width: closing a panel
  // must return to the selected compact footprint without a resize feedback loop.
  useEffect(() => {
    if (!desktop) return;
    const dock = document.querySelector("main")!;
    const observer = new ResizeObserver(() => {
      const rect = dock.getBoundingClientRect();
      getCurrentWindow()
        .setSize(
          new LogicalSize(
            Math.ceil(rect.width) + 12,
            Math.min(850, Math.ceil(rect.height) + 12),
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
  return (
    <main
      className={`dock compact ${config.settings.opaque ? "opaque" : ""} view-${config.settings.view} ${panel ? "panel-open" : ""}`}
      style={{ width }}
      data-theme={config.settings.theme}
      data-desktop={desktop}
    >
      <div className="compact-surface">
        <section
          className="accounts"
          aria-label="Remaining allowances"
          style={{
            gridTemplateColumns: `repeat(${Math.min(3, accounts.length)}, minmax(0, 1fr))`,
          }}
        >
          {accounts.map((a) => (
            <QuotaTile
              key={a.id}
              account={a}
              snapshot={config.cached[a.id]}
              settings={config.settings}
              now={now}
              expanded={expanded === a.id}
              onClick={() => {
                if (
                  !a.enabled ||
                  !config.accounts.some((saved) => saved.id === a.id)
                ) {
                  setSettings(true);
                  setExpanded(null);
                } else {
                  setExpanded(expanded === a.id ? null : a.id);
                  setSettings(false);
                }
              }}
            />
          ))}
        </section>
        <div className="compact-tools" aria-label="TokenFuel controls">
          <button
            aria-label="Refresh"
            title="Refresh usage"
            disabled={busy}
            onClick={() => run("refresh")}
          >
            <ArrowClockwise className={busy ? "spinning" : ""} />
          </button>
          <button
            aria-label={
              config.settings.view === "bars"
                ? "Show ring view"
                : "Show bar view"
            }
            title="Switch bars / rings"
            onClick={() =>
              preference({
                view: config.settings.view === "bars" ? "rings" : "bars",
              })
            }
          >
            {config.settings.view === "bars" ? <Circle /> : <ChartBar />}
          </button>
          <button
            aria-label="Settings"
            aria-expanded={settings}
            title="Settings"
            onClick={() => {
              setSettings(!settings);
              setExpanded(null);
            }}
          >
            <GearSix />
          </button>
          <button
            aria-label="Drag widget"
            title="Drag widget"
            onPointerDown={drag}
          >
            <DotsSix />
          </button>
        </div>
      </div>
      {(demo || !desktop) && (
        <div className="preview-label">
          {demo
            ? "Sample data"
            : "Browser preview · use Windows app to connect"}
        </div>
      )}
      {expanded && (
        <section className="details">
          {(() => {
            const a = config.accounts.find((a) => a.id === expanded)!;
            const s = config.cached[a.id];
            return (
              <>
                <div className="section-title">
                  <strong>All limits · {accountName(a)}</strong>
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
                  {a.label}
                  {a.workspace ? ` · ${a.workspace}` : ""}
                </p>
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
                      <small>
                        {q.resetsAt
                          ? countdown(q.resetsAt, now)
                          : q.resetLabel || "Reset not reported"}{" "}
                        · {q.source === "manual" ? "Snapshot" : "Checked"}{" "}
                        {Math.max(
                          0,
                          Math.floor((now - Date.parse(q.observedAt)) / 60000),
                        )}
                        m ago
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
            <strong>Settings{version && ` · v${version}`}</strong>
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
            {(
              [
                "opaque",
                "startup",
                "alerts",
                "snapToEdges",
                "alwaysOnTop",
              ] as const
            ).map((key) => (
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
                    alwaysOnTop: "Always on top",
                  }[key]
                }
              </label>
            ))}
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
            Only enabled connections appear in the widget. If none are enabled,
            one account stays visible to help you connect. Temporary failures
            keep the account visible.
          </p>
          <p className="hint">
            Choose your provider, allow access, and Save. Supported connections
            find your existing sign-in automatically; you usually do not need a
            token. Connections stay on this device. Browser sign-in windows are
            isolated and session-only. Gemini live view reloads its Usage page;
            the capture source requires explicit refresh.
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
            New providers are provisional until verified with your live account.
            Claude Enterprise monthly limits require office verification.
            Ordinary ChatGPT counters remain unsupported; Gemini live polling
            needs isolated-window verification.
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
    </main>
  );
}
export function AccountEditor({
  account: a,
  run,
}: {
  account: Account;
  run: (n: string, args?: Record<string, unknown>) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState(a),
    [manual, setManual] = useState(false),
    [secretInput, setSecretInput] = useState("");
  useEffect(() => setDraft(a), [a.id, a.revision]);
  useEffect(() => setSecretInput(""), [draft.provider, draft.connection]);
  return (
    <div className="account-editor">
      <div className="edit-row">
        <img src={`/providers/${draft.provider}.svg`} alt="" />
        <select
          aria-label="Provider"
          value={draft.provider}
          onChange={(e) => {
            const prov = e.target.value as Provider;
            setDraft({
              ...draft,
              provider: prov,
              connection:
                prov === "openai"
                  ? "codexCli"
                  : prov === "claude"
                    ? "claudeCli"
                    : prov === "opencode"
                      ? "opencodeGo"
                      : prov === "cursor"
                        ? "cursorLocal"
                        : prov === "grok"
                          ? "grokCli"
                          : prov === "copilot"
                            ? "copilotCli"
                            : prov === "antigravity"
                              ? "antigravityLocal"
                              : "browser",
              enabled: false,
            });
          }}
        >
          {(
            [
              "claude",
              "openai",
              "gemini",
              "grok",
              "opencode",
              "cursor",
              "copilot",
              "antigravity",
            ] as const
          ).map((p) => (
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
          {draft.provider === "gemini" && (
            <option value="geminiWeb">
              Gemini live Usage view · experimental
            </option>
          )}
          {draft.provider === "opencode" && (
            <option value="opencodeGo">
              OpenCode Go status · experimental
            </option>
          )}
          {draft.provider === "cursor" && (
            <option value="cursorLocal">
              Cursor local session · experimental
            </option>
          )}
          {draft.provider === "grok" && (
            <option value="grokCli">Grok CLI auth · experimental</option>
          )}
          {draft.provider === "copilot" && (
            <option value="copilotCli">GitHub Copilot · documented</option>
          )}
          {draft.provider === "antigravity" && (
            <option value="antigravityLocal">
              Google Antigravity · experimental
            </option>
          )}
          {["openai", "claude", "gemini"].includes(draft.provider) && (
            <option value="browser">Usage view · experimental capture</option>
          )}
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
            {draft.connection.endsWith("Cli") ||
            draft.connection.endsWith("Local")
              ? "and local session access"
              : ""}
          </label>
        )}
        {(draft.connection === "browser" ||
          draft.connection === "geminiWeb" ||
          draft.connection === "claudeCli" ||
          draft.connection === "opencodeGo" ||
          draft.connection === "cursorLocal" ||
          draft.connection === "grokCli" ||
          draft.connection === "antigravityLocal") && (
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
      <p className="hint">{connectionHints[draft.connection]}</p>
      <details className="advanced-connection">
        <summary>Advanced connection settings</summary>
        <div className="edit-row">
          <input
            aria-label="Workspace"
            placeholder="Workspace ID (optional)"
            value={draft.workspace}
            maxLength={120}
            onChange={(e) => setDraft({ ...draft, workspace: e.target.value })}
          />
          {draft.connection === "codexCli" && (
            <input
              aria-label="Codex executable path"
              placeholder="Codex executable path (auto-detected)"
              value={draft.cliPath ?? ""}
              onChange={(e) =>
                setDraft({ ...draft, cliPath: e.target.value || null })
              }
            />
          )}
        </div>
        {secretConnections.includes(draft.connection) &&
          draft.provider === a.provider &&
          draft.connection === a.connection && (
            <>
              <p className="hint">
                Use an account-specific secret only if automatic sign-in is
                unavailable or you need a separate account. Saved secrets stay
                in Windows Credential Manager. Save the account before adding a
                secret.
              </p>

              <div className="edit-row">
                {draft.hasCustomSecret ? (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      fontSize: "0.85em",
                    }}
                  >
                    <span>🔒 Secret saved in Credential Manager</span>
                    <button
                      type="button"
                      onClick={async () => {
                        if (await run("clear_account_secret", { id: a.id })) {
                          setDraft({ ...draft, hasCustomSecret: false });
                        }
                      }}
                    >
                      Clear Secret
                    </button>
                  </div>
                ) : (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      width: "100%",
                    }}
                  >
                    <input
                      type="password"
                      aria-label="Account secret or token"
                      placeholder="API Token, Auth Cookie, or JSON"
                      value={secretInput}
                      onChange={(e) => setSecretInput(e.target.value)}
                      style={{ flex: 1 }}
                    />
                    <button
                      type="button"
                      disabled={!secretInput.trim()}
                      onClick={async () => {
                        if (
                          await run("save_account_secret", {
                            id: a.id,
                            secret: secretInput.trim(),
                          })
                        ) {
                          setSecretInput("");
                          setDraft({ ...draft, hasCustomSecret: true });
                        }
                      }}
                    >
                      Save Secret
                    </button>
                  </div>
                )}
              </div>
            </>
          )}
      </details>
      {(draft.connection === "browser" || draft.connection === "geminiWeb") && (
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
