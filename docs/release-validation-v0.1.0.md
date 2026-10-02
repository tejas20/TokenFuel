# Validation for TokenFuel 0.1.0

Date: 2 October 2026. Feature branch `feature/additional-providers` merged into `main` with a merge commit. Follow-up fixes simplify setup and strengthen account isolation.

## Automated checks

- `pnpm test`: **28 tests passed** across formatting, freshness, widget visibility, quota rendering, Usage DOM parsing and account setup.
- `cargo test --workspace --locked`: **32 tests passed** (11 native app tests, 21 core tests).
- `pnpm build`: TypeScript and production Vite build passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed with the checkout's normal Windows line-ending conversion.
- Windows PowerShell parsed both the launcher and packaging script without errors.

Native integration tests create only random test credentials and synthetic SQLite data. They verify Credential Manager save/read/delete/idempotent deletion, SQLite parameter binding, read-only access and in-memory backup. They do not read or modify real provider credentials. Other regression tests verify missing custom secrets fail before fallback or network use, source changes cannot reuse a different provider's secret, remote WebView origins are rejected, and unsupported token fields do not appear in setup.

## Packaging and native delivery

Regular and offline NSIS installers were both installed successfully for the current user (exit code 0). They created the Start menu shortcut and preserved account IDs, sources and permissions. Installed binaries from both variants matched byte for byte. The portable executable matched the installed code except for Tauri's expected three-byte bundle-type marker (`NSS` for the installer, `UNK` for standalone).

The installed app displayed `Settings · v0.1.0` and real Codex five-hour/weekly counters. Settings and compact bars were visually inspected with the Computer Use skill. Existing local Codex consent was retained; no additional provider was enabled. Reopening the app retained the original process ID and exactly one widget process. The quota cache advanced after launch, confirming a live read. All three download hashes matched `SHA256SUMS.txt`. Installer sizes were approximately 5.2 MiB regular and 208 MiB offline; portable approximately 4.2 MiB.

An installed WebView2 runtime was present throughout these checks. Packaging the offline runtime and installing its wrapper does not prove the missing-runtime branch on a clean or policy-restricted machine. Released binaries remain unsigned.

## Limits

Synthetic provider fixtures validate normalization, pool separation and conservative missing-data behavior. They do not certify provider endpoint availability across plans or accounts. Live testing of new providers, Claude Enterprise monthly limits, native Gemini sign-in and sustained polling, SSO, multiple physical monitors/DPI, Windows startup and notification crossings remains outstanding. ARM64/32-bit and a clean machine without WebView2 are not tested on this x64 development PC.
