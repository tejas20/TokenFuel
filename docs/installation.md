# Install TokenFuel on Windows

**Download the installer, run it, and choose your provider.** No coding, repository clone, Git, GitHub CLI, Node.js, Rust, pnpm or Python is needed. This release supports Windows 10/11 x64. Native ARM64 and 32-bit builds are not included.

## Recommended installation

1. Open [TokenFuel 0.1.0](https://github.com/tejas20/TokenFuel/releases/tag/v0.1.0) while signed in to GitHub with access to the private repository.
2. Download **TokenFuel_0.1.0_x64-setup.exe** and run it. It installs for your Windows user; the app itself does not require administrator access.
3. Open TokenFuel from the Start menu. Setup opens automatically when no account is connected. Choose the provider, allow the connection, and Save. Experimental sources also need their separate opt-in.

The installer includes Microsoft's small WebView2 bootstrapper. It checks for the runtime and installs it if missing; that step needs internet. WebView2 Evergreen receives Microsoft's runtime updates. Existing WebView2 is reused. [Tauri installer options](https://v2.tauri.app/distribute/windows-installer/), [Microsoft runtime distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

The repository remains private. This is a published release, so collaborators with **read access** can download it; push access is not required as it was for the older drafts. Non-collaborators cannot download private assets. Making it public is a separate decision.

## Offline or managed machines

Use **TokenFuel_0.1.0_x64-offline-setup.exe** when installation must work without downloading WebView2. It contains Microsoft's larger Evergreen standalone runtime installer. Quota services still need internet after installation. Organization policy can require IT approval or block a runtime install; TokenFuel does not bypass those controls.

These builds are unsigned. Windows may show an unknown publisher or SmartScreen warning. Publisher signing is the next distribution improvement; checksums do not replace it.

## Connect without copying tokens

Supported connections look for known local app/CLI sign-ins **after you allow access**. Installing TokenFuel alone does not authorize credential reads.

| Provider | Simplest connection | Coverage |
|---|---|---|
| OpenAI | Existing signed-in Codex app or CLI | Codex pools; ordinary ChatGPT model counters unavailable |
| Claude | Existing signed-in Claude Code CLI | Experimental; enterprise monthly verification pending |
| Cursor | Existing signed-in Cursor app | Experimental; live verification pending |
| GitHub Copilot | Existing Copilot CLI or GitHub CLI sign-in | Live verification pending |
| Grok | Existing Grok CLI sign-in | Experimental Build credits; ordinary chat counters unavailable |
| Google Antigravity | Existing local sign-in | Experimental; live verification pending |
| OpenCode Go | Existing monitor configuration if present | Experimental; otherwise needs workspace ID/session cookie in Advanced settings |
| Gemini | Save permissions, open isolated Usage window and sign in | Experimental; keep window open, native polling verification pending |

Automatic discovery may find an installation without a usable subscription/session. If it cannot read usage, the widget gives a connection status and keeps cached data visibly aged. Compare a new source with the provider's Usage page before relying on it.

**Advanced connection settings** contains workspace IDs, the optional Codex executable path, and account-specific secrets for supported sources. Most local sources do not need these. Secrets use Windows Credential Manager and never appear in the quota UI or config file. If a custom secret goes missing, the connection stops instead of switching to another account. Disconnecting, changing source or removing the account deletes TokenFuel's saved secret. Original app credentials are not rewritten.

Browser sign-ins are isolated and temporary. SSO can reject embedded sign-in; manual snapshots remain available. Startup and alerts are opt-in. Run the app again to bring its existing widget forward; the tray menu can show/hide, refresh or quit it.

## Portable alternative

Extract **TokenFuel_0.1.0_x64-portable.zip** and double-click `TokenFuel.exe`. WebView2 must already be installed; otherwise use either installer. Configuration and quota cache still use Windows application storage for `com.tejas20.tokenfuel`; supported secrets use Windows Credential Manager. Portable does not mean settings or secrets travel with the ZIP.

## Verify and update

Download **SHA256SUMS.txt** from the same release and compare the SHA-256 entry for your exact installer or ZIP. PowerShell example:

```powershell
Get-FileHash .\TokenFuel_0.1.0_x64-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

A mismatch means the download must not be run. Matching hashes check file integrity against the release, not publisher identity.

There is no automatic updater yet. Quit using the tray menu, download the newer installer and run it. Existing settings are retained. Settings shows the running app's version. Downloaded binaries do not automatically include later `main` commits.

The optional `Start-TokenFuel.cmd` launcher is for private testers who already use GitHub CLI. Authenticate with `gh auth login` and run it from a checkout; it downloads and verifies the portable ZIP pinned to `v0.1.0`, then caches it under `%LOCALAPPDATA%\TokenFuel\Preview\v0.1.0`. It requires WebView2 and does not automatically update. `-DownloadOnly` downloads and verifies without starting. Normal users should choose the installer.

## Troubleshooting

- **Release missing / 404:** sign in with an account invited to this private repository. Published releases need read access; older drafts need push access.
- **Runtime missing / blocked download:** try the offline installer or ask your IT administrator.
- **Usage unavailable:** follow the source's setup hint, sign in to the provider's app/CLI, authorize the source and Refresh. The app does not expose quota counters a provider does not supply.
- **App appears hidden:** run it again or use the tray menu.

See [release validation](release-validation-v0.1.0.md) and the [office checklist](office-validation.md) for coverage and remaining checks.
