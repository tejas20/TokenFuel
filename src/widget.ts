import type { Account, Limit, Provider, Snapshot } from "./types";

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
  accounts: Account[],
  cached: Record<string, Snapshot>,
  focus: boolean,
): number {
  const content = accounts.reduce((width, a) => {
    const limits = a.enabled ? (cached[a.id]?.limits ?? []) : [];
    return width + 36 + Math.max(1, limits.length) * 70;
  }, 40);
  return Math.min(480, Math.max(focus ? 300 : 180, content + (focus ? 80 : 0)));
}

export function focusedAccount(accounts: Account[], id?: string | null) {
  return accounts.find((a) => a.enabled && a.id === id);
}

// Only shorten unambiguous generic windows. Named model and workspace pools
// retain their names even when several pools have the same period.
export function compactQuotaLabel(q: Limit): string {
  const generic: Record<string, string> = {
    "5 hours": "5h",
    "5-hour": "5h",
    "5h": "5h",
    weekly: "Week",
    monthly: "Month",
    "monthly budget": "Monthly",
    daily: "Day",
  };
  return generic[q.name.toLowerCase()] ?? quotaLabel(q);
}
