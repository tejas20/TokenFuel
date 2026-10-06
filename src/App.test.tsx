// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, test, vi } from "vitest";
import type { Config, Limit } from "./types";
import { emptyAccount } from "./widget";

vi.mock("./bridge", () => ({
  desktop: false,
  demo: false,
  initial: {
    accounts: [],
    cached: {},
    settings: { theme: "dark", intervalSecs: 120 },
  },
  command: vi.fn(),
}));
import App from "./App";
import { command } from "./bridge";

afterEach(() => vi.resetAllMocks());
test.each(["copilot", "antigravity"] as const)(
  "%s details show a failed connection only once",
  async (provider) => {
    const message = "Provider connection failed.";
    const account = { ...emptyAccount, id: provider, provider, enabled: true };
    const config: Config = {
      schemaVersion: 2,
      position: null,
      settings: {
        theme: "dark",
        intervalSecs: 120,
        opaque: false,
        alwaysOnTop: true,
        startup: false,
        alerts: false,
        snapToEdges: true,
      },
      accounts: [account],
      cached: {
        [provider]: {
          accountId: provider,
          provider,
          status: "offline",
          message,
          limits: [],
          fetchedAt: new Date().toISOString(),
          retryAt: null,
        },
      },
    };
    vi.mocked(command).mockResolvedValue(config);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    (
      globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }
    ).IS_REACT_ACT_ENVIRONMENT = true;
    try {
      await act(async () => root.render(<App />));
      await act(async () =>
        container.querySelector<HTMLButtonElement>(".quota-tile")!.click(),
      );
      const details = container.querySelector(".details")!;
      expect(details.textContent!.split(message)).toHaveLength(2);
      expect(details.querySelector('[role="status"]')?.textContent).toBe(
        message,
      );
      expect(details.querySelector("img")?.getAttribute("src")).toBe(
        `/providers/${provider}.svg`,
      );
    } finally {
      await act(async () => root.unmount());
      container.remove();
    }
  },
);
test("provider details, simplified menu, saved focus, switching and disabling a focused account", async () => {
  const q = (id: string, remainingPercent: number): Limit => ({
    id,
    name: id === "session" ? "5 hours" : "Weekly",
    product: "Codex",
    scope: "account",
    unit: "percent",
    period: id === "session" ? "rolling" : "weekly",
    resetsAt: new Date(Date.now() + 86400000).toISOString(),
    used: null,
    total: null,
    remaining: null,
    remainingPercent,
    unlimited: false,
    source: "documented",
    observedAt: new Date().toISOString(),
  });
  let config: Config = {
    schemaVersion: 1,
    position: null,
    settings: {
      theme: "dark",
      opaque: false,
      alwaysOnTop: true,
      startup: false,
      alerts: false,
      intervalSecs: 120,
      snapToEdges: true,
    },
    accounts: [
      {
        ...emptyAccount,
        id: "work",
        label: "Work",
        enabled: true,
      },
      {
        ...emptyAccount,
        id: "personal",
        provider: "copilot",
        label: "Personal",
        enabled: true,
      },
    ],
    cached: {},
  };
  for (const a of config.accounts)
    config.cached[a.id] = {
      accountId: a.id,
      provider: a.provider,
      status: "available",
      message: "",
      limits: [q("session", 94), q("week", a.id === "personal" ? 18 : 84)],
      fetchedAt: new Date().toISOString(),
      retryAt: null,
    };
  vi.mocked(command).mockImplementation(
    async <T,>(
      name: string,
      args: Record<string, unknown> = {},
    ): Promise<T> => {
      if (name === "save_settings")
        config = {
          ...config,
          settings: {
            ...config.settings,
            ...(args.settings as Partial<Config["settings"]>),
          },
        };
      if (name === "save_account")
        config = {
          ...config,
          accounts: config.accounts.map((a) =>
            a.id === (args.account as Config["accounts"][0]).id
              ? (args.account as Config["accounts"][0])
              : a,
          ),
        };
      return config as T;
    },
  );
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  (
    globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }
  ).IS_REACT_ACT_ENVIRONMENT = true;
  const click = async (selector: string) => {
    const button = container.querySelector<HTMLButtonElement>(selector);
    expect(button).not.toBeNull();
    await act(async () => button!.click());
  };
  try {
    await act(async () => root.render(<App />));
    expect(container.querySelectorAll(".accounts progress")).toHaveLength(4);
    expect(container.querySelector(".compact-tools.vertical")).not.toBeNull();
    expect(
      [...container.querySelectorAll(".compact-tools button")].map((b) =>
        b.getAttribute("aria-label"),
      ),
    ).toEqual(["Widget menu", "Refresh usage", "Drag widget"]);
    await click('.quota-tile[aria-label*="Work"]');
    expect(container.querySelectorAll(".details progress")).toHaveLength(2);
    expect(container.querySelector(".details")?.textContent).toContain("Reset");
    expect(container.querySelector('.details [aria-label^="Pin"]')).toBeNull();
    expect(
      container.querySelector('.details [aria-label="Pinned quota"]'),
    ).toBeNull();
    expect(
      container.querySelector(".detail-actions")?.textContent,
    ).not.toContain("Remaining allowances");
    expect(container.querySelectorAll(".accounts progress")).toHaveLength(4);
    await click(".detail-actions .primary");
    expect(command).toHaveBeenCalledWith("save_settings", {
      settings: { focusAccountId: "work" },
    });
    expect(container.querySelectorAll(".quota-tile")).toHaveLength(1);
    expect(container.querySelector(".compact-tools.vertical")).toBeNull();
    expect(
      container.querySelector('[aria-label*="other accounts running low"]'),
    ).not.toBeNull();
    // The persisted configuration restores the same selection in a new mount.
    await act(async () => root.render(<App key="restart" />));
    expect(container.querySelector(".focus-selector")?.textContent).toContain(
      "Codex",
    );
    await click(".focus-selector");
    const menu = container.querySelector(".widget-menu")!;
    expect(menu.textContent).not.toMatch(
      /Refresh usage|Move widget|Show ring view|Show bar view/,
    );
    expect(menu.querySelector('[aria-label="Drag widget"]')).toBeNull();
    await click('.focus-option[aria-pressed="false"]');
    expect(config.settings.focusAccountId).toBe("personal");
    await click(".compact-tools button[aria-controls]");
    const allAccounts = [
      ...container.querySelectorAll<HTMLButtonElement>(".widget-menu > button"),
    ].find((b) => b.textContent === "All accounts")!;
    await act(async () => allAccounts.click());
    expect(config.settings.focusAccountId).toBeNull();
    expect(container.querySelectorAll(".quota-tile")).toHaveLength(2);
    expect(container.querySelector(".compact-tools.vertical")).not.toBeNull();
    await click('.quota-tile[aria-label*="Work"]');
    await act(async () =>
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" })),
    );
    expect(container.querySelector(".details")).toBeNull();
    await click('.quota-tile[aria-label*="Work"]');
    await click(".detail-actions .primary");
    await click(".quota-tile");
    config = {
      ...config,
      accounts: config.accounts.map((a) => ({
        ...a,
        enabled: a.id !== "work",
      })),
    };
    await click(".compact-tools button[aria-controls]");
    await click('.compact-tools [aria-label="Refresh usage"]');
    expect(container.querySelector(".focus-selector")).toBeNull();
    expect(container.querySelectorAll(".quota-tile")).toHaveLength(1);
    expect(container.querySelector(".compact-tools.vertical")).toBeNull();
    expect(container.querySelector(".details")).toBeNull();
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});

test("focus mode warns when a hidden account's reading ages without a new poll", async () => {
  vi.useFakeTimers();
  const observedAt = new Date().toISOString();
  const config: Config = {
    schemaVersion: 1,
    position: null,
    settings: {
      theme: "dark",
      intervalSecs: 120,
      focusAccountId: "work",
      opaque: false,
      alwaysOnTop: true,
      startup: false,
      alerts: false,
      snapToEdges: true,
    },
    accounts: [
      { ...emptyAccount, id: "work", label: "Work", enabled: true },
      { ...emptyAccount, id: "personal", label: "Personal", enabled: true },
    ],
    cached: {
      personal: {
        accountId: "personal",
        provider: "openai",
        status: "available",
        message: "Checked successfully",
        fetchedAt: observedAt,
        retryAt: null,
        limits: [
          {
            id: "week",
            name: "Weekly",
            product: "Codex",
            scope: "account",
            unit: "percent",
            period: "weekly",
            resetsAt: null,
            used: null,
            total: null,
            remaining: null,
            remainingPercent: 95,
            unlimited: false,
            source: "documented",
            observedAt,
          },
        ],
      },
    },
  };
  vi.mocked(command).mockResolvedValue(config);
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  (
    globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }
  ).IS_REACT_ACT_ENVIRONMENT = true;
  try {
    await act(async () => root.render(<App />));
    expect(container.querySelector(".focus-selector")).not.toBeNull();
    expect(container.querySelector(".menu-warning")).toBeNull();
    await act(async () => vi.advanceTimersByTime(300000));
    expect(
      container.querySelector('[aria-label*="connection issues"]'),
    ).not.toBeNull();
    await act(async () =>
      container
        .querySelector<HTMLButtonElement>(
          ".compact-tools button[aria-controls]",
        )!
        .click(),
    );
    await act(async () =>
      container
        .querySelector<HTMLButtonElement>('[aria-label="Settings"]')!
        .click(),
    );
    expect(container.querySelector(".settings .error")?.textContent).toContain(
      "Personal: Cached usage is stale",
    );
    expect(
      container.querySelector(".settings .error")?.textContent,
    ).not.toContain("Checked successfully");
    // An outside click must not discard the account editor's unsaved draft.
    await act(async () => document.dispatchEvent(new Event("pointerdown")));
    expect(container.querySelector(".settings")).not.toBeNull();
  } finally {
    await act(async () => root.unmount());
    container.remove();
    vi.useRealTimers();
  }
});
