import type { Account, Settings } from "./types";

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
  return { openai: "OpenAI", claude: "Claude", gemini: "Gemini", grok: "Grok" }[
    a.provider
  ];
}

export function widgetWidth(
  count: number,
  view: Settings["view"],
  panel: boolean,
): number {
  if (panel) return 540;
  // Additional accounts wrap at three columns instead of stretching across monitors.
  return Math.min(3, Math.max(1, count)) * (view === "bars" ? 166 : 96) + 48;
}
