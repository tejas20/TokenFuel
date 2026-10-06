# Validation for TokenFuel 0.2.0

Date: 6 October 2026. Windows x64 release, maintained by @tejas20. The rebuilt release includes the reviewed current source: wrapping quota rows, visible Move/Refresh controls, shortened generic labels, preserved Antigravity groups, Windows native TLS and corrected connection errors. It replaces the earlier 0.2.0 prerelease at the maintainer's request and is published as Latest. Earlier packaging/native checks below are historical unless specifically identified as rebuild checks.

## Automated checks

- `pnpm install --frozen-lockfile`: succeeded with the checked-in lockfile for the rebuild.
- `pnpm test`: **45 tests passed** across seven frontend/DOM suites.
- `cargo test --workspace --locked`: **41 tests passed** (19 native app tests, 22 core tests), including isolated Windows Credential Manager and SQLite tests with synthetic data.
- `pnpm build`: strict TypeScript and production Vite build passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Windows PowerShell parsed the updated public launcher without errors.

## Packaging and native checks

### Rebuild checks

The complete Tauri production packaging build passed. Regular and offline NSIS installers both upgraded the existing 0.2.0 installation silently with exit code 0 and installed identical executables. All account configurations and preferences survived both upgrades and the extracted portable launch. The installed and portable executables contain identical application code; the only three differing bytes are Tauri's documented `NSS` versus `UNK` bundle marker.

All three rebuilt package SHA-256 hashes were verified. The portable ZIP contains the executable, README, project license, third-party notices and full Lobe Icons, Primer Octicons and Phosphor license texts. Its extracted executable launched successfully; a duplicate launch exited with code 0 and retained one running widget.

The installed widget was visually checked for wrapping quota rows, visible Move/Refresh controls and a working menu. The extracted portable showed saved Codex focus. The production browser preview displayed all eighteen sample quota windows without horizontal clipping in bar view; ring view was visually checked. Live Codex, Copilot and Antigravity reads succeeded on this PC during the rebuild checks.

Code review covered all changed runtime, parser, UI and packaging files. Fixed an overridden CSS row gap and missing portable icon license texts. Updated the launcher's cache directory so this same-version rebuild cannot reuse the earlier 0.2.0 download. No release-blocking findings remained after these repairs. PowerShell parsing and `git diff --check` passed. Original published assets were backed up locally and verified before replacement.

### Earlier prerelease checks

The packaging script built the regular NSIS installer, offline NSIS installer, portable ZIP and `SHA256SUMS.txt` for 0.2.0. All three package hashes were verified locally. The ZIP contains only `TokenFuel.exe`, the MIT license, third-party notices and the portable README.

Both installer variants completed silent installation/upgrade successfully (exit code 0) for the current Windows user. The installed product reports 0.2.0; the regular and offline variants install identical application code. Existing account IDs, sources and preferences survived the upgrade.

The native widget was visually inspected using Computer Use. The compact strip displayed live Codex quota windows and explicit connection issues for unavailable sources. The menu and account focus worked; focus was saved to configuration and survived the offline installer upgrade/relaunch. Original preferences were restored after testing. A duplicate launch exited while retaining exactly one original widget process. The extracted portable executable was also launched on this PC with WebView2 already installed.

No real provider secrets, screenshots or local configuration were added to the release or repository. Local upgrade verification files remain in ignored `artifacts/` storage.

## Public repository review

- All reachable Git refs were scanned with Gitleaks 8.30.1: no secret findings in repository history. Historical commit authors use the owner's GitHub noreply address.
- Forty historical non-icon raster images were visually reviewed: design references and sample-data application previews, without visible private account information.
- All sixteen existing Actions logs were downloaded and scanned separately: no secret findings. Historical CI upload paths and packaging inputs were reviewed; they contain app binaries/packages rather than local credentials or configuration. Historical CI binary artifacts were not individually downloaded/rescanned.
- All five existing release portable ZIPs were downloaded, checked against their published checksums and inventoried. Their included text documents passed secret scanning. Release notes were reviewed. Draft previews remain unpublished.

This is a scoped publication review of the earlier prerelease, not a guarantee that every possible secret pattern can be detected.

## Limits

WebView2 was already present on this PC. Offline packaging and wrapper installation do not prove installation on a clean machine missing the runtime or behind managed-device restrictions. Windows sign-out/startup, real threshold notifications, multiple physical monitors/DPI, tray interaction and SSO were not newly certified in this release.

Copilot and Antigravity live reads were also verified on this PC after the native TLS fix; see [provider connection validation](provider-connection-validation.md). Most other provider adapters have synthetic coverage; live validation across plans/accounts remains pending. Claude Enterprise office checks and Gemini native isolated sign-in/sustained polling remain outstanding. Codex counters do not cover ordinary ChatGPT models, and Grok Build credits do not cover ordinary Grok chat. Binaries are unsigned, there is no automatic updater, and native ARM64/32-bit builds are not included. Marking the release Latest does not certify experimental adapters or remove these limitations.
