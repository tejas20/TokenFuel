(() => {
  // Gemini personal Usage page verified on 2026-10-02. Fixed selectors keep reads out of the sidebar/chat.
  if (
    location.hostname === "gemini.google.com" &&
    location.pathname === "/usage"
  ) {
    const surface = document.querySelector("usage-metrics-window");
    if (
      !surface ||
      surface.querySelector("h2")?.textContent.trim() !== "Usage limits"
    )
      return [];
    const out = [];
    const ageText = [
      ...surface.querySelectorAll(".usage-metrics-description p"),
    ]
      .map((el) => el.textContent.trim())
      .find((text) => /^Updated /i.test(text));
    const ageMatch = ageText?.match(
      /^Updated (\d+) (min|minute|hr|hour)s? ago$/i,
    );
    const ageSeconds = /^Updated just now$/i.test(ageText || "")
      ? 0
      : ageMatch
        ? Number(ageMatch[1]) * (/^(hour|hr)$/i.test(ageMatch[2]) ? 3600 : 60)
        : null;
    for (const [selector, name] of [
      ['[data-test-id="gxu-currently"]', "5 hours"],
      ['[data-test-id="gxu-weekly"]', "Weekly"],
    ]) {
      const rows = surface.querySelectorAll(selector);
      if (rows.length !== 1) continue;
      const texts = [...rows[0].querySelectorAll("p")].map((el) =>
        el.textContent.trim(),
      );
      const matches = texts
        .map((text) => text.match(/^(\d+(?:\.\d+)?)%\s+used$/i))
        .filter(Boolean);
      if (matches.length !== 1) continue;
      const usedPercent = Number(matches[0][1]);
      if (!Number.isFinite(usedPercent) || usedPercent < 0 || usedPercent > 100)
        continue;
      const resetLabel =
        texts.find((text) =>
          /^Resets (?:at \d{1,2}:\d{2}\s*(?:AM|PM)|[A-Z][a-z]{2} \d{1,2} at \d{1,2}:\d{2}\s*(?:AM|PM))$/i.test(
            text,
          ),
        ) || null;
      out.push({ name, usedPercent, resetsAt: null, resetLabel, ageSeconds });
    }
    return out;
  }
  // Only a visible Usage surface is eligible; return quota scalars, never page content.
  const visible = (el) => !el.closest('[hidden],[aria-hidden="true"]');
  const dialogs = [
    ...document.querySelectorAll('[role="dialog"], [role="alertdialog"]'),
  ];
  const root =
    dialogs.find(
      (el) =>
        visible(el) &&
        /\busage(?: limits?)?\b/i.test(
          el.getAttribute("aria-label") ||
            el.querySelector('h1,h2,h3,[role="heading"]')?.textContent ||
            "",
        ),
    ) ||
    (/\/settings\/usage\b/.test(location.pathname)
      ? document.querySelector("main")
      : null);
  if (!root) return [];
  function period(text) {
    // Multiple periods or model-specific pools in one row cannot be collapsed safely.
    if (/sonnet|opus|thinking|deep research/i.test(text)) return null;
    const names = [
      [/\bweek(?:ly)?\b/i, "Weekly"],
      [/\b(?:5|five)[ -]?hours?\b/i, "5 hours"],
      [/\bmonth(?:ly)?\b/i, "Monthly budget"],
      [/\bdaily\b/i, "Daily"],
    ].filter(([pattern]) => pattern.test(text));
    return names.length === 1 ? names[0][1] : null;
  }
  function row(el) {
    for (
      let p = el.parentElement, depth = 0;
      p && p !== root && depth < 3;
      p = p.parentElement, depth++
    ) {
      const walker = document.createTreeWalker(p, NodeFilter.SHOW_TEXT);
      const parts = [];
      let node;
      while ((node = walker.nextNode()) && parts.join(" ").length < 1200) {
        if (node.parentElement && visible(node.parentElement))
          parts.push(node.textContent || "");
      }
      const text = parts.join(" ").slice(0, 1200);
      const name = period(text);
      if (name) return { el: p, text, name };
    }
    return null;
  }
  const counters = new Map(),
    ambiguous = new Set();
  function add(el, value, remaining) {
    const r = row(el);
    if (!r || !Number.isFinite(value) || value < 0 || value > 100) return;
    const usedPercent = remaining ? 100 - value : value;
    const old = counters.get(r.name);
    if (old && (old.el !== r.el || old.usedPercent !== usedPercent)) {
      ambiguous.add(r.name);
      return;
    }
    const dt = r.el.querySelector("time[datetime]")?.getAttribute("datetime");
    const resetsAt =
      dt &&
      /^\d{4}-\d\d-\d\dT\d\d:\d\d(?::\d\d(?:\.\d+)?)?(?:Z|[+-]\d\d:\d\d)$/.test(
        dt,
      ) &&
      Number.isFinite(Date.parse(dt))
        ? new Date(dt).toISOString()
        : null;
    counters.set(r.name, { el: r.el, name: r.name, usedPercent, resetsAt });
  }
  for (const el of root.querySelectorAll('[role="progressbar"]')) {
    if (!visible(el) || !el.hasAttribute("aria-valuenow")) continue;
    const r = row(el);
    const orientation = el.getAttribute("aria-label") || r?.text || "";
    const remaining = /\bremaining\b/i.test(orientation),
      used = /\bused\b|utilization/i.test(orientation);
    if (
      remaining === used ||
      Number(el.getAttribute("aria-valuemax") || 100) !== 100 ||
      Number(el.getAttribute("aria-valuemin") || 0) !== 0
    )
      continue;
    add(el, Number(el.getAttribute("aria-valuenow")), remaining);
  }
  for (const el of root.querySelectorAll("p,span")) {
    if (!visible(el) || el.children.length) continue;
    const text = (el.textContent || "").trim();
    const suffix = text.match(/^(\d+(?:\.\d+)?)\s*%\s*(remaining|used)$/i);
    const prefix = text.match(/^(remaining|used)\s*:?\s*(\d+(?:\.\d+)?)\s*%$/i);
    if (suffix)
      add(el, Number(suffix[1]), suffix[2].toLowerCase() === "remaining");
    else if (prefix)
      add(el, Number(prefix[2]), prefix[1].toLowerCase() === "remaining");
  }
  return [...counters.values()]
    .filter((q) => !ambiguous.has(q.name))
    .sort(
      (a, b) =>
        ["5 hours", "Daily", "Weekly", "Monthly budget"].indexOf(a.name) -
        ["5 hours", "Daily", "Weekly", "Monthly budget"].indexOf(b.name),
    )
    .slice(0, 12)
    .map(({ el, ...q }) => q);
})();
