# TokenFuel 0.1.0

First published release in this private repository. Windows 10/11 x64. Early release: experimental provider connections remain opt-in.

## Install

Download **TokenFuel_0.1.0_x64-setup.exe**, run it, and open TokenFuel from the Start menu. No Git, Node.js, Rust, pnpm, Python or GitHub CLI is needed. The app installs for your Windows user. If WebView2 is missing, the embedded Microsoft bootstrapper installs it with internet access.

Choose **TokenFuel_0.1.0_x64-offline-setup.exe** for a machine that cannot download WebView2 during installation. It includes Microsoft's Evergreen runtime installer. Usage services still need internet. Managed devices can require IT approval.

**TokenFuel_0.1.0_x64-portable.zip** runs after extraction when WebView2 is already present. **SHA256SUMS.txt** includes SHA-256 hashes for all three downloads. These builds are unsigned; hashes do not replace publisher signing.

Quit an older widget from its tray menu before installing. Existing settings and account permissions are retained.

## Included

- Compact bars and rings, separate quota windows, reset countdowns, cache age, tray controls and optional Windows startup/alerts.
- OpenAI Codex, Claude Code and Gemini usage sources, plus new OpenCode Go, Cursor, Grok Build, GitHub Copilot and Google Antigravity adapters.
- Setup opens when no accounts are connected. Connection instructions describe automatic sign-in reuse; workspace and secret fields are tucked under Advanced settings.
- Existing local sign-ins are read only after permission. Optional account secrets are protected by Windows Credential Manager; missing custom secrets fail instead of falling back to a different account. Changing source or disconnecting removes the app's stored secret.
- Reopening the app brings the existing widget forward.

## Limits

Only Codex live quota reads have previously been verified on this PC. The new provider parsers have synthetic regression coverage, not certification across accounts or plans. Claude Enterprise, native Gemini isolated sign-in/polling, SSO, new-provider live expiry/rate-limiting and physical monitor/DPI/startup/notification scenarios need further validation. Gemini browser DOM extraction was verified separately; it does not certify native sign-in.

OpenAI tracks Codex pools, not every ordinary ChatGPT model. Grok tracks Build credits, not ordinary chat. Browser sign-ins are isolated and temporary. There is no automatic updater. ARM64-native and 32-bit binaries are not included. Download access requires read access to this private repository; repository visibility is unchanged.

See the [installation guide](https://github.com/tejas20/TokenFuel/blob/main/docs/installation.md) and [release validation](https://github.com/tejas20/TokenFuel/blob/main/docs/release-validation-v0.1.0.md).
