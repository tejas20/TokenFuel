# TokenFuel

A Windows desktop widget for remaining AI subscription allowances. Private development preview; MIT licensed. The selected soft-glass dock has horizontal bars and circular gauges, separate account/workspace pools, quota pinning, tray controls, remembered position, optional startup and threshold alerts.

## Provider support

| Provider | Connection | Current validation |
|---|---|---|
| OpenAI | Installed Codex app-server, documented `account/rateLimits/read` | Adapter implemented; live verification pending consent |
| Claude | Experimental Claude Code OAuth usage endpoint | Synthetic payload tests include personal windows and member monthly monetary limits; live office verification outstanding |
| Claude / Gemini / ChatGPT | Isolated, session-only sign-in window; explicit visible Usage capture | Experimental, conservative DOM parser; signed-in provider validation outstanding |
| All supported providers | Manual percentage / decimal budget / unlimited snapshots | Clearly manual; never verified automatic tracking |
| Grok | Planned adapter | Not enabled |

Codex counters are labelled Codex. They do not represent every ordinary ChatGPT model. Browser capture does not reload a provider page or infer reset dates, window limits, or currency. Refresh the provider's Usage view before capturing. Google or enterprise SSO may reject embedded sign-in; use manual snapshots if that happens. Enterprise monthly support remains provisional until compared on the office laptop.

## Run and build

Windows 10/11 x64, WebView2, Git, Node.js 24+, pnpm 11.19.0, Rust 1.99.0 MSVC, Visual Studio 2022 C++ build tools and Windows SDK.

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
# Browser-only design preview, with labelled sample data:
pnpm dev
# http://127.0.0.1:1420/?demo=1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm test
pnpm tauri build
```

Installer: `target/release/bundle/nsis/`. Portable executable: `target/release/tokenfuel.exe` (requires WebView2). Release artifacts are private. Python is optional development tooling, never an app dependency.

## Local data and security

Networking, parsing, credentials, scheduling and calculations live in Rust. React receives decimal strings and normalized snapshots. Local config and quota cache are stored under the Windows application configuration directory for `com.tejas20.tokenfuel`. No conversation content, telemetry or token logging is implemented.

All accounts start disconnected. The connection consent control authorizes existing CLI session access. Experimental connections have a separate opt-in. Claude tokens copied from an authorized local CLI session are protected by Windows Credential Manager (`TokenFuel`, account UUID). Disconnecting stops reads; removing the account deletes TokenFuel's credential. Original provider credentials are never rewritten. Browser sessions use an incognito WebView and are not persisted across restarts. Provider windows have no local capabilities, and all native application commands additionally validate the calling widget's identity and origin.

Polling defaults to two minutes, with separate account backoff and provider Retry-After floors. Percentages require a supplied percentage or valid denominator. Manual and browser snapshots retain their capture time; cached data and connection errors remain explicit. Monthly reset dates are provider supplied, never assumed to be the first. Notifications fire only on observed downward crossings of 20% and 10%.

See [setup](docs/setup.md), [office validation](docs/office-validation.md), [research](docs/provider-research.md), and [third-party notices](THIRD_PARTY_NOTICES.md).
