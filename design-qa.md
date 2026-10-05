# Combined TokenFuel bar — design QA

Date: 2026-10-05. Scope: existing Windows application frontend and native window/configuration changes.

## Evidence and normalization

- Visual truth: `docs/design/combined-bar-reference.png` (1774 × 887 design board).
- Browser implementation: `http://127.0.0.1:1420/?demo=1&providers=claude,openai,gemini`, fictional sample data, dark theme.
- Full browser captures: `docs/design/strip-browser-full.jpg`, `details-browser-full.jpg`, `focus-browser-full.jpg`; 1280 × 720 CSS/pixels, devicePixelRatio 1.
- README captures: `docs/images/widget-strip.jpg` (492 × 77), `widget-details.jpg` (492 × 365), `widget-focus.jpg` (312 × 77). Cropped from full browser captures, including margins and the explicit Sample data label.
- Component measurements: strip 480 × 48 CSS px, focus 300 × 48 CSS px, Gemini detail panel 340 × 280 CSS px.
- Combined comparison input, opened and inspected: `docs/design/combined-comparison.png`. Left column contains source crops; right column contains the corresponding implementation crops. Rows are all accounts, Codex focus, Gemini details. Source crops are scaled to equal widths with aspect ratios preserved, rather than squashed to the implementation height.
- The source board's illustrated bars scale to approximately 37/34 px tall at the agreed widths. The implementation follows the explicit 48 px height from the brief. The detail panel is taller than the mock because it retains source, scope, reading age, quota pinning and a close control from the real application. These are intentional product constraints.
- The component-level combined comparison serves as the focused comparison: all labels, percentages, meter fills and controls are readable at native scale. No reliance on the framed board alone.

## Findings and comparison history

1. **[P2, fixed] Third provider's weekly quota scrolled out of the initial three-provider strip.** Initial in-chat browser capture showed only Gemini's 5h value. Reduced tile padding and quota minimum widths. The final strip has a 439 px account viewport and fits its full account content without scrolling. The final comparison shows all five quotas and the persistent menu.
2. **[P2, fixed] Long quota names collided with values and additional provider icons disappeared on navy.** Evidence before correction: `docs/design/before-long-label-fix.jpg`, eight accounts at 1280 × 720. Corrected grid intrinsic sizing and neutral icon filtering. Post-fix evidence: `docs/design/many-accounts-browser-full.jpg`, eight accounts at the stricter 320 × 720 viewport. All 18 quota nodes remain present, all measured label/value pairs have no overlap, last account is reachable through horizontal scrolling and the menu remains on-screen. Antigravity's icon is now visible.
3. **[P2, fixed] Detail value/reset layout was taller and less faithful than the selected design.** Replaced a full-width meter plus repeated source/reset caption with the source's meter-left/reset-right arrangement. Kept source and age on a secondary line. Added spacing so the reset caption does not collide with that metadata. Recaptured the final 340 × 280 panel and inspected the combined comparison after the fix.

No actionable P0/P1/P2 findings remain in the verified frontend.

Code review follow-up: fixed the focus-menu warning when hidden readings age without a new poll; constrained native resize placement to the monitor work area; protected newly saved disabled accounts from automatic discovery; cleaned up late event subscriptions and surfaced drag failures. The VS Code debugger now builds the app and waits for Vite instead of starting a second desktop instance. Corrected installer instructions to distinguish published 0.1.0 setup from unreleased automatic discovery.

## Required fidelity surfaces

- **Typography:** Segoe UI, 11 px short window labels, 13 px strip values, 15 px detail values and provider heading. Numeric values use tabular figures. Compact text is deliberately readable at the agreed 48 px footprint. Named feature/model quotas retain their names and scroll instead of overlapping or collapsing into generic periods. Detail remaining/reset copy has explicit spacing.
- **Spacing and layout:** one navy strip, light dividers, rounded 10 px surface, individual meters, one overflow control, narrower focus mode with account dropdown. Detail panel has the same hierarchy and primary Focus this account action as the mock. Source frame titles and descriptions remain outside the product. The decorative mock pointer is omitted; the active provider and panel heading identify the selected account.
- **Colors and tokens:** navy #172537, peach/teal/lavender meters, amber low readings. Light theme uses darker meter and icon colors. Stale readings are dimmed and visibly marked. Meter lengths use actual percentages, including zero; unknown percentages are never inferred. The mock's imprecise painted meter lengths are not copied into numerical behavior.
- **Image and icon quality:** existing supplied provider SVG assets are retained; action icons use the existing Phosphor library. No rasterized UI, generated substitute logos, custom SVG artwork or placeholder assets. Additional monochrome icons have contrast in dark and light themes.
- **Copy/content:** separate quota labels and remaining values; full quota/product/scope/source/age in details; provider-supplied reset dates only. Sample data is explicitly labelled. Manual readings have a visible pencil marker. Focus selection is by account ID, so duplicate providers remain separate. README distinguishes the unreleased UI from the existing installer.

## Interactions and validation

Browser checks: provider details open/close, Escape dismissal with focus restoration, quota pin/unpin reordering without losing windows, Focus this account, account switching and All accounts, other-account low indicator, overflow menu, refresh, bar/ring switching, Settings, theme change, and eight-account scrolling at 320 px width. Browser error/warning logs were checked and empty during verification. Stale treatment was also observed after sample readings aged.

Automated checks: 33 frontend tests, 40 Rust tests, frontend production build, Rust Clippy with warnings denied, Rust formatting, and Git whitespace checks. The optimized Windows build also passed (`pnpm tauri build --no-bundle`), producing `target/release/tokenfuel.exe`. Integration regression covers saved focus restoration, selection switching, disabled-account fallback and closing details for a removed/disabled selection. Persistence regression covers old settings, saved focus and clearing it without resetting other preferences. Review regressions cover hidden stale readings and resize placement at screen edges, including monitors with negative origins.

## Implementation checklist

- [x] Combined strip, details and focus mode in the existing application.
- [x] Preserve independent pools, unknown/unlimited readings and freshness.
- [x] Persist focus and retain compatible old settings.
- [x] Adapt native initial/minimum height to the 48 px surface plus 12 px margins.
- [x] Save actual application screenshots and update README/features.
- [x] Inspect corrected source/implementation comparison and complete required checks.

## Follow-up polish and test limits

P3: a pointer anchored to the clicked provider could be added to the detail panel. The current panel is compact and labelled but always aligns to the right edge.

Windows tray, startup, live provider authentication, native dragging and monitor-edge placement were not exercised through native UI in this pass. Existing native implementations are retained; Rust compilation/tests validate the configuration change. Browser preview uses sample data and does not certify live subscription coverage. No new installer was packaged or published.

final result: passed
