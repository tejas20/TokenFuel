# Development preview validation

2026-10-02, Windows x64. This is a testable development preview, not certification of every provider or plan.

- 15 Rust tests pass: decimal arithmetic/overages, unknown denominators, changed payloads, monthly-only enterprise shapes, stable scoped pools, currency mismatch, freshness/expired resets, retry scheduling, navigation restrictions.
- 10 frontend/DOM regression tests pass: countdown, per-quota staleness, missing pinned pools, observed Gemini markup, used/remaining direction, duplicate/hidden/ambiguous counters and explicit reset/freshness labels.
- Rust formatting and Clippy with warnings denied; strict TypeScript/Vite production build pass.
- Bar/ring previews and settings verified in the browser; this does not certify native tray, startup, monitor/DPI or alert interactions.
- Codex installed app-server live quota reads succeeded, with separate five-hour/weekly windows. These counters belong to Codex.
- Exact shipped Gemini extraction matched the signed-in PRO Usage page, including a page reload, provider freshness and displayed reset labels. Native isolated-window Gemini live polling remains unverified.
- Private GitHub owner/visibility verified; two previous hosted Windows validation runs passed. Reviewed source and refreshed private release artifacts are delivered together.

Claude Enterprise verification remains on the office laptop checklist. Ordinary ChatGPT model counters remain unsupported; the presence of the ChatGPT app does not expose a supported quota source to TokenFuel. Sign-ins in the development browser are not shared with the widget. No credentials or conversations were exported during these checks.

See `review-2026-10-02.md` for the review findings and `office-validation.md` for remaining device/account checks. Release binaries are unsigned development builds.
