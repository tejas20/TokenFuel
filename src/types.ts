export type Provider =
  | "claude"
  | "openai"
  | "gemini"
  | "grok"
  | "opencode"
  | "cursor"
  | "copilot"
  | "antigravity"
  | "unknown";
export interface Limit {
  id: string;
  name: string;
  product: string;
  scope: string;
  unit: string;
  period: string;
  resetsAt: string | null;
  resetLabel?: string | null;
  used: string | null;
  total: string | null;
  remaining: string | null;
  remainingPercent: number | null;
  unlimited: boolean;
  source: "documented" | "experimental" | "manual";
  observedAt: string;
}
export interface Snapshot {
  accountId: string;
  provider: Provider;
  identity?: {
    displayName: string;
    plan: string | null;
    workspace: string | null;
  } | null;
  status: string;
  message: string;
  limits: Limit[];
  fetchedAt: string | null;
  retryAt: string | null;
}
export interface Account {
  id: string;
  revision?: number;
  provider: Provider;
  label: string;
  workspace: string;
  connection:
    | "codexCli"
    | "claudeCli"
    | "browser"
    | "geminiWeb"
    | "opencodeGo"
    | "cursorLocal"
    | "grokCli"
    | "copilotCli"
    | "antigravityLocal"
    | "manual"
    | "unknown";
  enabled: boolean;
  experimental: boolean;
  pinnedLimit: string | null;
  cliPath: string | null;
  credentialPath: string | null;
  manualLimits: Limit[];
  hasCustomSecret?: boolean;
}
export interface Settings {
  focusAccountId?: string | null;
  view: "bars" | "rings";
  theme: "system" | "dark" | "light";
  opaque: boolean;
  alwaysOnTop: boolean;
  startup: boolean;
  alerts: boolean;
  intervalSecs: number;
  snapToEdges: boolean;
}
export interface Config {
  settingsIssues?: string[];
  discoveredProviders?: Provider[];
  schemaVersion: number;
  settings: Settings;
  accounts: Account[];
  cached: Record<string, Snapshot>;
  position: null | [number, number];
}
