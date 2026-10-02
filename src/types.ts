export type Provider = "claude" | "openai" | "gemini" | "grok";
export interface Limit {
  id: string;
  name: string;
  product: string;
  scope: string;
  unit: string;
  period: string;
  resetsAt: string | null;
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
  connection: "codexCli" | "claudeCli" | "browser" | "manual";
  enabled: boolean;
  experimental: boolean;
  pinnedLimit: string | null;
  cliPath: string | null;
  credentialPath: string | null;
  manualLimits: Limit[];
}
export interface Settings {
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
  schemaVersion: number;
  settings: Settings;
  accounts: Account[];
  cached: Record<string, Snapshot>;
  position: null | [number, number];
}
