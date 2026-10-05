import React, { useEffect, useRef, useState } from "react";

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
  DotsThree,
  CaretDown,
  SquaresFour,
} from "@phosphor-icons/react";
import "react-circular-progressbar/dist/styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen } from "@tauri-apps/api/event";
import { command, desktop, demo, initial } from "./bridge";
import { QuotaTile, QuotaWindow } from "./QuotaTile";
import {
  accountName,
  focusedAccount,
  visibleAccounts,
  visibleLimits,
  widgetWidth,
} from "./widget";
import { displayStatus } from "./freshness";
import type { Account, Config, Provider } from "./types";
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
  codexCli: "Automatically finds Codex. Tracks Codex allowances only.",
  claudeCli: "Uses your Claude Code sign-in. Experimental.",
  cursorLocal: "Uses your Cursor sign-in. Experimental.",
  copilotCli: "Uses your Copilot or GitHub CLI sign-in.",
  grokCli: "Uses Grok CLI sign-in. Build credits only.",
  antigravityLocal: "Uses your Antigravity sign-in. Experimental.",
  opencodeGo:
    "Uses your Go monitor configuration. Otherwise, add details in Advanced.",
  geminiWeb:
    "Sign in below and keep the Usage window open. Sign-in lasts this session.",
  browser:
    "Sign in below, then capture visible usage. Manual refresh; session-only sign-in.",
  manual: "A snapshot you update yourself.",
  unknown: "Choose a connection source.",
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
    [menu, setMenu] = useState(false),
    [expanded, setExpanded] = useState<string | null>(null),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [now, setNow] = useState(Date.now());
  const menuButton = useRef<HTMLButtonElement>(null);
  const detailTrigger = useRef<HTMLButtonElement | null>(null);
  const dockRef = useRef<HTMLElement>(null);
  useEffect(() => {
    command<Config>("read_config")
      .then((loaded) => {
        setConfig(loaded);
      })
      .catch((e) => setError(String(e)));
    if (desktop)
      getVersion()
        .then(setVersion)
        .catch(() => {});
    const timer = setInterval(() => setNow(Date.now()), 30000);
    let dispose: (() => void) | undefined;
    let disposed = false;
    if (desktop)
      listen<Config>("usage-updated", (e) => setConfig(e.payload)).then(
        (f) => {
          if (disposed) f();
          else dispose = f;
        },
        (e) => setError(String(e)),
      );
    return () => {
      disposed = true;
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
    await run("save_settings", { settings: change });
  }
  const accounts = visibleAccounts(config.accounts);
  const focus = focusedAccount(accounts, config.settings.focusAccountId);
  const shownAccounts = focus ? [focus] : accounts;
  const detailAccount = config.accounts.find(
    (a) => a.id === expanded && a.enabled,
  );
  const hiddenLow = focus
    ? accounts.filter(
        (a) =>
          a.id !== focus.id &&
          config.cached[a.id]?.limits.some(
            (q) =>
              q.remainingPercent !== null &&
              q.remainingPercent < 20 &&
              displayStatus(
                config.cached[a.id],
                q,
                now,
                config.settings.intervalSecs,
                a.connection === "browser",
              ) === "available",
          ),
      ).length
    : 0;
  const issues = [
    ...(config.settingsIssues ?? []),
    ...config.accounts.flatMap((a) => {
      const snapshot = config.cached[a.id];
      if (!a.enabled || !snapshot) return [];
      const stale = snapshot.limits.some(
        (q) =>
          displayStatus(
            snapshot,
            q,
            now,
            config.settings.intervalSecs,
            a.connection === "browser",
          ) === "stale",
      );
      if (snapshot.status === "available" && !stale) return [];
      const message =
        snapshot.status === "available"
          ? "Cached usage is stale. Try Refresh."
          : snapshot.message || "Usage unavailable. Try Refresh.";
      return [`${names[a.provider]} · ${a.label}: ${message}`];
    }),
  ];
  const panel = settings || !!detailAccount || menu;
  const barWidth = widgetWidth(shownAccounts, config.cached, !!focus);
  const width = settings ? 540 : Math.max(barWidth, panel ? 340 : 0);
  useEffect(() => {
    if (expanded && !detailAccount) setExpanded(null);
  }, [expanded, detailAccount]);
  useEffect(() => {
    function dismiss(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      setExpanded(null);
      setMenu(false);
      setSettings(false);
      (detailAccount ? detailTrigger.current : menuButton.current)?.focus();
    }
    function outside(event: PointerEvent) {
      if (!dockRef.current?.contains(event.target as Node)) {
        setExpanded(null);
        setMenu(false);
      }
    }
    document.addEventListener("keydown", dismiss);
    document.addEventListener("pointerdown", outside);
    return () => {
      document.removeEventListener("keydown", dismiss);
      document.removeEventListener("pointerdown", outside);
    };
  }, [detailAccount]);
  async function selectFocus(id: string | null) {
    if (await run("save_settings", { settings: { focusAccountId: id } })) {
      setExpanded(null);
      setSettings(false);
      setMenu(false);
      menuButton.current?.focus();
    }
  }
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
    try {
      await getCurrentWindow().startDragging();
      await command("snap_window");
    } catch (e) {
      setError(String(e));
    }
  }
  return (
    <main
      ref={dockRef}
      className={`dock compact ${config.settings.opaque ? "opaque" : ""} view-${config.settings.view} ${focus ? "focus-mode" : ""} ${panel ? "panel-open" : ""}`}
      style={{ width }}
      data-theme={config.settings.theme}
      data-desktop={desktop}
    >
      <div
        className="compact-surface"
        style={{ width: settings ? "100%" : barWidth }}
      >
        <span
          className="drag-edge"
          title="Drag widget"
          onPointerDown={drag}
          aria-hidden="true"
        />
        {settings ? (
          <strong className="bar-title">TokenFuel</strong>
        ) : (
          focus && (
            <button
              className={`focus-selector provider ${focus.provider}`}
              aria-label={`Switch focused account · ${accountName(focus)} · ${focus.label}`}
              aria-expanded={menu}
              aria-controls="widget-menu"
              onClick={() => {
                setMenu(!menu);
                setExpanded(null);
              }}
            >
              <img src={`/providers/${focus.provider}.svg`} alt="" />
              <span>{accountName(focus)}</span>
              <CaretDown />
            </button>
          )
        )}
        <section
          className="accounts"
          aria-label="Remaining allowances"
          hidden={settings}
        >
          {shownAccounts.map((a) => (
            <QuotaTile
              key={a.id}
              account={a}
              snapshot={config.cached[a.id]}
              settings={config.settings}
              now={now}
              expanded={expanded === a.id}
              focused={!!focus}
              onClick={() => {
                detailTrigger.current =
                  document.activeElement as HTMLButtonElement;
                setMenu(false);
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
            ref={menuButton}
            aria-label={`Widget menu${hiddenLow ? ` · ${hiddenLow} other accounts running low` : ""}${issues.length ? " · connection issues" : ""}`}
            aria-expanded={menu}
            aria-controls="widget-menu"
            title={
              hiddenLow
                ? `${hiddenLow} other accounts running low`
                : issues.length
                  ? "Menu · connection issues"
                  : "Menu · refresh, focus and settings"
            }
            className={hiddenLow || issues.length ? "menu-warning" : undefined}
            onClick={() => {
              setMenu(!menu);
              setExpanded(null);
              setSettings(false);
            }}
          >
            <DotsThree weight="bold" />
            {(hiddenLow > 0 || issues.length > 0) && (
              <span className="menu-indicator">{hiddenLow || "!"}</span>
            )}
          </button>
        </div>
      </div>
      {menu && (
        <section
          id="widget-menu"
          className="widget-menu popover"
          aria-label="Widget menu"
        >
          <div className="section-title">
            <strong>TokenFuel</strong>
            <button
              aria-label="Close menu"
              onClick={() => {
                setMenu(false);
                menuButton.current?.focus();
              }}
            >
              <X />
            </button>
          </div>
          <button disabled={busy} onClick={() => run("refresh")}>
            <ArrowClockwise className={busy ? "spinning" : ""} />
            {busy ? "Refreshing…" : "Refresh usage"}
          </button>
          <button
            aria-pressed={config.settings.alwaysOnTop}
            disabled={busy}
            onClick={() =>
              preference({ alwaysOnTop: !config.settings.alwaysOnTop })
            }
          >
            <PushPin
              weight={config.settings.alwaysOnTop ? "fill" : "regular"}
            />
            Always on top{" "}
            <small>{config.settings.alwaysOnTop ? "On" : "Off"}</small>
          </button>
          <div className="menu-section">View</div>
          <button
            aria-pressed={!focus}
            disabled={busy}
            onClick={() => selectFocus(null)}
          >
            <SquaresFour />
            All accounts
          </button>
          {accounts
            .filter((a) => a.enabled)
            .map((a) => (
              <button
                key={a.id}
                className={`focus-option provider ${a.provider}`}
                disabled={busy}
                aria-pressed={focus?.id === a.id}
                onClick={() => selectFocus(a.id)}
              >
                <img src={`/providers/${a.provider}.svg`} alt="" />
                <span>
                  Focus {accountName(a)}
                  <small>
                    {[a.label, a.workspace].filter(Boolean).join(" · ")}
                  </small>
                </span>
              </button>
            ))}
          {hiddenLow > 0 && (
            <p className="attention">
              {hiddenLow} other accounts have a low allowance. Switch to All
              accounts to check them.
            </p>
          )}
          <div className="menu-section">Appearance & connections</div>
          <button
            aria-label={
              config.settings.view === "bars"
                ? "Show ring view"
                : "Show bar view"
            }
            title="Switch bars / rings"
            disabled={busy}
            onClick={() =>
              preference({
                view: config.settings.view === "bars" ? "rings" : "bars",
              })
            }
          >
            {config.settings.view === "bars" ? <Circle /> : <ChartBar />}
            {config.settings.view === "bars"
              ? "Show ring view"
              : "Show bar view"}
          </button>
          <button
            aria-label="Settings"
            aria-expanded={settings}
            title={
              issues.length
                ? "Settings · " + issues.length + " issues"
                : "Settings"
            }
            className={issues.length ? "settings-warning" : undefined}
            onClick={() => {
              setSettings(!settings);
              setExpanded(null);
              setMenu(false);
            }}
          >
            <GearSix />
            Settings
            {issues.length > 0 && (
              <span aria-label={issues.length + " settings issues"}>!</span>
            )}
          </button>
          <button
            aria-label="Drag widget"
            title="Drag widget"
            onPointerDown={drag}
          >
            <DotsSix />
            Move widget
          </button>
        </section>
      )}
      {(demo || !desktop) && (
        <div className="preview-label">
          {demo
            ? "Sample data"
            : "Browser preview · use Windows app to connect"}
        </div>
      )}
      {detailAccount && (
        <section
          className={`details popover provider ${detailAccount.provider}`}
          aria-label={`${accountName(detailAccount)} quota details`}
        >
          {(() => {
            const a = detailAccount;
            const s = config.cached[a.id];
            return (
              <>
                <div className="section-title">
                  <div className="detail-heading">
                    <img src={`/providers/${a.provider}.svg`} alt="" />
                    <strong>
                      {accountName(a)} <small>· {a.label}</small>
                    </strong>
                  </div>
                  <button
                    aria-label="Close details"
                    onClick={() => {
                      setExpanded(null);
                      detailTrigger.current?.focus();
                    }}
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
                {a.workspace && (
                  <p>
                    {a.label}
                    {a.workspace ? ` · ${a.workspace}` : ""}
                  </p>
                )}
                {s?.message && s.status !== "available" && (
                  <p role="status">
                    {s?.message ||
                      "Connect this account in settings to read its allowance."}
                  </p>
                )}
                {(!s || !s.limits.length) && (
                  <p>
                    {s?.message ||
                      "No allowance reported yet. Try Refresh or check this connection in Settings."}
                  </p>
                )}
                {visibleLimits(s, a.pinnedLimit).map((q) => (
                  <div className="limit-row" key={q.id}>
                    <div className="detail-quota-body">
                      <QuotaWindow
                        account={a}
                        snapshot={s}
                        quota={q}
                        settings={config.settings}
                        now={now}
                        detailed
                      />
                      <small>
                        {q.product} · {q.scope} · {q.source} ·{" "}
                        {Number.isFinite(Date.parse(q.observedAt))
                          ? Math.max(
                              0,
                              Math.floor(
                                (now - Date.parse(q.observedAt)) / 60000,
                              ),
                            ) + "m ago"
                          : "Age unknown"}
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
                    <button
                      aria-label={`Pin ${q.name}`}
                      aria-pressed={a.pinnedLimit === q.id}
                      disabled={busy}
                      onClick={() =>
                        run("save_account", {
                          account: {
                            ...a,
                            pinnedLimit: a.pinnedLimit === q.id ? null : q.id,
                          },
                        })
                      }
                    >
                      <PushPin
                        weight={a.pinnedLimit === q.id ? "fill" : "regular"}
                      />
                    </button>
                  </div>
                ))}
                <div className="detail-actions">
                  <button
                    className="primary"
                    disabled={busy}
                    onClick={() =>
                      selectFocus(focus?.id === a.id ? null : a.id)
                    }
                  >
                    {focus?.id === a.id
                      ? "Show all accounts"
                      : "Focus this account"}
                  </button>
                  <small>
                    {demo
                      ? "Sample data"
                      : a.experimental
                        ? "Experimental connection"
                        : "Remaining allowances"}
                  </small>
                </div>
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
          {issues.length > 0 && (
            <div className="error" role="status">
              {issues.map((issue, i) => (
                <p key={i}>{issue}</p>
              ))}
            </div>
          )}
          <p className="hint">
            Local connections are detected at launch. Enable or disable them
            below.
          </p>
          {config.accounts.map((a) => (
            <details key={a.id} className="connection-section">
              <summary>
                {names[a.provider]} ·{" "}
                {a.enabled
                  ? "Enabled"
                  : a.provider === "gemini"
                    ? "Sign-in needed"
                    : "Disabled"}
              </summary>
              <AccountEditor account={a} run={run} />
            </details>
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
            Enabled
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
                Optional account secret, protected by Windows Credential
                Manager. Save the account first.
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
