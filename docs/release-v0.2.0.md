# TokenFuel 0.2.0 — Windows public prerelease

The compact quota strip, provider details and saved account focus are now included in the Windows downloads. TokenFuel remains owned and maintained by [@tejas20](https://github.com/tejas20), with MIT-licensed source and optional [GitHub Sponsors](https://github.com/sponsors/tejas20) support.

## Install

Windows 10/11 x64. Download **TokenFuel_0.2.0_x64-setup.exe**, run it, then open TokenFuel from the Start menu. No developer tools or GitHub account are needed. The regular installer uses a Microsoft WebView2 bootstrapper if the runtime is missing; that installation step needs internet.

Use **TokenFuel_0.2.0_x64-offline-setup.exe** when WebView2 cannot be downloaded during setup. It contains the Evergreen standalone runtime installer. Provider usage services still need internet, and corporate policy may require IT approval.

Alternatively, extract **TokenFuel_0.2.0_x64-portable.zip** and open `TokenFuel.exe` with WebView2 already installed. **SHA256SUMS.txt** covers all three downloads. These binaries are unsigned; checksums verify integrity, not publisher identity.

Quit the old widget before installing. Existing settings are retained. There is no automatic updater; download and install newer releases manually. The optional `Start-TokenFuel.cmd` launcher downloads and verifies this public portable release without GitHub CLI or authentication.

## Changes

- A 48 px horizontal strip, separate meters for every reported quota and horizontal scrolling for larger account lists.
- Provider details with reset countdowns, source/freshness information, quota pinning and a saved focus mode.
- A menu for refresh, account focus, bars/rings, settings and moving the widget. Expanded panels stay within the monitor work area.
- Usage-first startup and detection of supported local sources, including experimental adapters. Disabled/removed sources stay disabled; Gemini still needs a separate isolated browser sign-in.
- Preference saves preserve other recent changes and show Windows application failures in Settings. Invalid configuration is reported rather than silently reset.
- Public download guidance, contribution/security reporting guides and a repository Sponsor button for @tejas20.

## Validation and limits

33 frontend tests and 40 Rust tests passed locally, together with the production TypeScript/Vite build, Clippy and Rust formatting. See [release validation](https://github.com/tejas20/TokenFuel/blob/main/docs/release-validation-v0.2.0.md) for packaging and native checks.

Codex live quota reads were previously verified on this PC. Gemini's quota parser was checked against a signed-in PRO Usage page; native isolated sign-in and sustained polling remain unverified. Most other provider adapters have synthetic fixture coverage, with live validation still pending. Compare experimental readings against your provider's Usage page before relying on them.

Codex quotas do not represent ordinary ChatGPT model allowances; Grok Build credits do not represent ordinary Grok chat. Claude Enterprise office checks, SSO, clean-machine missing-WebView2 installation, multiple physical monitors/DPI, Windows sign-out/startup and real notification crossings remain outstanding. Native ARM64 and 32-bit builds are not included.

See [installation](https://github.com/tejas20/TokenFuel/blob/main/docs/installation.md), [discovery](https://github.com/tejas20/TokenFuel/blob/main/docs/discovery.md), [license](https://github.com/tejas20/TokenFuel/blob/main/LICENSE) and [third-party notices](https://github.com/tejas20/TokenFuel/blob/main/THIRD_PARTY_NOTICES.md).
