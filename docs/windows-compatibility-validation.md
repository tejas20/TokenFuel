# Windows distribution compatibility work - 2026-10-06

The office report shows SmartScreen rejecting an unrecognized installer and a separate `Start-Process` access-denied error when launching the portable executable. The precise office policy/antivirus/permissions cause remains unconfirmed. The reference app worked there through WinGet.

Implemented locally:

- Failed launches preserve the native Windows error code, generate diagnostics, and return exit code 1.
- `Start-TokenFuel.cmd -Diagnose` inspects the existing cache without downloading or launching. `Test-TokenFuel.ps1 -ExecutablePath <path>` supports inspecting a separate installer or portable executable.
- Signed packaging accepts a Tauri signing config for both bundlers and checks valid timestamped signatures with a consistent signer across installers and the portable app. CI opts into unsigned test packaging explicitly.
- WinGet manifests are generated from final installer bytes for later submission.

Validation on this development PC:

- Windows PowerShell 5.1 regression script passed: verified cache, download-only, diagnostics with/without cache, simulated Windows error 5, tamper rejection, missing signing configuration, signature/timestamp/signer rejection, signing config forwarding, and final installer hashes in generated manifests. Signature-positive cases use mocks; they do not prove real signing.
- Actual regular/offline NSIS rebundling and portable ZIP generation passed with `-SkipBuild -AllowUnsigned` using the existing 0.2.0 application binary.
- `winget validate` accepted the generated three-file manifest directory.
- Real executable diagnostics reported `NotSigned` and detected this PC's WebView2 runtime. No office device or policy was accessed.

Not completed: acquiring/configuring a trusted signing identity, signing a real release, publishing new release bytes, WinGet submission/acceptance, testing clean-machine installs/upgrades/uninstalls through WinGet, and retesting office launch. Existing published release assets are unchanged. No claim is made that these tooling changes alone resolve the office block.
