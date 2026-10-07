# Install TokenFuel on Windows

Version 0.2.0 is the public Windows release with wrapping quota rows, Menu → Refresh → Move controls, simplified menus and details, account focus, automatic local detection and provider connection fixes. Controls stack vertically for multiple visible providers and stay horizontal for one provider or a focused account. Ring view and quota pinning are removed. Experimental provider limitations still apply. See the [changelog](../CHANGELOG.md).

**Download the installer, run it, and review detected connections in Settings.** No coding, repository clone, Git, GitHub account, GitHub CLI, Node.js, Rust, pnpm or Python is needed. This release supports Windows 10/11 x64. Native ARM64 and 32-bit builds are not included.

## Recommended installation

1. Open [TokenFuel 0.2.0](https://github.com/tejas20/TokenFuel/releases/tag/v0.2.0) in your browser. Downloads are public and need no GitHub sign-in.
2. Download **TokenFuel_0.2.0_x64-setup.exe** and run it. It installs for your Windows user; the app itself does not require administrator access.
3. Open TokenFuel from the Start menu. The usage strip opens first and detects supported local sources automatically, including experimental adapters. Use Settings to review connection issues or disable sources. Gemini still requires an isolated browser sign-in.

The installer includes Microsoft's small WebView2 bootstrapper. It checks for the runtime and installs it if missing; that step needs internet. WebView2 Evergreen receives Microsoft's runtime updates. Existing WebView2 is reused. [Tauri installer options](https://v2.tauri.app/distribute/windows-installer/), [Microsoft runtime distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

Use the published 0.2.0 release for the current UI. Older releases are hidden as maintainer-only drafts.

## Offline or managed machines

Use **TokenFuel_0.2.0_x64-offline-setup.exe** when installation must work without downloading WebView2. It contains Microsoft's larger Evergreen standalone runtime installer. Quota services still need internet after installation. Organization policy can require IT approval or block a runtime install; TokenFuel does not bypass those controls.

These builds are unsigned. Windows may show an unknown publisher or SmartScreen warning. Publisher signing is the next distribution improvement; checksums do not replace it.

### SmartScreen or "Access is denied" on an office PC

These are distinct observations. **"Windows protected your PC"** is a SmartScreen reputation warning. The published 0.2.0 files are unsigned. Signing future releases identifies the publisher, but even a signed new release can still warn until it gains reputation. See [Microsoft's explanation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

**`Start-Process ... Access is denied`** means Windows refused to launch the executable. The screenshot alone cannot distinguish AppLocker/App Control, endpoint antivirus, file permissions, or another Windows restriction. It is not evidence that a provider connection failed. A missing WebView2 runtime is a separate prerequisite and does not establish the cause of this error.

With the updated checkout, inspect the cached executable without downloading or starting it:

```powershell
.\Start-TokenFuel.cmd -Diagnose
```

The launcher also saves diagnostics automatically after a failed launch and returns exit code 1. To inspect a downloaded installer or an extracted portable executable instead:

```powershell
.\Test-TokenFuel.ps1 -ExecutablePath "$env:USERPROFILE\Downloads\TokenFuel_0.2.0_x64-setup.exe"
```

The command prints a JSON report path in the temporary directory. It records Windows/PowerShell versions, the file path and SHA-256, signature status, Internet zone ID, WebView2 detection, and matching recent AppLocker/Code Integrity event IDs when readable. It does not read provider credentials or usage, launch the app, download files, or modify policy. Review local paths before sharing with IT. Missing or unreadable event logs do not prove an app is allowed; IT may need its endpoint-security console. [AppLocker events](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/applocker/using-event-viewer-with-applocker), [App Control events](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/operations/event-id-explanations).

Ask IT to review the exact release hash, publisher signature (once available), and permitted deployment location. The portable launcher runs from the current user's LocalAppData directory, which some managed policies restrict. Reinstalling the same unsigned binary with another downloader does not establish trust. Do not disable SmartScreen, change application-control rules, remove download markings, or run as administrator as a blanket workaround. If PowerShell itself is restricted, give IT the original error and downloaded installer directly.

### WinGet status

Claude Code Usage Monitor documents a published WinGet package. TokenFuel now has manifest-generation tooling, but **this change does not publish a TokenFuel WinGet package**. Do not expect `winget install tejas20.TokenFuel` to work until the manifests have been submitted and accepted into the WinGet community repository. See the [maintainer steps](releasing.md#windows-signing-and-winget).

WinGet offers a consistent installation route and verifies the manifest's installer hash. It still runs the installer and does not exempt the app from office policy or remove TokenFuel's WebView2 requirement. The other monitor working via WinGet does not prove TokenFuel's executable is approved.

## Connect without copying tokens

At launch, TokenFuel checks known local app/CLI locations and supported credential stores, then enables detected sources for quota reads. Previously disabled or removed connections are respected. Gemini needs a separate browser sign-in. See [discovery locations](discovery.md).

| Provider | Simplest connection | Coverage |
|---|---|---|
| OpenAI | Existing signed-in Codex app or CLI | Codex pools; ordinary ChatGPT model counters unavailable |
| Claude | Existing signed-in Claude Code CLI | Experimental; enterprise monthly verification pending |
| Cursor | Existing signed-in Cursor app | Experimental; live verification pending |
| GitHub Copilot | Existing Copilot CLI or GitHub CLI sign-in | Live read verified on this PC; wider plan validation pending |
| Grok | Existing Grok CLI sign-in | Experimental Build credits; ordinary chat counters unavailable |
| Google Antigravity | Existing local sign-in | Experimental; live read verified on this PC |
| OpenCode Go | Existing monitor configuration if present | Experimental; otherwise needs workspace ID/session cookie in Advanced settings |
| Gemini | Save permissions, open isolated Usage window and sign in | Experimental; keep window open, native polling verification pending |

Automatic discovery may find an installation without a usable subscription/session. If it cannot read usage, the widget gives a connection status and keeps cached data visibly aged. Compare a new source with the provider's Usage page before relying on it.

**Advanced connection settings** contains workspace IDs, the optional Codex executable path, and account-specific secrets for supported sources. Most local sources do not need these. Secrets use Windows Credential Manager and never appear in the quota UI or config file. If a custom secret goes missing, the connection stops instead of switching to another account. Disconnecting, changing source or removing the account deletes TokenFuel's saved secret. Original app credentials are not rewritten.

Browser sign-ins are isolated and temporary. SSO can reject embedded sign-in; manual snapshots remain available. Always on top defaults to on for new settings. Startup and alerts are opt-in. Preferences are saved immediately and restored at launch; Windows application failures appear in Settings. Run the app again to bring its existing widget forward; the tray menu can show/hide, refresh or quit it.

## Portable alternative

Extract **TokenFuel_0.2.0_x64-portable.zip** and double-click `TokenFuel.exe`. WebView2 must already be installed; otherwise use either installer. Configuration and quota cache still use Windows application storage for `com.tejas20.tokenfuel`; supported secrets use Windows Credential Manager. Portable does not mean settings or secrets travel with the ZIP.

## Verify and update

Download **SHA256SUMS.txt** from the same release and compare the SHA-256 entry for your exact installer or ZIP. PowerShell example:

```powershell
Get-FileHash .\TokenFuel_0.2.0_x64-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

A mismatch means the download must not be run. Matching hashes check file integrity against the release, not publisher identity.

There is no automatic updater yet. Quit using the tray menu, download the newer installer and run it. Existing settings are retained. Settings shows the running app's version. Downloaded binaries do not automatically include later `main` commits.

The optional `Start-TokenFuel.cmd` launcher works from a checkout without GitHub CLI or authentication. It downloads and verifies the public portable ZIP pinned to `v0.2.0`, then caches this rebuild under `%LOCALAPPDATA%\TokenFuel\Preview\v0.2.0\2026-10-06.3`. Update your checkout to use this new cache instead of an earlier 0.2.0 executable. It requires WebView2 and does not automatically update. `-DownloadOnly` downloads and verifies without starting. Normal users should choose the installer.

## Troubleshooting

- **Release missing / 404:** use the published `v0.2.0` release link above; older drafts are not public downloads.
- **Runtime missing / blocked download:** try the offline installer or ask your IT administrator.
- **Usage unavailable:** follow the source's setup hint, sign in to the provider's app/CLI, authorize the source and Refresh. The app does not expose quota counters a provider does not supply.
- **App appears hidden:** run it again or use the tray menu.

See [release validation](release-validation-v0.2.0.md) and the [office checklist](office-validation.md) for coverage and remaining checks.
