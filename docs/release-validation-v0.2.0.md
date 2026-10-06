# Validation for TokenFuel 0.2.0

Date: 6 October 2026. Windows x64 public prerelease, maintained by @tejas20. The release includes the compact quota strip from `48096fc` plus version, public-download, sponsorship and contributor/security documentation changes. Concurrent local source edits made after the frontend build are excluded from this release.

## Automated checks

- `pnpm install --frozen-lockfile`: succeeded with the checked-in lockfile. A pnpm update-metadata fetch failed; it did not affect dependency installation.
- `pnpm test`: **33 tests passed** across seven frontend/DOM suites.
- `cargo test --workspace --locked`: **40 tests passed** (19 native app tests, 21 core tests), including isolated Windows Credential Manager and SQLite tests with synthetic data.
- `pnpm build`: strict TypeScript and production Vite build passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Windows PowerShell parsed the updated public launcher without errors.

## Packaging and native checks

The packaging script built the regular NSIS installer, offline NSIS installer, portable ZIP and `SHA256SUMS.txt` for 0.2.0. All three package hashes were verified locally. The ZIP contains only `TokenFuel.exe`, the MIT license, third-party notices and the portable README.

Both installer variants completed silent installation/upgrade successfully (exit code 0) for the current Windows user. The installed product reports 0.2.0; the regular and offline variants install identical application code. Existing account IDs, sources and preferences survived the upgrade.

The native widget was visually inspected using Computer Use. The compact strip displayed live Codex quota windows and explicit connection issues for unavailable sources. The menu and account focus worked; focus was saved to configuration and survived the offline installer upgrade/relaunch. Original preferences were restored after testing. A duplicate launch exited while retaining exactly one original widget process. The extracted portable executable was also launched on this PC with WebView2 already installed.

No real provider secrets, screenshots or local configuration were added to the release or repository. Local upgrade verification files remain in ignored `artifacts/` storage.

## Public repository review

- All reachable Git refs were scanned with Gitleaks 8.30.1: no secret findings in repository history. Historical commit authors use the owner's GitHub noreply address.
- Forty historical non-icon raster images were visually reviewed: design references and sample-data application previews, without visible private account information.
- All sixteen existing Actions logs were downloaded and scanned separately: no secret findings. Historical CI upload paths and packaging inputs were reviewed; they contain app binaries/packages rather than local credentials or configuration. Historical CI binary artifacts were not individually downloaded/rescanned.
- All five existing release portable ZIPs were downloaded, checked against their published checksums and inventoried. Their included text documents passed secret scanning. Release notes were reviewed. Draft previews remain unpublished.

This is a scoped publication review, not a guarantee that every possible secret pattern can be detected.

## Limits

WebView2 was already present on this PC. Offline packaging and wrapper installation do not prove installation on a clean machine missing the runtime or behind managed-device restrictions. Windows sign-out/startup, real threshold notifications, multiple physical monitors/DPI, tray interaction and SSO were not newly certified in this release.

Most provider adapters have synthetic coverage; live validation across plans/accounts remains pending. Claude Enterprise office checks and Gemini native isolated sign-in/sustained polling remain outstanding. Codex counters do not cover ordinary ChatGPT models, and Grok Build credits do not cover ordinary Grok chat. Binaries are unsigned, there is no automatic updater, and native ARM64/32-bit builds are not included. These limitations are why 0.2.0 is a prerelease.
