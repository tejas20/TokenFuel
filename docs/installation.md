# Install TokenFuel on Windows

Use the prebuilt installer for the simplest setup. Building from source and cloning the repository are optional. All current builds require **Windows 10/11 x64**.

| Route | What you need | Best use |
|---|---|---|
| Installer (recommended) | Browser access to the release; internet if WebView2 needs installing | Normal desktop use |
| Portable ZIP | Browser access to the release; WebView2 already installed | Testing without installing the app |
| Checksum-verified launcher | Launcher files; authenticated GitHub CLI for the first download; WebView2 already installed | Private preview testers with CLI tooling |
| Build from source | Git, Node.js, pnpm, Rust, C++ tools, Windows SDK and WebView2 | Development; see [setup](setup.md) |

Git, GitHub CLI, Rust, Node.js, pnpm and Python are not app runtime dependencies. Individual tracking sources may need their own tools: automatic OpenAI tracking currently reads an installed, signed-in **Codex** app-server; simply installing the ChatGPT app does not provide those counters. See [provider support](../README.md#provider-support).

## Current private preview access

Open [TokenFuel releases](https://github.com/tejas20/TokenFuel/releases) while signed in to GitHub. The current preview is `v0.1.0-compact.2`, with these assets:

- `TokenFuel_0.1.0_x64-setup.exe`
- `TokenFuel_0.1.0_x64-portable.zip`
- `SHA256SUMS.txt`

The repository is private and the release is a **draft prerelease**. An account with push access can access the draft; ordinary repository read access does not expose draft releases. A missing release or a 404 can therefore mean insufficient access, not a missing build. GitHub CLI authentication must use an account with that same access. [GitHub draft release access](https://docs.github.com/en/rest/releases/releases#list-releases).

For broader private testing, a maintainer can publish a prerelease within the still-private repository so invited readers can download it. That is a future distribution step; the current release remains a draft and the repository remains private. Public distribution requires a separate explicit decision.

## Recommended: installer

1. Download `TokenFuel_0.1.0_x64-setup.exe` from the release in your browser.
2. Run the installer and follow its prompts. TokenFuel is configured to install for the current Windows user.
3. Open TokenFuel from the Start menu. In Settings, choose and authorize a provider source, then Save. Startup is opt-in.

The installer is configured to download and install the WebView2 runtime when it is missing. This step needs internet access; no developer tools are installed. [Tauri Windows installer documentation](https://v2.tauri.app/distribute/windows-installer/).

The preview is unsigned. Windows may show an unknown-publisher warning; checksum verification is not publisher signing. A signed installer is the intended route for wider distribution.

## Alternative: portable ZIP

Download `TokenFuel_0.1.0_x64-portable.zip`, extract it to a folder you own, and double-click `TokenFuel.exe`. Keep the extracted files together. WebView2 must already be installed; if it is missing, use the installer.

“Portable” means the app can run without its installer. Configuration and quota cache still use the Windows application configuration directory for `com.tejas20.tokenfuel`; supported secrets use Windows Credential Manager. Browser sessions are temporary. See [local data and security](../README.md#local-data-and-security).

## Optional: checksum-verified launcher

Install [GitHub CLI](https://cli.github.com/) and authenticate with `gh auth login`. Git is needed only if you obtain the launcher by cloning the repository. From a parent folder without a `TokenFuel` checkout:

```powershell
gh repo clone tejas20/TokenFuel
if ($LASTEXITCODE -eq 0) { .\TokenFuel\Start-TokenFuel.cmd }
```

From an existing checkout, start with one command:

```powershell
.\Start-TokenFuel.cmd
```

The launcher downloads the portable ZIP and `SHA256SUMS.txt` for the pinned `v0.1.0-compact.2` release using GitHub CLI. It verifies the ZIP before extraction, saves the executable's checksum locally, and rechecks that checksum before later launches. A valid cached copy starts without another download or GitHub authentication. The cache is `%LOCALAPPDATA%\TokenFuel\Preview\v0.1.0-compact.2`.

The CMD wrapper applies its PowerShell execution setting to that process only; it does not change machine policy. Provider connections and Windows startup still require the user's choices. To download and verify without launching:

```powershell
.\Start-TokenFuel.cmd -DownloadOnly
```

Checksums detect a mismatch against the release's checksum file. Because both come from the same release, they do not independently authenticate the publisher or replace code signing. The cached checksum is also a local integrity check, not a signature.

## Verify a browser download

Download `SHA256SUMS.txt` from the same release and compare the entry for the exact installer or ZIP with PowerShell's output. For example, from the download folder:

```powershell
Get-FileHash .\TokenFuel_0.1.0_x64-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

The hashes must match, ignoring letter case. Do not run a download whose hash differs. Browser downloads do not run the launcher's checksum check automatically.

## Updates and troubleshooting

There is no automatic updater in this preview. Quit TokenFuel using its tray menu before installing a newer release or replacing portable files. Download the intended version from Releases and verify its checksum. The launcher stays on its pinned preview even after new assets or source commits appear; a later launcher must explicitly target a newer release. Released binaries do not automatically include subsequent changes on `main`.

For an existing source checkout, quit the older running app, run `git pull --ff-only`, then `.\Start-TokenFuel.cmd` to use the checked-in launcher's new version. The `v0.1.0-compact.2` preview fixes hidden weekly/monthly windows: each reported pool is visible in bars and rings without opening details. A provider that reports no monthly quota still has no monthly counter; Claude Enterprise live monthly validation remains outstanding.

- **Release missing / download denied:** check the signed-in account and draft-release access. For CLI downloads, run `gh auth status` and authenticate the correct account.
- **GitHub CLI missing:** use the browser installer, or install GitHub CLI if you prefer the launcher.
- **WebView2 missing:** use the installer; portable and launcher routes do not bootstrap it.
- **App already running:** use the TokenFuel tray menu to show the widget.
- **Usage unavailable:** open Settings, authorize a supported source, and check its connection status. Installing the app alone does not authorize provider tracking. Follow the [connection instructions](../README.md#connect-accounts).

For office testing, follow the [validation checklist](office-validation.md).
