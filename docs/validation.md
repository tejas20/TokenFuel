# Development preview validation

2026-10-02, Windows x64. This is a testable development preview, not certification of every provider or plan.

- Rust workspace tests: quota decimal arithmetic, overages, zero/unknown denominators, changed payloads, enterprise monthly-only payloads, scoped pools, stable pin identities, currency mismatches, retry backoff and hostile Retry-After values; provider-window navigation boundaries.
- Rust formatting and Clippy with warnings denied.
- TypeScript strict checking, Vite production build and frontend reset/countdown test.
- Browser-rendered bar/ring switching, low-remaining labels, settings, light/opaque appearance, quota expansion and pin state. Final fresh preview console contained no errors or warnings.
- Windows NSIS and portable executable compiled successfully. Native window/tray behaviour still requires device interaction checks; browser testing does not verify these.
- Private GitHub owner and visibility verified via authenticated GitHub CLI.

Live local Codex testing awaits the user's explicit session-access consent. The diagnostic asks for `--consent-local-session` before any read. Run `cargo run -p tokenfuel --bin tokenfuel-diagnose -- --consent-local-session` only after consenting. It returns quota numbers and reset timestamps, never credentials or account identity.

Gemini and Claude signed-in usage captures remain unverified; no authenticated provider view was available in the browser used for development. Claude Enterprise monthly verification requires the office laptop. Physical scaling, monitor placement, alerts, startup, provider session expiry and real HTTP rate limiting remain on the office test checklist. Synthetic parser tests are not substitutes for these live checks.

See `design-qa.md` for visual comparison evidence and `office-validation.md` for the remaining device/account checks. Release binaries are unsigned development builds; signing belongs to the public-release process.
