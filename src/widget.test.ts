import { expect, test } from "vitest";
import {
  accountName,
  emptyAccount,
  visibleAccounts,
  widgetWidth,
  focusedAccount,
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
test("strip width is bounded and focus keeps a readable minimum", () => {
  expect(widgetWidth([account("one")], {}, false)).toBe(180);
  expect(widgetWidth([account("one")], {}, true)).toBe(300);
  expect(
    widgetWidth(
      Array.from({ length: 8 }, (_, i) => account(String(i))),
      {},
      false,
    ),
  ).toBe(480);
});

test("focus selects an enabled account by identity, falling back after disable or removal", () => {
  const all = [
    account("work", { enabled: true }),
    account("personal", { enabled: true }),
  ];
  expect(focusedAccount(all, "personal")).toBe(all[1]);
  expect(focusedAccount(all, "deleted")).toBeUndefined();
  expect(focusedAccount([account("personal")], "personal")).toBeUndefined();
  expect(focusedAccount(all, null)).toBeUndefined();
});
