import { renderToStaticMarkup } from "react-dom/server";
import { JSDOM } from "jsdom";
import { expect, test } from "vitest";
import { QuotaTile } from "./QuotaTile";
import { emptyAccount, quotaLabel, visibleLimits } from "./widget";
import type { Account, Limit, Settings, Snapshot } from "./types";

const now = Date.parse("2026-10-02T15:00:00Z");
const settings: Settings = {
  view: "bars",
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
  view: Settings["view"] = "bars",
  status = "available",
) {
  return new JSDOM(
    renderToStaticMarkup(
      <QuotaTile
        account={{ ...account, ...changes }}
        snapshot={{ ...snapshot(limits), status }}
        settings={{ ...settings, view }}
        now={now}
        expanded={false}
        onClick={() => {}}
      />,
    ),
  ).window.document;
}

test.each(["bars", "rings"] as const)(
  "%s shows every window including weekly and monthly without expansion",
  (view) => {
    const doc = render(
      [
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
      ],
      {},
      view,
    );
    const rows = [...doc.querySelectorAll(".quota-window")];
    expect(rows).toHaveLength(3);
    expect(
      rows.map((q) => q.querySelector(".tile-quota")?.textContent),
    ).toEqual(["5 hours", "Weekly", "Monthly · Enterprise budget"]);
    expect(rows.map((q) => q.getAttribute("aria-label"))).toEqual(
      expect.arrayContaining([
        expect.stringContaining("21% remaining"),
        expect.stringContaining("84% remaining"),
        expect.stringContaining("60% remaining · 120 USD left"),
      ]),
    );
    expect(
      doc.querySelectorAll(view === "bars" ? "progress" : ".mini-ring"),
    ).toHaveLength(3);
    expect(doc.querySelector("button")?.getAttribute("aria-expanded")).toBe(
      "false",
    );
  },
);

test("pinning reorders but never removes windows, including separate feature pools", () => {
  const limits = [
    limit("current"),
    limit("week"),
    limit("sonnet", { name: "Sonnet weekly" }),
  ];
  expect(visibleLimits(snapshot(limits), "week").map((q) => q.id)).toEqual([
    "week",
    "current",
    "sonnet",
  ]);
  expect(snapshot(limits).limits.map((q) => q.id)).toEqual([
    "current",
    "week",
    "sonnet",
  ]);
  const doc = render(limits, { pinnedLimit: "week" });
  expect(doc.querySelectorAll(".quota-window")).toHaveLength(3);
  expect(doc.querySelector(".quota-window .tile-quota")?.textContent).toBe(
    "Weekly",
  );
  expect(doc.querySelectorAll('[aria-label="Pinned quota"]')).toHaveLength(1);
});

test("a missing pin stays explicit while available weekly windows remain visible", () => {
  const doc = render([limit("week")], { pinnedLimit: "removed-monthly" });
  expect(doc.body.textContent).toContain("Pinned limit unavailable");
  expect(doc.body.textContent).toContain("84%");
  expect(doc.querySelector('[aria-label="Pinned quota"]')).toBeNull();
});

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
  ).toBe("Monthly budget");
});

test("expired login keeps all cached windows and disconnected accounts do not expose readings", () => {
  const limits = [limit("current"), limit("week")];
  const expired = render(limits, {}, "bars", "loginRequired");
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
