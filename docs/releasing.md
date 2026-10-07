# Publishing a GitHub release

Use this checklist for the next release. The rebuilt 0.2.0 contains wrapping quota rows, visible controls and provider connection fixes. Older releases remain hidden as drafts. Changes under **Unreleased** in [CHANGELOG.md](../CHANGELOG.md) are not part of published downloads.

1. Choose a new version and update `package.json`, `src-tauri/tauri.conf.json`, both applicable Cargo package versions and the lockfile. Update download names and links in the README, installation guide, portable README, and optional launcher if it should target the new release.
2. Run `pnpm install --frozen-lockfile`, `pnpm test`, `pnpm build`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo fmt --all --check` on Windows.
3. Run `powershell -File scripts/package-release.ps1 -SigningConfig <local-signing-config.json>`. Use the regular installer, offline installer, portable ZIP, `SHA256SUMS.txt`, and `SIGNATURES.json` in `artifacts/release/v<version>/`. Unsigned local/CI builds require `-AllowUnsigned` and must be described as unsigned.
4. Smoke-test the packaged build: first-run detection, usage-first startup, disabling a detected account, rapid preference changes, quit/relaunch, Windows sign-out/restart with startup enabled, and Settings error reporting. Check an existing installation retains its settings. Compare live quotas with provider pages where accounts are available.
5. Move the changelog entries into the new version and use them as the release description. Include Windows 10/11 x64 requirements, WebView2 options, unsigned-binary status, and the provider limitations below. Prereleases do not appear as GitHub's Latest release. The rebuilt 0.2.0 is published as a regular release at the maintainer's request; experimental adapters and outstanding validation remain documented.
6. Create a GitHub draft release from the tested commit/tag and attach the three downloads and both metadata files. Verify asset names, checksums and signature status. Publish when reviewed. Repository visibility is a separate maintainer decision; public downloads require a public repository or another public distribution location. Use a new version for changed release bytes; do not replace an installer already referenced by a WinGet manifest.

## Release-description essentials

- Download the regular installer for most PCs; choose the offline installer when WebView2 cannot be downloaded during setup. The portable ZIP requires WebView2 already installed.
- TokenFuel automatically connects detected local quota sources, including experimental adapters. Connections can be disabled in Settings. Gemini requires separate, session-only browser sign-in.
- Codex quotas do not cover all ChatGPT models; Grok Build credits do not cover ordinary Grok chat. Detection is not proof of working quota access for every plan. State which providers were actually tested for this release.
- Builds are currently unsigned. SHA-256 checksums verify integrity, not publisher identity. There is no automatic updater.

For a newly signed release, replace the unsigned statement with its actual verified publisher and signature status. Signing does not certify office-policy acceptance or guarantee immediate SmartScreen reputation.
- Link the [installation guide](installation.md), [discovery guide](discovery.md), MIT [license](../LICENSE), and [third-party notices](../THIRD_PARTY_NOTICES.md).

Keep credentials, personal usage captures, local configuration, and machine-specific validation output out of release assets. The packaging script includes the portable app, license, notices, and portable README.

For a standalone local executable, use `pnpm tauri build --no-bundle`. Plain
`cargo build --release` does not enable Tauri's bundled-asset protocol in this
project and can produce an executable that opens the development URL instead.
Smoke-test the installed executable, not just a workspace preview; the normal
Start-menu shortcut continues to use the installed copy until it is updated.

## Windows signing and WinGet

The published 0.2.0 builds remain unsigned. The workflow in `.github/workflows/windows.yml` explicitly generates unsigned validation artifacts, not trusted release builds. This change adds release tooling; it does not acquire a certificate, publish new binaries, or submit a WinGet package.

1. Obtain or configure a trusted Authenticode signing identity. For a certificate available to Windows SignTool, copy `src-tauri/tauri.signing.example.json` to a local JSON file outside the checkout and release assets, replace the thumbprint, and use your signing provider's RFC 3161 timestamp URL. For hardware/cloud signing, use Tauri's structured `bundle.windows.signCommand` (`cmd` and `args`, with `%1` as the binary-path argument). Keep keys and credentials out of the repository and command arguments. See [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/).
2. Run `powershell -File scripts/package-release.ps1 -SigningConfig C:\secure\tokenfuel-signing.json`. `TOKENFUEL_SIGNING_CONFIG` can also specify that path. The same config reaches the regular and offline bundlers. `-SkipBuild` skips compilation but still rebundles both installers. The script checks both installers and the portable executable for valid timestamped signatures and the same signer before generating the ZIP and checksums. An absent config or failed signature stops signed packaging. Tauri handles signing the app and NSIS uninstaller during bundling.
3. Review `SIGNATURES.json` and independently inspect the actual binaries with `Get-AuthenticodeSignature`. `SIGNATURES.json` is a human-readable report, not a signed attestation. Download and test the release on a clean Windows machine and the office device; test WebView2 present and missing. A valid signature still needs to meet the organization's application-control policy. [Microsoft SmartScreen reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).
4. Packaging generates three WinGet manifests under `artifacts/release/v<version>/winget/manifests/t/tejas20/TokenFuel/<version>/`, using the final regular installer's SHA-256, NSIS type and current-user scope. To regenerate only the manifests, run `powershell -File scripts/new-winget-manifest.ps1 -ReleaseDirectory artifacts/release/v<version>`. Run `winget validate --manifest <manifest-directory>`. Generation and validation do not install or publish anything.
5. After publishing the exact installer bytes, download that public asset separately and verify its hash equals `InstallerSha256`. Test interactive and silent install, upgrade, uninstall, publisher/display-name matching, and settings retention in an approved test environment. Submit the three manifests under that directory structure to `microsoft/winget-pkgs`, following [Microsoft's manifest instructions](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest). The planned identifier is `tejas20.TokenFuel`; repository review must confirm it. Do not advertise an install command until the submission is accepted and the package is discoverable from the public `winget` source.

Run `powershell -File scripts/test-windows-distribution.ps1` for launcher and packaging regression checks. These use synthetic failures and files; they do not certify SmartScreen reputation, real certificate signing, or office-policy acceptance.
