import { expect, test } from "vitest";
import {
  accountName,
  emptyAccount,
  visibleAccounts,
  widgetWidth,
} from "./widget";
import type { Account } from "./types";
const account = (id: string, change: Partial<Account> = {}): Account => ({
  ...emptyAccount,
  id,
  ...change,
});

test("only consented connections appear, without placeholders for other providers", () => {
  const all = [
    account("claude", { provider: "claude" }),
    account("codex", { enabled: true }),
    account("gemini", { provider: "gemini", enabled: true }),
  ];
  expect(visibleAccounts(all).map((a) => a.id)).toEqual(["codex", "gemini"]);
  expect(
    visibleAccounts(all.map((a) => ({ ...a, enabled: a.id === "codex" }))).map(
      (a) => a.id,
    ),
  ).toEqual(["codex"]);
});
test("no connections or no configured accounts still yields exactly one tile", () => {
  expect(
    visibleAccounts([
      account("gemini", { provider: "gemini", connection: "geminiWeb" }),
      account("codex"),
    ]).map((a) => a.id),
  ).toEqual(["codex"]);
  expect(visibleAccounts([])).toEqual([emptyAccount]);
});
test("temporary poll failures cannot hide enabled accounts; duplicate providers remain separate", () => {
  const all = [
    account("work", { enabled: true }),
    account("personal", { enabled: true }),
  ];
  expect(visibleAccounts(all)).toEqual(all); // No dependency on cache or poll status.
  expect(accountName(all[0])).toBe("Codex");
});
test("widget shrinks with account count and view, but panels have usable space", () => {
  expect(widgetWidth(1, "bars", false)).toBe(214);
  expect(widgetWidth(2, "bars", false)).toBe(380);
  expect(widgetWidth(2, "rings", false)).toBe(336);
  expect(widgetWidth(5, "bars", false)).toBe(546);
  expect(widgetWidth(1, "rings", true)).toBe(540);
});
