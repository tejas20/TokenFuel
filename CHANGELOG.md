# Changelog

## Unreleased

## 0.2.0 — 2026-10-06

- Rebuild the published 0.2.0 downloads from the reviewed current source and show this version as GitHub's Latest release. Older releases remain hidden as drafts.
- Use Windows native TLS certificate validation for provider connections without disabling certificate checks; Copilot and Antigravity reads succeeded on this PC.
- Render empty-quota connection failures once in provider details.
- Refresh Copilot and Antigravity marks and include icon license notices in portable packages.
- Use a new launcher cache directory for this rebuild so an earlier 0.2.0 executable is not reused.
- Keep Move and Refresh controls visible beside the three-dot menu, including refresh progress and busy-state feedback.
- Wrap provider tiles and their quota windows onto visible rows instead of hiding them behind horizontal scrolling.
- Standardize generic quota labels as 5h, Week, Month and Day; use Chat, Code and Premium for Copilot's compact feature labels while keeping full descriptions in tooltips and details.
- Preserve Antigravity quota group names so shortened windows remain distinguishable across model pools.

- Publish the repository under its existing owner, @tejas20, with GitHub Sponsors funding links and contributor/security reporting guidance.
- Make the optional checksum-verifying launcher download public release assets without GitHub CLI or authentication.

- Replace tall quota tiles with compact provider rows capped at 544 px wide, with independent meters for every quota and wrapping for larger collections.
- Add provider detail panels, saved account focus, an account switcher, and a single menu for refresh, appearance, pinning and Settings. Keep warnings for other low accounts visible during focus.
- Add application screenshots and a guide to the combined bar in the README.
- Keep expanding panels inside the monitor work area and warn about stale readings even when another account is focused.
- Open on remaining usage instead of Settings.
- Detect supported local quota sources at launch and connect them automatically, including experimental sources. Respect disabled and removed accounts.
- Show connection and Windows preference issues in Settings, with an indicator on its button.
- Save individual preferences without overwriting other recent changes. Keep saved preferences even when Windows cannot apply them, and retry at launch.
- Default Always on top to on for new configurations. Preserve existing choices.
- Stop silently replacing unreadable or invalid configuration with defaults.
- Shorten settings instructions and add public release and discovery documentation.

Gemini still requires an isolated browser sign-in. Detection does not guarantee a valid subscription or available quota counters; live validation remains provider-specific.

## 0.1.0

Initial Windows x64 release with compact bars and rings, separate quota windows, tray controls, local provider adapters, browser/manual snapshots, and optional startup and alerts. See the [original release notes](docs/release-v0.1.0.md).
