import { invoke } from "@tauri-apps/api/core";
import type { Config, Provider, Limit } from "./types";
export const desktop = "__TAURI_INTERNALS__" in window;
export const demo = new URLSearchParams(location.search).has("demo");
const now = Date.now();
const providers: Provider[] = [
  "claude",
  "openai",
  "gemini",
  "grok",
  "opencode",
  "cursor",
  "copilot",
  "antigravity",
];
const query = new URLSearchParams(location.search);
const demoProviders = query.has("providers")
  ? query.get("providers")!.split(",")
  : providers;
export const initial: Config = {
  schemaVersion: 1,
  position: null,
  settings: {
    view: demo && query.get("view") === "rings" ? "rings" : "bars",
    theme: "dark",
    opaque: false,
    alwaysOnTop: false,
    startup: false,
    alerts: false,
    intervalSecs: 120,
    snapToEdges: true,
  },
  accounts: providers.map((provider, i) => ({
    id: provider,
    provider,
    label: i === 0 ? "Work" : "Personal",
    workspace: "",
    connection:
      provider === "openai"
        ? "codexCli"
        : provider === "claude"
          ? "claudeCli"
          : provider === "opencode"
            ? "opencodeGo"
            : provider === "cursor"
              ? "cursorLocal"
              : provider === "grok"
                ? "grokCli"
                : provider === "copilot"
                  ? "copilotCli"
                  : provider === "antigravity"
                    ? "antigravityLocal"
                    : "browser",
    enabled: demo && demoProviders.includes(provider),
    experimental: false,
    pinnedLimit: null,
    cliPath: null,
    credentialPath: null,
    manualLimits: [],
    hasCustomSecret: false,
  })),
  cached: {},
};
function getDemoLimits(provider: Provider, now: number): Limit[] {
  switch (provider) {
    case "claude":
      return [
        {
          id: "claude-spend",
          name: "Monthly budget",
          product: "Claude",
          scope: "account",
          unit: "USD",
          period: "monthly",
          resetsAt: new Date(now + 12 * 86400000).toISOString(),
          used: "80",
          total: "200",
          remaining: "120",
          remainingPercent: 60,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
      ];
    case "openai":
      return [
        {
          id: "codex-5h",
          name: "5 hours",
          product: "Codex",
          scope: "account",
          unit: "percent",
          period: "rolling",
          resetsAt: new Date(now + 7800000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 94,
          unlimited: false,
          source: "documented",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "codex-weekly",
          name: "Weekly",
          product: "Codex",
          scope: "account",
          unit: "percent",
          period: "weekly",
          resetsAt: new Date(now + 259200000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 84,
          unlimited: false,
          source: "documented",
          observedAt: new Date(now).toISOString(),
        },
      ];
    case "opencode":
      return [
        {
          id: "opencode:five_hour",
          name: "5 hours",
          product: "OpenCode Go",
          scope: "account",
          unit: "USD",
          period: "rolling",
          resetsAt: new Date(now + 7800000).toISOString(),
          used: "1.25",
          total: "5",
          remaining: "3.75",
          remainingPercent: 75,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "opencode:weekly",
          name: "Weekly",
          product: "OpenCode Go",
          scope: "account",
          unit: "USD",
          period: "weekly",
          resetsAt: new Date(now + 259200000).toISOString(),
          used: "14",
          total: "35",
          remaining: "21",
          remainingPercent: 60,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "opencode:monthly",
          name: "Monthly",
          product: "OpenCode Go",
          scope: "account",
          unit: "USD",
          period: "monthly",
          resetsAt: new Date(now + 2592000000).toISOString(),
          used: "45",
          total: "150",
          remaining: "105",
          remainingPercent: 70,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
      ];
    case "cursor":
      return [
        {
          id: "cursor:plan:auto",
          name: "Cursor Models",
          product: "Cursor",
          scope: "account",
          unit: "percent",
          period: "monthly",
          resetsAt: new Date(now + 2592000000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 58,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "cursor:plan:api",
          name: "API models",
          product: "Cursor",
          scope: "account",
          unit: "percent",
          period: "monthly",
          resetsAt: new Date(now + 2592000000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 85,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
      ];
    case "grok":
      return [
        {
          id: "grok:credits",
          name: "Grok Credits",
          product: "Grok",
          scope: "account",
          unit: "percent",
          period: "monthly",
          resetsAt: new Date(now + 2592000000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 65,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "grok:on_demand",
          name: "On-demand spending",
          product: "Grok",
          scope: "account",
          unit: "USD",
          period: "monthly",
          resetsAt: new Date(now + 2592000000).toISOString(),
          used: "12.50",
          total: "50",
          remaining: "37.50",
          remainingPercent: 75,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: "grok:prepaid_balance",
          name: "Prepaid balance",
          product: "Grok",
          scope: "account",
          unit: "USD",
          period: "balance",
          resetsAt: null,
          used: null,
          total: "25",
          remaining: "25",
          remainingPercent: null,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
      ];
    default:
      return [
        {
          id: `${provider}-5h`,
          name: "5 hours",
          product: provider,
          scope: "account",
          unit: "percent",
          period: "rolling",
          resetsAt: new Date(now + 7800000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 66,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
        {
          id: `${provider}-weekly`,
          name: "Weekly",
          product: provider,
          scope: "account",
          unit: "percent",
          period: "weekly",
          resetsAt: new Date(now + 259200000).toISOString(),
          used: null,
          total: null,
          remaining: null,
          remainingPercent: 18,
          unlimited: false,
          source: "experimental",
          observedAt: new Date(now).toISOString(),
        },
      ];
  }
}

if (demo) {
  providers.forEach((provider) => {
    initial.cached[provider] = {
      accountId: provider,
      provider,
      status: "available",
      message: "Design preview · fictional sample data.",
      limits: getDemoLimits(provider, now),
      fetchedAt: new Date(now).toISOString(),
      retryAt: null,
    };
  });
}
let preview = JSON.parse(JSON.stringify(initial)) as Config;
export async function command<T = void>(
  name: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (desktop) return invoke<T>(name, args);
  if (name === "read_config") return preview as T;
  if (name === "save_settings")
    preview = { ...preview, settings: args.settings as Config["settings"] };
  else if (name === "save_account") {
    const account = args.account as Config["accounts"][0];
    const i = preview.accounts.findIndex((a) => a.id === account.id);
    if (i >= 0) preview.accounts[i] = account;
    else preview.accounts.push({ ...account, id: crypto.randomUUID() });
  } else if (name === "remove_account")
    preview.accounts = preview.accounts.filter((a) => a.id !== args.id);
  else if (name === "save_account_secret") {
    const acc = preview.accounts.find((a) => a.id === args.id);
    if (acc) acc.hasCustomSecret = true;
  } else if (name === "clear_account_secret") {
    const acc = preview.accounts.find((a) => a.id === args.id);
    if (acc) acc.hasCustomSecret = false;
  } else if (name === "open_provider" || name === "set_manual")
    throw Error("This connection needs the Windows app.");
  return undefined as T;
}
