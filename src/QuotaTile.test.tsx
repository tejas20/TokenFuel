import { renderToStaticMarkup } from "react-dom/server";
import { JSDOM } from "jsdom";
import { expect, test } from "vitest";
import { QuotaTile } from "./QuotaTile";
import { emptyAccount, quotaLabel } from "./widget";
import type { Account, Limit, Settings, Snapshot } from "./types";

const now = Date.parse("2026-10-02T15:00:00Z");
const settings: Settings = {
  theme: "dark",
  opaque: false,
  alwaysOnTop: false,
  startup: false,
  alerts: false,
  intervalSecs: 120,
  snapToEdges: true,
};
const account: Account = { ...emptyAccount, id: "personal", enabled: true };
const limit = (id: string, changes: Partial<Limit> = {}): Limit => ({
  id,
  name: id === "current" ? "5 hours" : "Weekly",
  product: "Codex",
  scope: "account",
  unit: "percent",
  period: id === "current" ? "rolling" : "weekly",
  resetsAt: "2026-10-05T15:00:00Z",
  used: null,
  total: null,
  remaining: null,
  remainingPercent: id === "current" ? 21 : 84,
  unlimited: false,
  source: "documented",
  observedAt: new Date(now).toISOString(),
  ...changes,
});
const snapshot = (limits: Limit[]): Snapshot => ({
  accountId: account.id,
  provider: "openai",
  status: "available",
  message: "",
  limits,
  fetchedAt: new Date(now).toISOString(),
  retryAt: null,
});
function render(
  limits: Limit[],
  changes: Partial<Account> = {},
  status = "available",
) {
  return new JSDOM(
    renderToStaticMarkup(
      <QuotaTile
        account={{ ...account, ...changes }}
        snapshot={{ ...snapshot(limits), status }}
        settings={settings}
        now={now}
        expanded={false}
        onClick={() => {}}
      />,
    ),
  ).window.document;
}

test("shows every window including weekly and monthly without expansion", () => {
  const doc = render([
    limit("current"),
    limit("week"),
    limit("month", {
      name: "Enterprise budget",
      period: "monthly",
      unit: "USD",
      total: "200",
      used: "80",
      remaining: "120",
      remainingPercent: 60,
    }),
  ]);
  const rows = [...doc.querySelectorAll(".quota-window")];
  expect(rows).toHaveLength(3);
  expect(rows.map((q) => q.querySelector(".tile-quota")?.textContent)).toEqual([
    "5h",
    "Week",
    "Month · Enterprise budget",
  ]);
  expect(rows.map((q) => q.getAttribute("aria-label"))).toEqual(
    expect.arrayContaining([
      expect.stringContaining("21% remaining"),
      expect.stringContaining("84% remaining"),
      expect.stringContaining("60% remaining · 120 USD left"),
    ]),
  );
  expect(doc.querySelectorAll("progress")).toHaveLength(3);
  expect(doc.querySelector("button")?.getAttribute("aria-expanded")).toBe(
    "false",
  );
});

test.each(["week", "removed-monthly"])(
  "legacy pin %s cannot reorder or mark quota windows",
  (pinnedLimit) => {
    const limits = [
      limit("current"),
      limit("week"),
      limit("sonnet", { name: "Sonnet weekly" }),
    ];
    const legacyAccount = { ...account, pinnedLimit };
    const doc = render(limits, legacyAccount);
    expect(doc.querySelectorAll(".quota-window")).toHaveLength(3);
    expect(
      [...doc.querySelectorAll(".tile-quota")].map((q) => q.textContent),
    ).toEqual(["5h", "Week", "Sonnet weekly"]);
    expect(doc.querySelector('[aria-label="Pinned quota"]')).toBeNull();
    expect(
      doc.querySelector('[aria-label="Pinned limit unavailable"]'),
    ).toBeNull();
  },
);

test("freshness and low treatments belong to each pool", () => {
  const doc = render([
    limit("current", { observedAt: "2026-10-02T14:00:00Z" }),
    limit("week", { remainingPercent: 10 }),
  ]);
  const rows = doc.querySelectorAll(".quota-window");
  expect(rows[0].classList.contains("unverified")).toBe(true);
  expect(rows[0].textContent).toContain("Stale · cached");
  expect(rows[1].classList.contains("unverified")).toBe(false);
  expect(rows[1].classList.contains("low")).toBe(true);
  expect(rows[1].textContent).toContain("Low");
  expect(rows[1].textContent).toContain("Reset 3d");
});

test("monthly zero, unknown denominator and unlimited are not invented percentages", () => {
  const doc = render([
    limit("zero", {
      name: "Budget",
      period: "monthly",
      unit: "USD",
      remaining: "0",
      remainingPercent: null,
    }),
    limit("unknown", {
      name: "Allocation",
      period: "monthly",
      remainingPercent: null,
    }),
    limit("unlimited", {
      name: "Plan",
      period: "monthly",
      remainingPercent: null,
      unlimited: true,
    }),
  ]);
  expect(doc.body.textContent).toContain("0 USD left");
  expect(doc.body.textContent).toContain("Unlimited");
  expect(doc.querySelectorAll("progress")).toHaveLength(0);
  expect(doc.querySelectorAll(".unknown-track")).toHaveLength(3);
  expect(doc.body.textContent).not.toContain("0%");
  expect(
    quotaLabel(limit("m", { name: "Monthly budget", period: "monthly" })),
  ).toBe("Month");
});

test("expired login keeps all cached windows and disconnected accounts do not expose readings", () => {
  const limits = [limit("current"), limit("week")];
  const expired = render(limits, {}, "loginRequired");
  expect(expired.querySelectorAll(".unverified")).toHaveLength(2);
  expect(
    [...expired.querySelectorAll(".tile-caption")].every(
      (q) => q.textContent === "Sign in · cached",
    ),
  ).toBe(true);
  const disconnected = render(limits, { enabled: false });
  expect(disconnected.querySelectorAll("progress")).toHaveLength(0);
  expect(disconnected.body.textContent).toContain("Connect");
  expect(disconnected.body.textContent).not.toContain("84%");
});

test("spending without a denominator keeps its limit unknown", () => {
  const doc = render([
    limit("spend", {
      name: "On-demand spend",
      period: "monthly",
      unit: "USD",
      used: "42.50",
      total: null,
      remaining: null,
      remainingPercent: null,
    }),
  ]);
  expect(doc.body.textContent).toContain("42.50 USD used (limit not reported)");
  expect(
    doc.querySelector(".quota-window")?.getAttribute("aria-label"),
  ).toContain("42.50 USD used (limit not reported)");
  expect(doc.body.textContent).not.toContain("uncapped");
  expect(doc.body.textContent).not.toContain("No reading");
});
