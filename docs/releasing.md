# Publishing a GitHub release

Use this checklist for the next release. Version 0.2.0 contains the compact quota strip and automatic local detection; older 0.1.0 downloads retain the earlier UI. Changes under **Unreleased** in [CHANGELOG.md](../CHANGELOG.md) are not part of published downloads.

1. Choose a new version and update `package.json`, `src-tauri/tauri.conf.json`, both applicable Cargo package versions and the lockfile. Update download names and links in the README, installation guide, portable README, and optional launcher if it should target the new release.
2. Run `pnpm install --frozen-lockfile`, `pnpm test`, `pnpm build`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo fmt --all --check` on Windows.
3. Run `powershell -File scripts/package-release.ps1`. Use the regular installer, offline installer, portable ZIP, and `SHA256SUMS.txt` in `artifacts/release/v<version>/`.
4. Smoke-test the packaged build: first-run detection, usage-first startup, disabling a detected account, rapid preference changes, quit/relaunch, Windows sign-out/restart with startup enabled, and Settings error reporting. Check an existing installation retains its settings. Compare live quotas with provider pages where accounts are available.
5. Move the changelog entries into the new version and use them as the release description. Include Windows 10/11 x64 requirements, WebView2 options, unsigned-binary status, and the provider limitations below. Mark the release as a prerelease while live validation is incomplete.
6. Create a GitHub draft release from the tested commit/tag and attach all four assets. Verify asset names and checksums. Publish when reviewed. Repository visibility is a separate maintainer decision; public downloads require a public repository or another public distribution location.

## Release-description essentials

- Download the regular installer for most PCs; choose the offline installer when WebView2 cannot be downloaded during setup. The portable ZIP requires WebView2 already installed.
- TokenFuel automatically connects detected local quota sources, including experimental adapters. Connections can be disabled in Settings. Gemini requires separate, session-only browser sign-in.
- Codex quotas do not cover all ChatGPT models; Grok Build credits do not cover ordinary Grok chat. Detection is not proof of working quota access for every plan. State which providers were actually tested for this release.
- Builds are currently unsigned. SHA-256 checksums verify integrity, not publisher identity. There is no automatic updater.
- Link the [installation guide](installation.md), [discovery guide](discovery.md), MIT [license](../LICENSE), and [third-party notices](../THIRD_PARTY_NOTICES.md).

Keep credentials, personal usage captures, local configuration, and machine-specific validation output out of release assets. The packaging script includes the portable app, license, notices, and portable README.
