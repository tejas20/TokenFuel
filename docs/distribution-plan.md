# Installation and sign-in decisions

## Shipped in 0.1.0

The default download is a small per-user NSIS installer with an embedded Microsoft WebView2 bootstrapper. Users run an EXE and launch from Start; developer tools are unnecessary. The separate offline installer includes the Evergreen standalone runtime for blocked-download environments. Portable ZIP is secondary. All packages use the same widget code and include checksums. Tauri adds a three-byte bundle-type marker to installed binaries; the standalone binary retains its portable marker.

This release is published inside the existing private repository. Invited readers can download it in a browser. Private GitHub access still adds friction; it cannot be eliminated for uninvited users without a distribution/visibility decision.

First launch opens setup. Local-source connection hints explain existing sign-in reuse. Provider selection, access permission and Save are the normal path. Token and workspace entry stays under Advanced settings. Automatic discovery happens only after connection consent. The widget does not extract browser cookies or modify another app's credentials. Custom secrets live in Windows Credential Manager and fail closed instead of falling back to another account. Original credentials remain with their provider app. Startup and alerts stay opt-in.

Experimental-source consent remains separate because those integrations can change without notice. This adds one step but avoids presenting an unsupported counter as a verified allowance. Unavailable counters, cached ages and sign-in failures remain visible.

## Next steps, in order

1. **Publisher-signed installer and executable.** Obtain a Windows signing identity or approved signing service, then sign and timestamp both artifacts in the build pipeline. Hashes do not establish publisher identity. Signing is the largest remaining installation-trust improvement; it does not guarantee immediate SmartScreen reputation.
2. **Validate new integrations on real accounts.** Check totals, multiple pools, identity, expiry, disconnect, retry limits and changed provider markup against the provider's UI. Keep provisional labels until that evidence exists.
3. **Decide distribution audience.** Keep invited private downloads for initial testing. For general non-coder distribution, approve public release assets or a separate download channel before creating it. Never embed a maintainer GitHub token in the widget to bypass private downloads.
4. **Native ARM64 package and clean-machine matrix.** Add a Windows ARM64 build with the matching toolchain; test its Credential Manager, Windows SQLite and WebView2 integration on actual hardware. Validate clean Windows 10/11 x64 installations with missing WebView2, offline setup and managed-device restrictions. Current packages are x64 only.
5. **Signed updates.** Add an opt-in updater with a dedicated update-signing key and a suitable download endpoint after signing/distribution are decided. A private GitHub updater cannot safely depend on a shared embedded access token. Until then, rerunning the newer installer retains configuration.
6. **Provider-owned browser login where supported.** Prefer official OAuth/device authorization flows with minimal scopes and protected refresh tokens. Browser-cookie copying should never become the primary setup path. Some providers do not expose subscription quotas at all; manual snapshots remain the honest fallback.

## Compatibility boundaries

WebView2 Evergreen is shared and serviced by Microsoft. Fixed runtime bundles add size and transfer patch responsibility to the app, so the offline installer installs Evergreen instead. Windows 7/8 are excluded from the support promise. Corporate policy and SSO restrictions require administrator/provider support rather than bypasses.

Sources: [Tauri Windows installer options](https://v2.tauri.app/distribute/windows-installer/), [Microsoft WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution), [Tauri single-instance plugin](https://v2.tauri.app/plugin/single-instance/).
