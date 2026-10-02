import { invoke } from "@tauri-apps/api/core";
import type { Config, Provider, Limit } from "./types";
export const desktop = "__TAURI_INTERNALS__" in window;
export const demo = new URLSearchParams(location.search).has("demo");
const now = Date.now();
const providers: Provider[] = ["claude", "openai", "gemini"];
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
          : "browser",
    enabled: demo && demoProviders.includes(provider),
    experimental: false,
    pinnedLimit: null,
    cliPath: null,
    credentialPath: null,
    manualLimits: [],
  })),
  cached: {},
};
if (demo)
  providers.forEach((provider, i) => {
    const q: Limit = {
      id: "sample",
      name: ["Monthly budget", "5 hours", "Weekly"][i],
      product: i === 1 ? "Codex" : "",
      scope: "account",
      unit: i === 0 ? "USD" : "percent",
      period: i === 0 ? "monthly" : i === 2 ? "weekly" : "rolling",
      resetsAt: new Date(
        now + [12 * 86400000, 7800000, 259200000][i],
      ).toISOString(),
      used: i === 0 ? "80" : null,
      total: i === 0 ? "200" : null,
      remaining: i === 0 ? "120" : null,
      remainingPercent: [60, 94, 18][i],
      unlimited: false,
      source: i === 1 ? "documented" : i === 2 ? "experimental" : "manual",
      observedAt: new Date(now).toISOString(),
    };
    initial.cached[provider] = {
      accountId: provider,
      provider,
      status: "available",
      message: "Design preview · fictional sample data.",
      limits: [q],
      fetchedAt: new Date(now).toISOString(),
      retryAt: null,
    };
  });
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
  else if (name === "open_provider" || name === "set_manual")
    throw Error("This connection needs the Windows app.");
  return undefined as T;
}
