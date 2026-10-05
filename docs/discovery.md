# Automatic connections

TokenFuel starts with the remaining-usage widget. At each launch it checks supported local sources and enables newly detected sources for quota reads. Experimental adapters are enabled too and retain their experimental labels. Detection is local; the regular polling loop then asks each connected provider for usage. Nothing is installed by this scan.

| Source | Locations checked |
|---|---|
| Codex | Native or npm-packaged `codex.exe` on PATH, `%APPDATA%\npm`, `%USERPROFILE%\.local\bin`, and direct or versioned `%LOCALAPPDATA%\OpenAI\Codex\bin` installations |
| Claude Code | `CLAUDE_CONFIG_DIR\.credentials.json`, otherwise `%USERPROFILE%\.claude\.credentials.json` |
| Cursor | `%APPDATA%\Cursor\User\globalStorage\state.vscdb` or `CURSOR_SESSION_TOKEN` |
| Copilot | Copilot/GitHub CLI Windows credentials, `%USERPROFILE%\.config\github-copilot` and `%LOCALAPPDATA%\github-copilot` auth files, supported token environment variables |
| Grok | `GROK_HOME\auth.json`, `%USERPROFILE%\.grok\auth.json`, or supported Grok/XAI token environment variables |
| OpenCode Go | `OPENCODE_GO_CONFIG_FILE`, `%APPDATA%\opencode-go\config.json`, user `.config\opencode-bar\opencode-go.json` or `.config\opencode-quota\opencode-go.json`, or Go cookie/workspace environment variables |
| Antigravity | Supported Windows credential entries, environment variables, user `.antigravity\auth.json` or `.config\antigravity\auth.json`, and `%LOCALAPPDATA%\antigravity\auth.json` |
| Gemini | No compatible local-session quota adapter. Enable Gemini live Usage view in Settings, open its isolated window, and sign in. Keep it open for polling. |

A detected source may still need sign-in or a qualifying subscription. The Settings button indicates reported failures; Settings displays the provider's next-step message. Cached usage remains visibly aged. Ordinary ChatGPT and Grok chat counters are not supplied by the Codex and Grok Build adapters.

Disable a connection and Save to stop reads. Previously edited, disabled, or removed connections are not automatically re-enabled. A source absent during an earlier scan can be discovered on a later launch. Advanced settings remain available for custom accounts and paths.

Preferences and discovery history are stored in `%APPDATA%\com.tejas20.tokenfuel\config.json`. Installer and portable copies for the same Windows user use this same location. Always on top defaults to on for new configurations; existing choices are retained. Startup remains opt-in.
