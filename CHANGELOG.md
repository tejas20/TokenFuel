# Changelog

## Unreleased

- Replace tall quota tiles with a 48 px horizontal strip capped at 480 px wide, with independent meters for every quota and horizontal scrolling for larger collections.
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
