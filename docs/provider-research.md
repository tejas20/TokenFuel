# Provider research and integration boundaries

Reviewed 2026-10-02. Provider contracts and product limits can change.

- [Official Codex app-server](https://learn.chatgpt.com/docs/app-server): initialize stdio JSON-RPC, acknowledge initialized, call `account/rateLimits/read`. Prefer `rateLimitsByLimitId`, fall back to the legacy bucket without duplicating pools. No model request or conversation access.
- [Claude member usage visibility](https://support.claude.com/en/articles/12883420-view-usage-analytics-for-team-and-enterprise-plans): validate member-visible allocation rather than requiring an administrator API. Account settings and enterprise billing types differ.
- [Gemini limits](https://support.google.com/gemini/answer/16275805?hl=en-GB): personal compute windows may include five-hour and weekly limits. No documented public consumer quota API was identified. Conservative visible view capture is experimental.
- [Reference Rust parser](https://github.com/CodeZeno/Claude-Code-Usage-Monitor/blob/main/src/poller/claude/limits.rs): current source already researches monthly spend and scoped limits. TokenFuel's parser is independently written and uses synthetic tests; it does not assume all enterprise accounts expose the same schema.

Claude OAuth usage (`api.anthropic.com/api/oauth/usage`) is undocumented experimental access, not a stable public contract. Never probe by sending paid messages. Unsupported schemas produce unavailable data. Monthly monetary values use decimal arithmetic and preserve currency/exponent. Unknown resets stay unknown.

Browser extraction reads only a usage-labelled dialog or the explicit usage settings page and returns a bounded list of scalar percentages plus fixed quota labels. Ambiguous used/remaining orientation is rejected. No cookies, account names, page dumps or chat content return to the app. Captures are user initiated; they are not verified automatic integrations.
