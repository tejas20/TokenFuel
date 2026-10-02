use crate::config::{Account, Connection};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tauri::{AppHandle, Manager};
use tokenfuel_core::{Provider, Snapshot, Status, parsers};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

pub struct Failure {
    pub status: Status,
    pub message: String,
    pub retry_after: Option<u64>,
}
impl Failure {
    pub fn new(status: Status, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            retry_after: None,
        }
    }
}

impl From<crate::http::Failure> for Failure {
    fn from(e: crate::http::Failure) -> Self {
        Self {
            status: e.status,
            message: e.message,
            retry_after: e.retry_after,
        }
    }
}

pub async fn fetch(app: &AppHandle, account: &Account) -> Result<Snapshot, Failure> {
    if !account.enabled {
        return Ok(Snapshot::empty(
            &account.id,
            account.provider,
            Status::Disconnected,
            "Connect this account to start tracking.",
        ));
    }
    match account.connection {
        Connection::Manual => Ok(Snapshot::ready(
            &account.id,
            account.provider,
            account.manual_limits.clone(),
        )),
        Connection::CodexCli if account.provider == Provider::Openai => codex(account).await,
        Connection::ClaudeCli if account.provider == Provider::Claude => claude(account).await,
        Connection::OpencodeGo if account.provider == Provider::Opencode => {
            opencode(account).await
        }
        Connection::Browser => browser(app, account).await,
        Connection::GeminiWeb if account.provider == Provider::Gemini => {
            gemini_web(app, account).await
        }
        _ => Err(Failure::new(
            Status::Unavailable,
            "This connection does not match the provider.",
        )),
    }
}

pub fn codex_path(account: &Account) -> Option<PathBuf> {
    if let Some(path) = &account.cli_path {
        let p = PathBuf::from(path);
        if p.is_file()
            && p.file_name()
                .is_some_and(|x| x.to_string_lossy().eq_ignore_ascii_case("codex.exe"))
        {
            return Some(p);
        }
        return None;
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join("codex.exe");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let dir = PathBuf::from(local).join("OpenAI/Codex/bin");
        let mut candidates: Vec<_> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path().join("codex.exe"))
            .filter(|p| p.is_file())
            .collect();
        candidates.sort_by_key(|p| std::fs::metadata(p).ok().and_then(|m| m.modified().ok()));
        return candidates.pop();
    }
    None
}

pub async fn codex(account: &Account) -> Result<Snapshot, Failure> {
    let exe = codex_path(account).ok_or_else(|| {
        Failure::new(
            Status::Unavailable,
            "Install the official Codex CLI or set its codex.exe path in connection settings.",
        )
    })?;
    let mut command = Command::new(exe);
    command
        .args([
            "app-server",
            "--stdio",
            "-c",
            "analytics.enabled=false",
            "-c",
            "feedback.enabled=false",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command.spawn().map_err(|_| {
        Failure::new(
            Status::Unavailable,
            "Could not start the installed Codex app-server.",
        )
    })?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| Failure::new(Status::Unavailable, "Codex input stream unavailable."))?;
    let mut output =
        BufReader::new(child.stdout.take().ok_or_else(|| {
            Failure::new(Status::Unavailable, "Codex output stream unavailable.")
        })?)
        .lines();
    let result=tokio::time::timeout(Duration::from_secs(25),async {
        input.write_all(b"{\"method\":\"initialize\",\"id\":1,\"params\":{\"clientInfo\":{\"name\":\"tokenfuel\",\"title\":\"TokenFuel\",\"version\":\"0.1.0\"}}}\n").await.map_err(|_|Failure::new(Status::Unavailable,"Codex initialization failed."))?;
        let mut initialized=false;
        let mut identity=None;
        loop {
            let line=output.next_line().await.map_err(|_|Failure::new(Status::Unavailable,"Codex disconnected."))?.ok_or_else(||Failure::new(Status::Unavailable,"Codex stopped before returning usage."))?;
            if line.len()>1_000_000{return Err(Failure::new(Status::Unavailable,"Unexpected Codex response size."));}
            let Ok(message)=serde_json::from_str::<Value>(&line) else {continue};
            if message.get("id").and_then(Value::as_i64)==Some(1) && !initialized {
                if message.get("error").is_some(){return Err(Failure::new(Status::Unavailable,"Installed Codex protocol is incompatible. Update Codex."));}
                initialized=true;input.write_all(b"{\"method\":\"initialized\",\"params\":{}}\n{\"method\":\"account/read\",\"id\":3,\"params\":{\"refreshToken\":false}}\n").await.map_err(|_|Failure::new(Status::Unavailable,"Codex account request failed."))?;
            }
            if message.get("id").and_then(Value::as_i64)==Some(3) {
                if let Some(raw)=message.pointer("/result/account")
                    && raw.get("type").and_then(Value::as_str)==Some("chatgpt")
                        && let Some(name)=raw.get("email").and_then(Value::as_str).filter(|s|s.len()<=254) {
                            identity=Some(tokenfuel_core::AccountIdentity{display_name:name.into(),plan:raw.get("planType").and_then(Value::as_str).filter(|s|s.len()<80).map(str::to_owned),workspace:None});
                        }
                input.write_all(b"{\"method\":\"account/rateLimits/read\",\"id\":2}\n").await.map_err(|_|Failure::new(Status::Unavailable,"Codex usage request failed."))?;
            }
            if message.get("id").and_then(Value::as_i64)==Some(2) {
                if let Some(error)=message.get("error") {
                    let e=error.get("message").and_then(Value::as_str).unwrap_or("").to_lowercase();
                    let status=if e.contains("auth")||e.contains("sign")||e.contains("login"){Status::LoginRequired}else{Status::Unavailable};
                    return Err(Failure::new(status,"Codex could not read subscription limits. Sign in to Codex using your ChatGPT account and retry."));
                }
                let raw=message.get("result").ok_or_else(||Failure::new(Status::Unavailable,"Codex returned an unsupported response."))?;
                let mut snapshot=Snapshot::ready(&account.id,Provider::Openai,parsers::codex(raw));
                snapshot.identity=identity;
                if !snapshot.limits.is_empty(){snapshot.message="Codex / shared Work allowance. Ordinary ChatGPT chat model counters are not exposed by this source.".into();}return Ok(snapshot);
            }
        }
    }).await;
    let _ = child.kill().await;
    result.map_err(|_| {
        Failure::new(
            Status::Offline,
            "Codex usage read timed out; cached readings are preserved.",
        )
    })?
}

pub async fn claude(account: &Account) -> Result<Snapshot, Failure> {
    if !account.experimental {
        return Err(Failure::new(
            Status::Disconnected,
            "Enable experimental access before reading a Claude session.",
        ));
    }
    let path = account
        .credential_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|h| PathBuf::from(h).join(".claude/.credentials.json"))
        })
        .ok_or_else(|| Failure::new(Status::LoginRequired, "Claude credentials were not found."))?;
    let raw = std::fs::read(&path).map_err(|_| {
        Failure::new(
            Status::LoginRequired,
            "Sign in to the Claude Code CLI, or connect the isolated Claude usage window.",
        )
    })?;
    let value: Value = serde_json::from_slice(&raw).map_err(|_| {
        Failure::new(
            Status::LoginRequired,
            "Claude credential format was not recognized.",
        )
    })?;
    let token=value.pointer("/claudeAiOauth/accessToken").and_then(Value::as_str).ok_or_else(||Failure::new(Status::LoginRequired,"Claude subscription OAuth credentials were not found. API keys do not reveal subscription usage."))?;
    let entry = keyring::Entry::new("TokenFuel", &account.id).map_err(|_| {
        Failure::new(
            Status::Unavailable,
            "Windows Credential Manager unavailable.",
        )
    })?;
    entry.set_password(token).map_err(|_| {
        Failure::new(
            Status::Unavailable,
            "Could not protect the Claude session in Credential Manager.",
        )
    })?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| Failure::new(Status::Offline, "Usage client unavailable."))?;
    let mut response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .map_err(|_| {
            Failure::new(
                Status::Offline,
                "Claude usage service could not be reached.",
            )
        })?;
    let status = response.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(Failure::new(
            Status::LoginRequired,
            "Claude rejected this session. Sign in again in Claude; TokenFuel does not rewrite its credentials.",
        ));
    }
    if status.as_u16() == 429 {
        let retry = response
            .headers()
            .get("retry-after")
            .and_then(|x| x.to_str().ok())
            .and_then(|s| {
                s.parse::<u64>().ok().or_else(|| {
                    DateTime::parse_from_rfc2822(s)
                        .ok()
                        .map(|dt| (dt.with_timezone(&Utc) - Utc::now()).num_seconds().max(0) as u64)
                })
            });
        return Err(Failure {
            status: Status::RateLimited,
            message: "Claude asked TokenFuel to wait before polling again.".into(),
            retry_after: retry,
        });
    }
    if !status.is_success() {
        return Err(Failure::new(
            Status::Unavailable,
            "The Claude usage endpoint is unavailable for this account.",
        ));
    }
    if response.content_length().is_some_and(|n| n > 1_000_000) {
        return Err(Failure::new(
            Status::Unavailable,
            "Unexpected Claude usage response size.",
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Failure::new(Status::Offline, "Claude response was interrupted."))?
    {
        if bytes.len() + chunk.len() > 1_000_000 {
            return Err(Failure::new(
                Status::Unavailable,
                "Unexpected Claude usage response size.",
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| {
        Failure::new(
            Status::Unavailable,
            "Claude changed its usage response format.",
        )
    })?;
    let mut result = Snapshot::ready(&account.id, Provider::Claude, parsers::claude(&value));
    result.message="Experimental Claude session source. Enterprise monthly support requires verification against your organisation's Usage page.".into();
    Ok(result)
}

pub async fn opencode(account: &Account) -> Result<Snapshot, Failure> {
    let mut workspace_id = account.workspace.trim().to_string();
    let mut auth_cookie = String::new();

    if account.has_custom_secret {
        if let Some(secret) = crate::credentials::read_account_secret(&account.id) {
            let secret = secret.trim();
            if let Ok(v) = serde_json::from_str::<Value>(secret) {
                if let Some(c) = v
                    .get("authCookie")
                    .or_else(|| v.get("auth_cookie"))
                    .or_else(|| v.get("cookie"))
                    .and_then(Value::as_str)
                {
                    auth_cookie = c.trim().to_string();
                }
                if workspace_id.is_empty() {
                    if let Some(w) = v
                        .get("workspaceId")
                        .or_else(|| v.get("workspace_id"))
                        .and_then(Value::as_str)
                    {
                        workspace_id = w.trim().to_string();
                    }
                }
            } else if !secret.is_empty() {
                auth_cookie = secret.to_string();
            }
        }
    }

    if auth_cookie.is_empty() {
        if let Ok(c) = std::env::var("OPENCODE_GO_AUTH_COOKIE") {
            auth_cookie = c.trim().to_string();
        }
    }
    if workspace_id.is_empty() {
        if let Ok(w) = std::env::var("OPENCODE_GO_WORKSPACE_ID") {
            workspace_id = w.trim().to_string();
        }
    }

    if auth_cookie.is_empty() || workspace_id.is_empty() {
        let mut candidates = Vec::new();
        if let Some(ref p) = account.credential_path {
            candidates.push(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("OPENCODE_GO_CONFIG_FILE") {
            candidates.push(PathBuf::from(p));
        }
        if let Ok(app_data) = std::env::var("APPDATA") {
            candidates.push(PathBuf::from(app_data).join("opencode-go").join("config.json"));
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            let home_p = PathBuf::from(home);
            candidates.push(home_p.join(".config").join("opencode-bar").join("opencode-go.json"));
            candidates.push(home_p.join(".config").join("opencode-quota").join("opencode-go.json"));
        }
        for path in candidates {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Value>(&content) {
                    if auth_cookie.is_empty() {
                        if let Some(c) = cfg
                            .get("authCookie")
                            .or_else(|| cfg.get("auth_cookie"))
                            .or_else(|| cfg.get("cookie"))
                            .and_then(Value::as_str)
                        {
                            auth_cookie = c.trim().to_string();
                        }
                    }
                    if workspace_id.is_empty() {
                        if let Some(w) = cfg
                            .get("workspaceId")
                            .or_else(|| cfg.get("workspace_id"))
                            .and_then(Value::as_str)
                        {
                            workspace_id = w.trim().to_string();
                        }
                    }
                    if !auth_cookie.is_empty() && !workspace_id.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    if auth_cookie.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "OpenCode Go auth cookie was not found. Set it in Account settings or OPENCODE_GO_AUTH_COOKIE.",
        ));
    }
    if workspace_id.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "OpenCode Go workspace ID was not found. Set it in Account settings or OPENCODE_GO_WORKSPACE_ID.",
        ));
    }

    let cookie = if auth_cookie.split(';').any(|part| {
        let p = part.trim_start();
        p.starts_with("auth=") || p.starts_with("__Host-console_session=")
    }) {
        auth_cookie
    } else {
        format!("auth={auth_cookie}")
    };

    let client = crate::http::create_client()?;
    let response = client
        .get("https://opencode.ai/console/api/go/status")
        .header("Accept", "application/json")
        .header("x-org-id", &workspace_id)
        .header("Cookie", &cookie)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| Failure::new(Status::Offline, format!("OpenCode Go request failed: {e}")))?;

    let status_code = response.status();
    let headers = response.headers().clone();
    crate::http::handle_http_status(status_code, &headers, "OpenCode Go")?;

    let bytes = crate::http::read_bounded_bytes(response, 1_048_576).await?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| Failure::new(Status::Unavailable, "OpenCode Go returned invalid JSON."))?;

    let limits = parsers::opencode(&value);
    if limits.is_empty() {
        return Err(Failure::new(
            Status::Unavailable,
            "OpenCode Go status returned no active subscription meters.",
        ));
    }

    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
    snapshot.message = format!("OpenCode Go · Org {workspace_id}");
    Ok(snapshot)
}

async fn browser(app: &AppHandle, account: &Account) -> Result<Snapshot, Failure> {
    if !account.experimental {
        return Err(Failure::new(
            Status::Disconnected,
            "Browser capture requires experimental opt-in.",
        ));
    }
    let label = format!("provider-{}", account.id);
    let window=app.get_webview_window(&label).ok_or_else(||Failure::new(Status::LoginRequired,"Open the isolated provider window, sign in, then open Settings → Usage limits. Keep that view open for capture."))?;
    let url = window
        .url()
        .map_err(|_| Failure::new(Status::Unavailable, "Provider window unavailable."))?;
    let expected = match account.provider {
        Provider::Claude => "claude.ai",
        Provider::Gemini => "gemini.google.com",
        Provider::Openai => "chatgpt.com",
        _ => {
            return Err(Failure::new(
                Status::Unavailable,
                "Grok is planned for a later release.",
            ));
        }
    };
    if url.host_str() != Some(expected) {
        return Err(Failure::new(
            Status::LoginRequired,
            "Complete provider sign-in, then open the provider's Usage view.",
        ));
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Mutex::new(Some(tx));
    window
        .eval_with_callback(include_str!("usage-view.js"), move |s| {
            if let Some(tx) = tx.lock().ok().and_then(|mut t| t.take()) {
                let _ = tx.send(s);
            }
        })
        .map_err(|_| Failure::new(Status::Unavailable, "Could not read the usage view."))?;
    let raw = tokio::time::timeout(Duration::from_secs(8), rx)
        .await
        .map_err(|_| Failure::new(Status::Offline, "Provider view did not respond."))?
        .map_err(|_| Failure::new(Status::Unavailable, "Provider window was closed."))?;
    if raw.len() > 32_000 {
        return Err(Failure::new(
            Status::Unavailable,
            "Usage view response was rejected.",
        ));
    }
    let value: Value = serde_json::from_str(&raw)
        .map_err(|_| Failure::new(Status::Unavailable, "Could not decode the usage view."))?;
    let mut limits = vec![];
    if let Some(entries) = value.as_array() {
        for v in entries.iter().take(12) {
            if account.connection == Connection::GeminiWeb
                && v.get("ageSeconds")
                    .and_then(Value::as_i64)
                    .filter(|age| (0..=604800).contains(age))
                    .is_none()
            {
                return Err(Failure::new(
                    Status::Unavailable,
                    "Gemini did not expose a recognised freshness label. Use explicit capture until this layout is verified.",
                ));
            }
            let Some(p) = v.get("usedPercent").and_then(Value::as_f64) else {
                continue;
            };
            let name = v
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Reported window");
            if ![
                "Weekly",
                "5 hours",
                "Monthly budget",
                "Reported window",
                "Daily",
            ]
            .contains(&name)
            {
                continue;
            }
            let product = match account.provider {
                Provider::Gemini => "Gemini",
                Provider::Claude => "Claude",
                Provider::Openai => "ChatGPT",
                _ => "Grok",
            };
            if let Some(mut q) = tokenfuel_core::UsageLimit::percentage(
                &format!("web:{name}"),
                name,
                product,
                "reported",
                p,
                tokenfuel_core::Source::Experimental,
            ) {
                q.resets_at = v
                    .get("resetsAt")
                    .and_then(Value::as_str)
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                q.reset_label = v
                    .get("resetLabel")
                    .and_then(Value::as_str)
                    .filter(|s| s.len() <= 80 && s.starts_with("Resets "))
                    .map(str::to_owned);
                if let Some(age) = v
                    .get("ageSeconds")
                    .and_then(Value::as_i64)
                    .filter(|age| (0..=604800).contains(age))
                {
                    q.observed_at = Utc::now() - chrono::Duration::seconds(age);
                }
                limits.push(q);
            }
        }
    }
    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
    snapshot.message="Experimental visible Usage view capture, not automatic tracking. Refresh the provider view before capture. Only explicit machine-readable reset timestamps are retained; other resets and totals remain unknown.".into();
    Ok(snapshot)
}

/// Reload only the dedicated, quota-only Gemini route. Existing Browser connections stay explicit captures.
async fn gemini_web(app: &AppHandle, account: &Account) -> Result<Snapshot, Failure> {
    if !account.experimental {
        return Err(Failure::new(
            Status::Disconnected,
            "Gemini live view requires experimental opt-in.",
        ));
    }
    let label = format!("provider-{}", account.id);
    let window = app.get_webview_window(&label).ok_or_else(|| {
        Failure::new(
            Status::LoginRequired,
            "Open the isolated Gemini window and sign in to enable live Usage polling.",
        )
    })?;
    let url = window
        .url()
        .map_err(|_| Failure::new(Status::Unavailable, "Gemini window unavailable."))?;
    if url.host_str() != Some("gemini.google.com") {
        return Err(Failure::new(
            Status::LoginRequired,
            "Complete Gemini sign-in before live polling.",
        ));
    }
    let state = app.state::<crate::State>();
    let before = *state
        .provider_loads
        .lock()
        .unwrap()
        .get(&label)
        .unwrap_or(&0);
    window
        .navigate("https://gemini.google.com/usage".parse().unwrap())
        .map_err(|_| Failure::new(Status::Offline, "Gemini Usage reload failed."))?;
    tokio::time::timeout(Duration::from_secs(20),async {
        loop {
            let loaded=*state.provider_loads.lock().unwrap().get(&label).unwrap_or(&0)>before;
            if loaded {
                let current=window.url().map_err(|_|Failure::new(Status::Offline,"Gemini window closed."))?;
                if current.host_str()!=Some("gemini.google.com") { return Err(Failure::new(Status::LoginRequired,"Gemini sign-in expired.")); }
                if current.path()=="/usage" {
                    let mut snapshot=browser(app,account).await?;
                    if snapshot.status==Status::Available {
                        let interval=state.config.lock().unwrap().settings.interval_secs;
                        snapshot.mark_stale(Utc::now(),interval,false);
                        snapshot.message="Experimental Gemini Usage page polling. Keep its isolated signed-in window open. Values are provider-reported; text-only reset labels are preserved without inferred timestamps.".into();
                        return Ok(snapshot);
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }).await.map_err(|_|Failure::new(Status::Offline,"Gemini Usage did not finish loading supported counters; cached readings are retained."))?
}

pub fn provider_url(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "https://claude.ai/settings/usage",
        Provider::Openai => "https://chatgpt.com/",
        Provider::Gemini => "https://gemini.google.com/usage",
        Provider::Grok => "https://grok.com/",
        Provider::Opencode => "https://opencode.ai/console",
        Provider::Cursor => "https://www.cursor.com/settings",
        Provider::Copilot => "https://github.com/settings/copilot",
        Provider::Antigravity => "https://cloud.google.com/",
        Provider::Unknown => "about:blank",
    }
}

pub fn safe_navigation(url: &tauri::Url, provider: Provider) -> bool {
    if url.scheme() != "https" {
        return url.as_str() == "about:blank";
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    let allowed: &[&str] = match provider {
        Provider::Claude => &[
            "claude.ai",
            "accounts.google.com",
            "login.microsoftonline.com",
            "login.live.com",
            "appleid.apple.com",
        ],
        Provider::Gemini => &[
            "gemini.google.com",
            "accounts.google.com",
            "myaccount.google.com",
        ],
        Provider::Openai => &[
            "chatgpt.com",
            "auth.openai.com",
            "auth0.openai.com",
            "accounts.google.com",
            "login.microsoftonline.com",
            "login.live.com",
            "appleid.apple.com",
        ],
        Provider::Grok => &["grok.com"],
        Provider::Opencode => &[
            "opencode.ai",
            "auth.opencode.ai",
            "accounts.google.com",
            "github.com",
        ],
        Provider::Cursor => &[
            "cursor.com",
            "www.cursor.com",
            "authenticator.cursor.sh",
            "github.com",
            "accounts.google.com",
        ],
        Provider::Copilot => &["github.com", "api.github.com"],
        Provider::Antigravity => &[
            "cloud.google.com",
            "accounts.google.com",
            "myaccount.google.com",
        ],
        Provider::Unknown => &[],
    };
    allowed.contains(&host)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_windows_reject_other_origins_and_deceptive_hosts() {
        assert!(safe_navigation(
            &tauri::Url::parse("https://accounts.google.com/login").unwrap(),
            Provider::Gemini
        ));
        for url in [
            "https://gemini.google.com.evil.example/",
            "http://gemini.google.com/",
            "file:///C:/secret",
        ] {
            assert!(!safe_navigation(
                &tauri::Url::parse(url).unwrap(),
                Provider::Gemini
            ));
        }
    }
}
