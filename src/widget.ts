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
  const name = shortQuotaName(q.name);
  const period = (
    { weekly: "Week", monthly: "Month", daily: "Day" } as Record<string, string>
  )[q.period];
  const label =
    period && !new RegExp(`\\b${period}(?:ly)?\\b`, "i").test(name)
      ? `${period} · ${name}`
      : name;
  return q.product === "Antigravity" && q.scope !== "account"
    ? `${q.scope.replace(/\s+models$/i, "")} · ${label}`
    : label;
}

export function widgetWidth(
  accounts: Account[],
  cached: Record<string, Snapshot>,
  focus: boolean,
): number {
  const content = accounts.reduce((width, a) => {
    const limits = a.enabled ? (cached[a.id]?.limits ?? []) : [];
    return width + (focus ? 0 : 36) + Math.max(1, limits.length) * 70;
  }, 104);
  return Math.min(544, Math.max(focus ? 300 : 180, content + (focus ? 80 : 0)));
}

export function focusedAccount(accounts: Account[], id?: string | null) {
  return accounts.find((a) => a.enabled && a.id === id);
}

// Strip provider boilerplate only from generic windows. Preserve named pools.
function shortQuotaName(name: string): string {
  const normalized = name
    .trim()
    .toLowerCase()
    .replace(/[-_]/g, " ")
    .replace(/\s+/g, " ");
  const generic = normalized.replace(
    /\s+(?:limit|quota|budget|allowance)(?:\s+remaining)?$/,
    "",
  );
  if (/^(?:session\s+)?(?:five\s+hours?|5\s*hours?|5h)$/.test(generic))
    return "5h";
  if (/^week(?:ly)?$/.test(generic)) return "Week";
  if (/^month(?:ly)?$/.test(generic)) return "Month";
  if (["day", "daily"].includes(generic)) return "Day";
  return name.trim();
}

export function compactQuotaLabel(q: Limit): string {
  if (q.product === "Copilot") {
    const features: Record<string, string> = {
      chat: "Chat",
      "code completions": "Code",
      "premium requests": "Premium",
    };
    const feature = features[q.name.trim().toLowerCase()];
    if (feature) return feature;
  }
  return quotaLabel(q);
}
