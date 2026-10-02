import type { Account, Limit, Provider, Settings, Snapshot } from "./types";

export const emptyAccount: Account = {
  id: "unconfigured",
  provider: "openai",
  label: "Personal",
  workspace: "",
  connection: "codexCli",
  enabled: false,
  experimental: false,
  pinnedLimit: null,
  cliPath: null,
  credentialPath: null,
  manualLimits: [],
};

// Connection consent, rather than a successful last poll, determines visibility.
// Expired/offline accounts must stay visible so their errors cannot disappear.
export function visibleAccounts(accounts: Account[]): Account[] {
  const enabled = accounts.filter((a) => a.enabled);
  if (enabled.length) return enabled;
  return [
    accounts.find((a) => a.connection === "codexCli") ??
      accounts[0] ??
      emptyAccount,
  ];
}

export function accountName(a: Account): string {
  if (a.provider === "openai" && a.connection === "codexCli") return "Codex";
  const names: Record<Provider, string> = {
    openai: "OpenAI",
    claude: "Claude",
    gemini: "Gemini",
    grok: "Grok",
    opencode: "OpenCode Go",
    cursor: "Cursor",
    copilot: "Copilot",
    antigravity: "Antigravity",
    unknown: "Unknown",
  };
  return names[a.provider] ?? "Account";
}

// Pinning controls prominence, not visibility: every reported pool stays separate.
export function visibleLimits(
  snapshot: Snapshot | undefined,
  pinned: string | null,
): Limit[] {
  const limits = snapshot?.limits ?? [];
  const featured = limits.find((q) => q.id === pinned);
  return featured
    ? [featured, ...limits.filter((q) => q !== featured)]
    : limits;
}

export function quotaLabel(q: Limit): string {
  const period = (
    { weekly: "Weekly", monthly: "Monthly", daily: "Daily" } as Record<
      string,
      string
    >
  )[q.period];
  return period && !q.name.toLowerCase().includes(period.toLowerCase())
    ? `${period} · ${q.name}`
    : q.name;
}

export function widgetWidth(
  count: number,
  view: Settings["view"],
  panel: boolean,
): number {
  if (panel) return 540;
  // Additional accounts wrap at three columns instead of stretching across monitors.
  return Math.min(3, Math.max(1, count)) * (view === "bars" ? 166 : 144) + 48;
}
