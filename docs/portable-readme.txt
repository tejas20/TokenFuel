TokenFuel 0.2.0 for Windows 10/11 x64

Extract the ZIP and double-click TokenFuel.exe. No developer tools are needed.
If WebView2 is missing, use the regular installer or offline installer instead.

The usage widget opens first. Supported local sources are detected and enabled
automatically. Use Settings to review issues or disable connections. New provider integrations
are provisional and must be compared with your provider's actual Usage page.
OpenAI shows Codex allowances, not every ChatGPT model's usage.

Advanced settings are optional. Secrets are stored in Windows Credential
Manager. Local app sign-ins are read only. Browser sign-ins are temporary.
Startup and alerts are opt-in. Use the tray menu to show, hide, refresh or quit.

Configuration and quota cache use the Windows application data directory.
The ZIP is portable, but configuration and credentials are not carried with it.
Quit from the tray before updating. Updates retain your local configuration.

Downloads and full installation guide:
https://github.com/tejas20/TokenFuel/releases/tag/v0.2.0
https://github.com/tejas20/TokenFuel/blob/main/docs/installation.md

Check TokenFuel.exe's Digital Signatures tab for this build's signing status.
Published 0.2.0 downloads are unsigned. Checksums check integrity, not publisher identity.
Public release downloads need no GitHub account or developer tools.

If Windows denies launch, ask IT to review the executable and office policy.
Run this read-only diagnostic in PowerShell from the extracted directory:
  .\Test-TokenFuel.ps1 -ExecutablePath .\TokenFuel.exe
It prints the path to a JSON report containing file/signature/runtime metadata
and matching policy event IDs where accessible. Review paths before sharing.
No provider credentials or usage data are read. No security settings are changed.
WinGet and portable downloads do not bypass Windows application-control policy.

Maintained by @tejas20. Optional support: https://github.com/sponsors/tejas20
