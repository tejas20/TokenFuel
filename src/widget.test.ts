import { expect, test } from "vitest";
import {
  accountName,
  emptyAccount,
  visibleAccounts,
  widgetWidth,
  focusedAccount,
  compactQuotaLabel,
  quotaLabel,
} from "./widget";
import type { Account, Limit } from "./types";
const account = (id: string, change: Partial<Account> = {}): Account => ({
  ...emptyAccount,
  id,
  ...change,
});

test.each([
  ["Five Hour Limit Remaining", "rolling", "5h"],
  ["5-hour", "rolling", "5h"],
  ["Session 5h", "rolling", "5h"],
  ["Weekly Limit Remaining", "weekly", "Week"],
  ["Weekly Quota", "weekly", "Week"],
  ["Monthly budget", "monthly", "Month"],
  ["Daily allowance", "daily", "Day"],
  ["Sonnet weekly", "weekly", "Sonnet weekly"],
  ["Enterprise budget", "monthly", "Month · Enterprise budget"],
])("shortens %s without losing named pools", (name, period, expected) => {
  const quota = { name, period, product: "Claude", scope: "account" } as Limit;
  expect(compactQuotaLabel(quota)).toBe(expected);
  expect(quotaLabel(quota)).toBe(expected);
});

test("Copilot features stay distinct and Antigravity groups survive shortened windows", () => {
  for (const [name, expected] of [
    ["Chat", "Chat"],
    ["Code completions", "Code"],
    ["Premium requests", "Premium"],
  ]) {
    const quota = {
      name,
      period: "monthly",
      product: "Copilot",
      scope: "account",
    } as Limit;
    expect(compactQuotaLabel(quota)).toBe(expected);
    expect(quotaLabel(quota)).toBe(`Month · ${name}`);
  }
  const quota = {
    name: "Weekly Limit Remaining",
    period: "weekly",
    product: "Antigravity",
    scope: "Gemini Models",
  } as Limit;
  expect(compactQuotaLabel(quota)).toBe("Gemini · Week");
  expect(compactQuotaLabel({ ...quota, scope: "Claude Models" })).toBe(
    "Claude · Week",
  );
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
  expect(widgetWidth([account("one")], {}, false)).toBe(210);
  expect(widgetWidth([account("one")], {}, true)).toBe(300);
  expect(
    widgetWidth(
      Array.from({ length: 8 }, (_, i) => account(String(i))),
      {},
      false,
    ),
  ).toBe(544);
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
