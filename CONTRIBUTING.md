# Contributing to TokenFuel

TokenFuel is maintained by [@tejas20](https://github.com/tejas20) and licensed under MIT. Contributions are welcome through issues and pull requests.

For bugs, include the TokenFuel version, Windows version, provider/source and steps to reproduce. Remove account identifiers, tokens, cookies, workspace names and personal quota data from screenshots or logs. Report security issues privately using [SECURITY.md](SECURITY.md).

Before a larger feature or new provider integration, open an issue to discuss the source, permissions and validation. Prefer documented provider APIs. Keep experimental integrations clearly labelled, preserve account isolation and freshness information, and never invent unsupported counters or reset dates.

## Build and check

Follow [development setup](docs/setup.md) on Windows 10/11 x64 with WebView2, Node.js 24+, pnpm 11.19.0, Rust 1.99.0 MSVC and Microsoft C++ Build Tools/Windows SDK.

```powershell
pnpm install --frozen-lockfile
pnpm test
pnpm build
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

Use `pnpm tauri dev` for the native app or `pnpm dev` and `http://127.0.0.1:1420/?demo=1` for a browser preview with sample data. Add focused regression coverage when changing calculations, credentials, parsers or account behavior. Use synthetic fixtures; do not commit real credentials, usage captures or local configuration.

Explain the problem, resulting behavior and validation in your pull request. Follow the [release checklist](docs/releasing.md) for packaging changes. Contributions are distributed under the repository's MIT license.
