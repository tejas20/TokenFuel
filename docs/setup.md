# Windows development setup

This page is for building TokenFuel from source. To use the prebuilt app, follow the [installation guide](installation.md). The installer needs no Git, GitHub CLI, Rust, Node.js, pnpm or Python.

This development machine has Git and bundled Node.js/pnpm. Installed for TokenFuel: GitHub CLI, Rust stable MSVC 1.99.0, Visual Studio 2022 Build Tools with the Native Desktop C++ workload and recommended Windows SDK. WebView2 is present. Python is bundled but optional.

For a standalone development machine, install Git, Node.js 24 LTS, pnpm 11.19.0, Rust through the official rustup installer, and Microsoft C++ Build Tools. The checked-in toolchain file selects Rust 1.99.0. Choose **Desktop development with C++**, including the Windows SDK. Restart the terminal so Cargo is on PATH. GitHub CLI is optional for repository/release management; use `gh auth login` if you install it. Public prebuilt releases need no GitHub sign-in.

[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) are the authority for desktop build requirements. Use the Windows certificate store for Node installations behind a trusted enterprise TLS proxy: `$env:NODE_USE_SYSTEM_CA='1'`. Do not disable certificate validation.

`pnpm install --frozen-lockfile`, then `pnpm tauri dev`. Use `pnpm tauri build` for NSIS and the portable executable. The first build downloads Rust crates and NSIS tooling. No Python runtime ships with the app.

GitHub owner and maintainer is [@tejas20](https://github.com/tejas20); the public repository is [tejas20/TokenFuel](https://github.com/tejas20/TokenFuel). See [contributing](../CONTRIBUTING.md) and [security reporting](../SECURITY.md).
