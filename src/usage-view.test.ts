import { JSDOM } from "jsdom";
import { readFileSync } from "node:fs";
import { expect, test } from "vitest";
const script = readFileSync(
  new URL("../src-tauri/src/usage-view.js", import.meta.url),
  "utf8",
);
function capture(html: string, path = "/app") {
  const dom = new JSDOM(html, {
    url: `https://gemini.google.com${path}`,
    runScripts: "outside-only",
  });
  return JSON.parse(JSON.stringify(dom.window.eval(script)));
}
const usage = (body: string) =>
  `<div role="dialog" aria-label="Usage limits">${body}</div>`;
test("observed Gemini personal Usage markup yields separate pools and preserves reset text without guessing timezone", () => {
  const html =
    '<usage-metrics-window><h2>Usage limits</h2><div class="usage-metrics-description"><p>Updated just now</p></div><div data-test-id="gxu-currently"><div><p>Current usage</p><p>1% used</p></div><p>Resets at 3:49 PM</p></div><div data-test-id="gxu-weekly"><div><p>Weekly limit</p><p>Resets Oct 7 at 12:49 PM</p></div><p>0% used</p></div></usage-metrics-window>';
  expect(capture(html, "/usage")).toEqual([
    {
      name: "5 hours",
      usedPercent: 1,
      resetsAt: null,
      resetLabel: "Resets at 3:49 PM",
      ageSeconds: 0,
    },
    {
      name: "Weekly",
      usedPercent: 0,
      resetsAt: null,
      resetLabel: "Resets Oct 7 at 12:49 PM",
      ageSeconds: 0,
    },
  ]);
  expect(capture(html, "/app")).toEqual([]);
  expect(
    capture(html.replace("Updated just now", "Updated 5 min ago"), "/usage")[0]
      .ageSeconds,
  ).toBe(300);
  expect(
    capture(
      html.replace("Updated just now", "Updated 8 minutes ago"),
      "/usage",
    )[0].ageSeconds,
  ).toBe(480);
});
test("separate five-hour and weekly rows preserve orientation and explicit reset timestamps", () => {
  const q = capture(
    usage(
      '<section><h3>5-hour limit</h3><span>30% used</span><time datetime="2026-10-02T15:00:00Z"></time></section><section><h3>Weekly</h3><div role="progressbar" aria-label="Remaining" aria-valuenow="87"></div></section>',
    ),
  );
  expect(q).toEqual([
    { name: "5 hours", usedPercent: 30, resetsAt: "2026-10-02T15:00:00.000Z" },
    { name: "Weekly", usedPercent: 13, resetsAt: null },
  ]);
});
test("missing, ambiguous, non-percent and hidden bars never become zero usage", () => {
  expect(
    capture(
      usage(
        '<section>Weekly<div role="progressbar" aria-label="Used"></div><div role="progressbar" aria-valuenow="4"></div><div role="progressbar" aria-label="Used" aria-valuenow="4" aria-valuemax="10"></div><span hidden>20% used</span></section>',
      ),
    ),
  ).toEqual([]);
});
test("duplicate weekly pools and mixed windows are rejected rather than silently merged", () => {
  expect(
    capture(
      usage(
        "<section>Weekly<span>20% used</span></section><section>Weekly<span>70% used</span></section>",
      ),
    ),
  ).toEqual([]);
  expect(
    capture(
      usage("<section>Weekly and 5-hour limits<span>20% used</span></section>"),
    ),
  ).toEqual([]);
});
test("ordinary page and chat percentages are ineligible", () => {
  expect(capture("<main>Weekly<span>20% used</span></main>")).toEqual([]);
  expect(
    capture(
      '<div role="dialog" aria-label="Conversation">Weekly<span>20% used</span></div>',
    ),
  ).toEqual([]);
});
test("text orientation and invalid reset remain explicit", () => {
  expect(
    capture(
      usage(
        '<section>Daily<span>Remaining: 64%</span><time datetime="tomorrow"></time></section>',
      ),
    ),
  ).toEqual([{ name: "Daily", usedPercent: 36, resetsAt: null }]);
});
