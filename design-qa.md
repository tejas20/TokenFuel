# TokenFuel compact visual QA

**Final result: passed** for the local frontend presentation and tested browser interactions. Native provider and device gaps below are not certified by this result.

## Source and full-view comparison

- User selected Slim Strip and Mini Rings by name. Original references: `docs/design/compact-1.png` and `compact-3.png`; revised combined target: `docs/design/compact-selected.png` (1536×1024), shown before UI implementation. It intentionally replaces the earlier large dock.
- Browser URL: `http://127.0.0.1:1420/?demo=1&providers=openai,gemini`. Default viewport 1715×1198, DPR 1. Sample values 94% and 18%; these are fictional and explicitly labelled.
- Full implementation screenshots: `docs/design/compact-live-bars-full.png`, `compact-live-rings-full.png`, `compact-live-light-full.png`, `compact-live-settings-full.png`. Narrow screenshot: `compact-live-narrow-full.png`, viewport 360×760.
- Compact logical surface sizes for two providers: strip 380×84, rings 240×116. The browser-only disclosure adds 17px; captured rectangles are 380×101 and 240×133. Fractional horizontal centring requires one extra capture pixel. Native transparent margins add 12px around the surface and omit the disclosure.
- `scripts/compare-compact.mjs` crops the target strip at 122,189 /1278×252 and rings at 430,638 /674×313, and browser captures at their observed DOM bounds. It normalizes source/implementation widths to 760px for strips and 480px for rings. `docs/design/compact-comparison.png` stacks source strip, implemented strip, source rings and implemented rings. All four surfaces were viewed together. `compact-delivery.png` is the two implemented modes, enlarged for inspection.
- Focused crops `compact-bars.png`, `compact-rings.png`, `compact-light.png`, `compact-narrow.png` show all widget content; no additional focus crop was necessary.

## Findings and repair history

1. [P2, fixed] Quota/reset labels clipped to 4.5px because the strip's flex children shrank. Reduced gaps/padding, gave captions non-shrinking 12px lines; final screenshots show intact labels. DOM checks confirm no vertically clipped captions at 360px.
2. [P2, fixed] A shared button SVG rule shrank the circular gauge to 17px. Scoped the gauge SVG to 48×48; final DOM measurements and screenshots confirm full rings with centred percentages. Utility icons remain 15px.
3. [P2, fixed] Disabled preview accounts could expose old cached percentages in the fallback tile. Disabled accounts now select no quota; zero enabled connections renders one dash/Connect tile even when cached demo data exists.
4. [P2, fixed] Provider columns and the native minimum size forced a large window. Connected-account grid count, computed view width and native minimum sizes now permit 214px one-account strips and 144px one-account rings. Native ResizeObserver measures content width and height so panels can both expand and shrink.
5. Development-only Vite cached an empty module during a file rewrite. Refreshed the development server and recaptured final screenshots. Production TypeScript/Vite and native builds succeed; fresh browser verification has no new errors.

## Fidelity and intentional choices

Single small glass surface, rounded 12px border, provider marks/names, thin bars, centred mini rings and right-side controls follow the chosen references. The generated reference is illustrative: implementation keeps separate quota and reset/warning lines, the original sample values, an explicit low-remaining label, and a browser disclosure. Ordinary Codex is named Codex. Real licensed SVG marks and Phosphor controls are retained; no generated screenshot substitutes for functional UI. Light/opaque variants use high-contrast text and darker gauge accents. Reduced-motion rules disable animation and ring transitions.

At most three columns are used; further accounts wrap in a bounded scrolling region. Compact captions ellipsize long quota/reset labels; complete account/workspace, source, timestamp and reset remain in accessible tile descriptions, tooltips and expanded details. Settings always retain all configured accounts even when their widget tiles are hidden.

## Interactions and state verification

Verified: bars/rings switch; two, one and zero enabled accounts; three-account layouts at 360px with no page overflow; disabled cache suppression; Connect leading to Settings; Settings expanding and closing; theme/light/opaque controls; expanded quota details and pinning; Refresh button; all utility controls labelled and keyboard focus styled. All-account deletion fallback and visibility independent of transient poll errors have automated regression coverage. Pinning never silently substitutes another missing quota.

No actionable P0/P1/P2 findings remain. Browser error logs for the fresh final document are clear. Tests: 14 frontend/DOM cases and 17 Rust tests (including Claude scope and Codex window-identity regressions); strict TypeScript/Vite, Clippy and formatting pass. Windows installer/portable builds succeed.

## Follow-up polish / validation gaps

- [P3] Assess actual Windows display scaling, optical spacing and truncation at 125/150/200%.
- Native drag/snap, multiple monitors, tray, startup and alerts still need physical interaction checks; this browser QA does not certify them.
- Gemini native isolated sign-in/reload and live Claude Enterprise member comparisons remain outstanding. Codex polling is live-verified separately in `docs/reference-comparison.md`.

final result: passed
