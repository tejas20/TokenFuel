// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { expect, test, vi } from "vitest";
import type { Account } from "./types";
import { emptyAccount } from "./widget";

vi.mock("./bridge", () => ({
  desktop: false,
  demo: false,
  initial: {
    accounts: [],
    cached: {},
    settings: { view: "bars", theme: "dark", intervalSecs: 120 },
  },
  command: vi.fn(async () => ({
    accounts: [],
    cached: {},
    settings: { view: "bars", theme: "dark", intervalSecs: 120 },
  })),
}));
import App, { AccountEditor } from "./App";

function editor(
  connection: Account["connection"],
  provider: Account["provider"] = "openai",
) {
  const html = renderToStaticMarkup(
    <AccountEditor
      account={{ ...emptyAccount, id: "account", connection, provider }}
      run={async () => true}
    />,
  );
  const container = document.createElement("div");
  container.innerHTML = html;
  return container;
}

test("Codex setup offers consent and Save without asking for a token", () => {
  const ui = editor("codexCli");
  expect(ui.textContent).toContain("Automatically finds");
  expect(ui.querySelector('input[type="password"]')).toBeNull();
  expect(
    ui
      .querySelector('select[aria-label="Connection source"]')
      ?.closest("details"),
  ).toBeNull();
  expect(
    [...ui.querySelectorAll("button")]
      .find((b) => b.textContent === "Save")
      ?.closest("details"),
  ).toBeNull();
  expect(
    ui.querySelector('input[type="checkbox"]')?.hasAttribute("checked"),
  ).toBe(false);
});

test("eligible provider secrets and workspace IDs are inside collapsed advanced settings", () => {
  const ui = editor("cursorLocal", "cursor");
  const advanced = ui.querySelector("details")!;
  expect(advanced.hasAttribute("open")).toBe(false);
  expect(ui.querySelector('input[type="password"]')?.closest("details")).toBe(
    advanced,
  );
  expect(ui.querySelector('[aria-label="Workspace"]')?.closest("details")).toBe(
    advanced,
  );
  expect(ui.textContent).toContain("Experimental opt-in");
});

test("manual and browser sources never request an unsupported secret", () => {
  for (const source of [
    "manual",
    "browser",
    "geminiWeb",
    "claudeCli",
  ] as const) {
    expect(editor(source).querySelector('input[type="password"]')).toBeNull();
  }
});

test("new providers offer their local source and manual input without unsupported browser capture", () => {
  for (const [source, provider] of [
    ["cursorLocal", "cursor"],
    ["grokCli", "grok"],
    ["opencodeGo", "opencode"],
    ["copilotCli", "copilot"],
    ["antigravityLocal", "antigravity"],
  ] as const) {
    const ui = editor(source, provider);
    expect(ui.querySelector('option[value="browser"]')).toBeNull();
    expect(ui.querySelector('option[value="manual"]')).not.toBeNull();
  }
});

test("first run opens setup when no accounts are connected", async () => {
  const container = document.createElement("div");
  const root = createRoot(container);
  (
    globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }
  ).IS_REACT_ACT_ENVIRONMENT = true;
  await act(async () => {
    root.render(<App />);
  });
  expect(container.querySelector(".settings")).not.toBeNull();
  expect(container.textContent).toContain("Choose your provider");
  await act(async () => root.unmount());
});
