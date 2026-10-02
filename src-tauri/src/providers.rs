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

fn account_secret(account: &Account) -> Result<Option<String>, Failure> {
    if !account.has_custom_secret {
        return Ok(None);
    }
    crate::credentials::read_account_secret(&account.id)
        .filter(|secret| !secret.trim().is_empty())
        .map(Some)
        .ok_or_else(|| Failure::new(Status::LoginRequired,
            "The saved account secret is missing. Update it or clear it to use the local sign-in."))
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
    if account.connection.requires_experimental_opt_in() && !account.experimental {
        return Err(Failure::new(
            Status::Disconnected,
            "Enable experimental opt-in before connecting this usage source.",
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
        Connection::OpencodeGo if account.provider == Provider::Opencode => opencode(account).await,
        Connection::CursorLocal if account.provider == Provider::Cursor => cursor(account).await,
        Connection::GrokCli if account.provider == Provider::Grok => grok(account).await,
        Connection::CopilotCli if account.provider == Provider::Copilot => copilot(account).await,
        Connection::AntigravityLocal if account.provider == Provider::Antigravity => {
            antigravity(account).await
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

    if let Some(secret) = account_secret(account)? {
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
            if workspace_id.is_empty()
                && let Some(w) = v
                    .get("workspaceId")
                    .or_else(|| v.get("workspace_id"))
                    .and_then(Value::as_str)
            {
                workspace_id = w.trim().to_string();
            }
        } else if !secret.is_empty() {
            auth_cookie = secret.to_string();
        }
    }

    if account.has_custom_secret && (auth_cookie.is_empty() || workspace_id.is_empty()) {
        return Err(Failure::new(
            Status::LoginRequired,
            "The saved OpenCode secret needs an auth cookie and workspace ID. Update Advanced connection settings.",
        ));
    }
    if auth_cookie.is_empty()
        && let Ok(c) = std::env::var("OPENCODE_GO_AUTH_COOKIE")
    {
        auth_cookie = c.trim().to_string();
    }
    if workspace_id.is_empty()
        && let Ok(w) = std::env::var("OPENCODE_GO_WORKSPACE_ID")
    {
        workspace_id = w.trim().to_string();
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
            candidates.push(
                PathBuf::from(app_data)
                    .join("opencode-go")
                    .join("config.json"),
            );
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            let home_p = PathBuf::from(home);
            candidates.push(
                home_p
                    .join(".config")
                    .join("opencode-bar")
                    .join("opencode-go.json"),
            );
            candidates.push(
                home_p
                    .join(".config")
                    .join("opencode-quota")
                    .join("opencode-go.json"),
            );
        }
        for path in candidates {
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(cfg) = serde_json::from_str::<Value>(&content)
            {
                if auth_cookie.is_empty()
                    && let Some(c) = cfg
                        .get("authCookie")
                        .or_else(|| cfg.get("auth_cookie"))
                        .or_else(|| cfg.get("cookie"))
                        .and_then(Value::as_str)
                {
                    auth_cookie = c.trim().to_string();
                }
                if workspace_id.is_empty()
                    && let Some(w) = cfg
                        .get("workspaceId")
                        .or_else(|| cfg.get("workspace_id"))
                        .and_then(Value::as_str)
                {
                    workspace_id = w.trim().to_string();
                }
                if !auth_cookie.is_empty() && !workspace_id.is_empty() {
                    break;
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

fn extract_cursor_user_id(jwt: &str) -> Option<String> {
    let payload = jwt.split('.').nth(1)?;
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(payload))
        .ok()?;
    let json: Value = serde_json::from_slice(&decoded).ok()?;
    let subject = json.get("sub")?.as_str()?;
    Some(
        subject
            .rsplit_once('|')
            .map(|(_, id)| id.to_string())
            .unwrap_or_else(|| subject.to_string()),
    )
}

fn cursor_cookie_from_access_token(access_token: &str) -> Option<String> {
    let user_id = extract_cursor_user_id(access_token)?;
    Some(format!("{user_id}%3A%3A{access_token}"))
}

fn normalize_cursor_session_cookie(token: &str) -> Option<String> {
    let token = token.trim();
    if token.is_empty() || token.bytes().any(|b| matches!(b, b'\r' | b'\n')) {
        return None;
    }
    let token = token
        .strip_prefix("WorkosCursorSessionToken=")
        .unwrap_or(token)
        .trim();
    if token.contains("%3A%3A") {
        Some(token.to_string())
    } else if token.contains("::") {
        Some(token.replace("::", "%3A%3A"))
    } else {
        cursor_cookie_from_access_token(token).or_else(|| Some(token.to_string()))
    }
}

fn cursor_state_db_path(account: &Account) -> Option<PathBuf> {
    if let Some(ref p) = account.credential_path {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    let app_data = std::env::var_os("APPDATA")?;
    let path = PathBuf::from(app_data)
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    path.is_file().then_some(path)
}

pub async fn cursor(account: &Account) -> Result<Snapshot, Failure> {
    let mut session_cookie = None;

    if let Some(secret) = account_secret(account)? {
        session_cookie = normalize_cursor_session_cookie(&secret);
    }

    if account.has_custom_secret && session_cookie.is_none() {
        return Err(Failure::new(
            Status::LoginRequired,
            "The saved Cursor session is invalid. Update or clear the account secret.",
        ));
    }

    if session_cookie.is_none()
        && let Ok(token) = std::env::var("CURSOR_SESSION_TOKEN")
    {
        session_cookie = normalize_cursor_session_cookie(&token);
    }

    if session_cookie.is_none()
        && let Some(path) = cursor_state_db_path(account)
        && let Ok(Some(access_token)) = crate::winsqlite::query_optional_text(
            &path,
            "SELECT value FROM ItemTable WHERE key = ?",
            "cursorAuth/accessToken",
        )
    {
        session_cookie = cursor_cookie_from_access_token(&access_token);
    }

    let cookie = session_cookie.ok_or_else(|| {
        Failure::new(
            Status::LoginRequired,
            "Cursor session was not found. Sign in to Cursor, set CURSOR_SESSION_TOKEN, or enter credentials in Account settings.",
        )
    })?;

    let client = crate::http::create_client()?;
    let cookie_header = format!("WorkosCursorSessionToken={cookie}");
    let response = client
        .get("https://cursor.com/api/usage-summary")
        .header("Cookie", &cookie_header)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| Failure::new(Status::Offline, format!("Cursor request failed: {e}")))?;

    let status_code = response.status();
    let headers = response.headers().clone();
    crate::http::handle_http_status(status_code, &headers, "Cursor")?;

    let bytes = crate::http::read_bounded_bytes(response, 1_048_576).await?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| Failure::new(Status::Unavailable, "Cursor returned invalid JSON."))?;

    let limits = parsers::cursor(&value);
    if limits.is_empty() {
        return Err(Failure::new(
            Status::Unavailable,
            "Cursor usage summary returned no plan usage.",
        ));
    }

    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
    snapshot.message = "Cursor Models · monthly allowance".into();
    Ok(snapshot)
}

fn grok_auth_path(account: &Account) -> Option<PathBuf> {
    if let Some(ref p) = account.credential_path {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Ok(home) = std::env::var("GROK_HOME") {
        let path = PathBuf::from(home).join("auth.json");
        if path.is_file() {
            return Some(path);
        }
    }
    let user_profile = std::env::var_os("USERPROFILE")?;
    let path = PathBuf::from(user_profile).join(".grok").join("auth.json");
    path.is_file().then_some(path)
}

pub async fn grok(account: &Account) -> Result<Snapshot, Failure> {
    let mut access_token = String::new();
    let mut user_id = String::new();

    // 1. Check custom secret in Windows Credential Manager
    if let Some(secret) = account_secret(account)? {
        let secret = secret.trim();
        if let Ok(v) = serde_json::from_str::<Value>(secret) {
            if let Some(tok) = v
                .get("key")
                .or_else(|| v.get("access_token"))
                .or_else(|| v.get("token"))
                .and_then(Value::as_str)
            {
                access_token = tok.trim().to_string();
            }
            if let Some(uid) = v
                .get("user_id")
                .or_else(|| v.get("userId"))
                .and_then(Value::as_str)
            {
                user_id = uid.trim().to_string();
            }
        } else if !secret.is_empty() {
            access_token = secret.to_string();
        }
    }

    if account.has_custom_secret && access_token.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "The saved Grok secret needs an access token. Update or clear the account secret.",
        ));
    }

    // 2. Check environment
    if access_token.is_empty()
        && let Ok(tok) = std::env::var("GROK_API_KEY")
            .or_else(|_| std::env::var("XAI_API_KEY"))
            .or_else(|_| std::env::var("GROK_SESSION_TOKEN"))
    {
        access_token = tok.trim().to_string();
    }

    // 3. Check auth.json
    if access_token.is_empty()
        && let Some(path) = grok_auth_path(account)
        && let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(entries) =
            serde_json::from_str::<std::collections::BTreeMap<String, Value>>(&content)
    {
        let mut best: Option<(String, String, Option<chrono::DateTime<Utc>>)> = None;
        for (scope, entry) in entries {
            let valid_scope = scope == "https://accounts.x.ai/sign-in"
                || scope
                    .strip_prefix("https://auth.x.ai::")
                    .is_some_and(|client| !client.is_empty());
            if !valid_scope {
                continue;
            }
            let key = entry
                .get("key")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if key.is_empty() {
                continue;
            }
            let uid = entry
                .get("user_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            let exp = entry.get("expires_at").and_then(|v| {
                v.as_str().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                })
            });
            let is_better = match (&best, exp) {
                (None, _) => true,
                (Some((_, _, Some(best_exp))), Some(this_exp)) => this_exp > *best_exp,
                (Some((_, _, None)), Some(_)) => true,
                _ => false,
            };
            if is_better {
                best = Some((key.to_string(), uid.to_string(), exp));
            }
        }
        if let Some((tok, uid, _)) = best {
            access_token = tok;
            user_id = uid;
        }
    }

    if access_token.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "Grok session was not found. Run 'grok login' to sign in or enter token in Account settings.",
        ));
    }

    let client = crate::http::create_client()?;
    let mut request = client
        .get("https://cli-chat-proxy.grok.com/v1/billing?format=credits")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("X-XAI-Token-Auth", "xai-grok-cli")
        .header("x-grok-client-mode", "cli")
        .header("x-grok-client-version", "1.0.0");
    if !user_id.is_empty() {
        request = request.header("x-userid", &user_id);
    }

    let response = request
        .send()
        .await
        .map_err(|e| Failure::new(Status::Offline, format!("Grok request failed: {e}")))?;

    let status_code = response.status();
    let headers = response.headers().clone();
    crate::http::handle_http_status(status_code, &headers, "Grok")?;

    let bytes = crate::http::read_bounded_bytes(response, 1_048_576).await?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| Failure::new(Status::Unavailable, "Grok returned invalid JSON."))?;

    let limits = parsers::grok(&value);
    if limits.is_empty() {
        return Err(Failure::new(
            Status::Unavailable,
            "Grok billing returned no usage limits.",
        ));
    }

    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
    snapshot.message = "Grok Build · shared credits".into();
    Ok(snapshot)
}

fn is_header_safe_token(token: &str) -> bool {
    let t = token.trim();
    !t.is_empty() && t.bytes().all(|b| (0x21..=0x7e).contains(&b))
}

fn copilot_cli_credentials() -> Vec<crate::credentials::StoredCredential> {
    let mut credentials = crate::credentials::enumerate_generic("https://github.com:")
        .into_iter()
        .filter(|c| c.target.ends_with(".copilot-cli"))
        .chain(crate::credentials::enumerate_generic(
            "copilot-cli/https://github.com:",
        ))
        .collect::<Vec<_>>();
    credentials.sort_by_key(|c| std::cmp::Reverse(c.last_written));
    credentials
}

fn gh_cli_credentials() -> Vec<crate::credentials::StoredCredential> {
    let mut credentials = crate::credentials::enumerate_generic("gh:github.com:");
    credentials.sort_by_key(|c| {
        (
            c.target != "gh:github.com:",
            std::cmp::Reverse(c.last_written),
        )
    });
    credentials
}

fn copilot_file_credentials() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let dirs = [
        std::env::var_os("USERPROFILE")
            .map(|u| PathBuf::from(u).join(".config").join("github-copilot")),
        std::env::var_os("LOCALAPPDATA").map(|l| PathBuf::from(l).join("github-copilot")),
    ];
    for dir in dirs.into_iter().flatten() {
        for file in ["hosts.json", "apps.json"] {
            let p = dir.join(file);
            if p.is_file()
                && let Ok(content) = std::fs::read_to_string(&p)
                && let Ok(val) = serde_json::from_str::<Value>(&content)
                && let Some(tok) = val
                    .get("github.com")
                    .and_then(|g| g.get("oauth_token").or_else(|| g.get("token")))
                    .and_then(Value::as_str)
            {
                out.push((format!("file {}", p.display()), tok.trim().to_string()));
            }
        }
    }
    out
}

fn copilot_custom_token(secret: Option<&str>) -> Result<String, Failure> {
    let secret = secret.unwrap_or_default().trim();
    let token = match serde_json::from_str::<Value>(secret) {
        Ok(v) => v
            .get("token")
            .or_else(|| v.get("oauth_token"))
            .or_else(|| v.get("key"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned(),
        Err(_) => secret.to_owned(),
    };
    if !is_header_safe_token(&token) {
        return Err(Failure::new(
            Status::LoginRequired,
            "The saved Copilot token is missing or invalid. Update the account secret or clear it to use the local sign-in.",
        ));
    }
    Ok(token)
}

pub async fn copilot(account: &Account) -> Result<Snapshot, Failure> {
    let mut candidates: Vec<(String, String)> = Vec::new();

    // 1. Custom secret in Credential Manager
    if account.has_custom_secret {
        let secret = crate::credentials::read_account_secret(&account.id);
        candidates.push((
            "custom secret".into(),
            copilot_custom_token(secret.as_deref())?,
        ));
    } else {
        // 2. Environment variables
        for var in [
            "COPILOT_GITHUB_TOKEN",
            "GH_TOKEN",
            "GITHUB_TOKEN",
            "COPILOT_TOKEN",
        ] {
            if let Ok(tok) = std::env::var(var) {
                let tok = tok.trim();
                if !tok.is_empty() {
                    candidates.push((format!("env {var}"), tok.to_string()));
                }
            }
        }

        // 3. Stored Windows credentials from Copilot CLI and GitHub CLI
        for cred in copilot_cli_credentials() {
            candidates.push((
                format!("credential {}", cred.target),
                cred.secret.trim().to_string(),
            ));
        }
        for cred in gh_cli_credentials() {
            candidates.push((
                format!("credential {}", cred.target),
                cred.secret.trim().to_string(),
            ));
        }

        // 4. Local file credentials
        for (src, tok) in copilot_file_credentials() {
            candidates.push((src, tok));
        }
    }

    // Filter valid tokens and deduplicate
    let mut deduped: Vec<(String, String)> = Vec::new();
    for (src, tok) in candidates {
        if is_header_safe_token(&tok) && !deduped.iter().any(|(_, existing)| existing == &tok) {
            deduped.push((src, tok));
        }
    }

    if deduped.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "GitHub Copilot login was not found. Sign in via Copilot CLI or GitHub CLI, or provide a token in Account settings.",
        ));
    }

    let client = crate::http::create_client()?;
    let mut last_failure: Option<Failure> = None;

    for (source, token) in &deduped {
        let auth_val = if token.starts_with("token ") || token.starts_with("Bearer ") {
            token.clone()
        } else {
            format!("token {token}")
        };

        let res = client
            .get("https://api.github.com/copilot_internal/user")
            .header("Authorization", auth_val)
            .header("Accept", "application/json")
            .header("User-Agent", "TokenFuel/0.1.0")
            .header("Editor-Version", "vscode/1.95.0")
            .header("Editor-Plugin-Version", "copilot/1.250.0")
            .send()
            .await;

        let response = match res {
            Ok(r) => r,
            Err(e) => {
                last_failure = Some(Failure::new(
                    Status::Offline,
                    format!("GitHub Copilot request failed ({source}): {e}"),
                ));
                continue;
            }
        };

        let status_code = response.status();
        let headers = response.headers().clone();

        if status_code == reqwest::StatusCode::UNAUTHORIZED
            || status_code == reqwest::StatusCode::FORBIDDEN
        {
            last_failure = Some(Failure::new(
                Status::LoginRequired,
                format!("GitHub Copilot rejected authentication from {source}."),
            ));
            continue;
        }

        if status_code == reqwest::StatusCode::NOT_FOUND {
            last_failure = Some(Failure::new(
                Status::Unavailable,
                format!("Account ({source}) has no active Copilot subscription."),
            ));
            continue;
        }

        if let Err(e) = crate::http::handle_http_status(status_code, &headers, "GitHub Copilot") {
            last_failure = Some(e.into());
            continue;
        }

        let bytes = match crate::http::read_bounded_bytes(response, 1_048_576).await {
            Ok(b) => b,
            Err(e) => {
                last_failure = Some(e.into());
                continue;
            }
        };

        let value: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                last_failure = Some(Failure::new(
                    Status::Unavailable,
                    "GitHub Copilot returned invalid JSON.",
                ));
                continue;
            }
        };

        let limits = parsers::copilot(&value);
        if limits.is_empty() {
            last_failure = Some(Failure::new(
                Status::Unavailable,
                "GitHub Copilot returned no recognized quota pools.",
            ));
            continue;
        }

        let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
        if let Some(plan) = value.get("copilot_plan").and_then(Value::as_str) {
            snapshot.message = format!("GitHub Copilot · {plan}");
        } else {
            snapshot.message = "GitHub Copilot".into();
        }
        return Ok(snapshot);
    }

    Err(last_failure.unwrap_or_else(|| {
        Failure::new(
            Status::Unavailable,
            "Could not retrieve GitHub Copilot quota.",
        )
    }))
}

const ANTIGRAVITY_ENDPOINTS: &[&str] = &[
    "https://daily-cloudcode-pa.googleapis.com",
    "https://daily-cloudcode-pa.sandbox.googleapis.com",
    "https://cloudcode-pa.googleapis.com",
];

struct AntigravityCreds {
    access_token: String,
    refresh_token: Option<String>,
    expiry: Option<String>,
}

fn read_antigravity_token_data(account: &Account) -> Option<AntigravityCreds> {
    // 1. Custom secret in Credential Manager
    if account.has_custom_secret {
        let secret = account_secret(account).ok()??;
        let secret = secret.trim();
        if let Ok(v) = serde_json::from_str::<Value>(secret) {
            let tok = v
                .get("token")
                .and_then(|t| t.get("access_token"))
                .or_else(|| v.get("access_token"))
                .or_else(|| v.get("token"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let ref_tok = v
                .get("token")
                .and_then(|t| t.get("refresh_token"))
                .or_else(|| v.get("refresh_token"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let exp = v
                .get("token")
                .and_then(|t| t.get("expiry"))
                .or_else(|| v.get("expiry"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            if let Some(access_token) = tok {
                return Some(AntigravityCreds {
                    access_token,
                    refresh_token: ref_tok,
                    expiry: exp,
                });
            }
        } else if !secret.is_empty() {
            return Some(AntigravityCreds {
                access_token: secret.to_string(),
                refresh_token: None,
                expiry: None,
            });
        }
        return None;
    }

    // 2. Env vars
    for var in [
        "ANTIGRAVITY_TOKEN",
        "ANTIGRAVITY_ACCESS_TOKEN",
        "GOOGLE_ACCESS_TOKEN",
        "GEMINI_CLI_TOKEN",
    ] {
        if let Ok(tok) = std::env::var(var) {
            let tok = tok.trim();
            if !tok.is_empty() {
                return Some(AntigravityCreds {
                    access_token: tok.to_string(),
                    refresh_token: std::env::var("ANTIGRAVITY_REFRESH_TOKEN").ok(),
                    expiry: None,
                });
            }
        }
    }

    // 3. Stored Windows credential `gemini:antigravity`
    if let Some(content) = crate::credentials::read_generic("gemini:antigravity")
        && let Ok(v) = serde_json::from_str::<Value>(&content)
    {
        let tok = v
            .get("token")
            .and_then(|t| t.get("access_token"))
            .or_else(|| v.get("access_token"))
            .or_else(|| v.get("token"))
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string());
        let ref_tok = v
            .get("token")
            .and_then(|t| t.get("refresh_token"))
            .or_else(|| v.get("refresh_token"))
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string());
        let exp = v
            .get("token")
            .and_then(|t| t.get("expiry"))
            .or_else(|| v.get("expiry"))
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string());
        if let Some(access_token) = tok {
            return Some(AntigravityCreds {
                access_token,
                refresh_token: ref_tok,
                expiry: exp,
            });
        }
    }

    // 4. File credentials
    let file_dirs = [
        std::env::var_os("USERPROFILE")
            .map(|u| PathBuf::from(u).join(".antigravity").join("auth.json")),
        std::env::var_os("USERPROFILE").map(|u| {
            PathBuf::from(u)
                .join(".config")
                .join("antigravity")
                .join("auth.json")
        }),
        std::env::var_os("LOCALAPPDATA")
            .map(|l| PathBuf::from(l).join("antigravity").join("auth.json")),
    ];
    for opt in file_dirs.into_iter().flatten() {
        if opt.is_file()
            && let Ok(content) = std::fs::read_to_string(&opt)
            && let Ok(v) = serde_json::from_str::<Value>(&content)
        {
            let tok = v
                .get("token")
                .and_then(|t| t.get("access_token"))
                .or_else(|| v.get("access_token"))
                .or_else(|| v.get("token"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let ref_tok = v
                .get("token")
                .and_then(|t| t.get("refresh_token"))
                .or_else(|| v.get("refresh_token"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let exp = v
                .get("token")
                .and_then(|t| t.get("expiry"))
                .or_else(|| v.get("expiry"))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            if let Some(access_token) = tok {
                return Some(AntigravityCreds {
                    access_token,
                    refresh_token: ref_tok,
                    expiry: exp,
                });
            }
        }
    }

    None
}

fn installed_antigravity_oauth_clients() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        paths.push(local.join("Programs/Antigravity/resources/bin/language_server.exe"));
        paths.push(local.join("agy/bin/agy.exe"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        paths.push(
            PathBuf::from(program_files).join("Antigravity/resources/bin/language_server.exe"),
        );
    }
    for path in paths {
        if let Ok(bytes) = std::fs::read(&path) {
            let clients = oauth_clients_from_bytes(&bytes);
            if !clients.is_empty() {
                return clients;
            }
        }
    }
    Vec::new()
}

fn oauth_clients_from_bytes(bytes: &[u8]) -> Vec<(String, String)> {
    const CLIENT_ID_SUFFIX: &str = ".apps.googleusercontent.com";
    const CLIENT_SECRET_PREFIX: &str = "GOCSPX-";
    let mut ids = Vec::new();
    let mut secrets = Vec::new();
    for run in
        bytes.split(|byte| !byte.is_ascii_alphanumeric() && !matches!(*byte, b'.' | b'_' | b'-'))
    {
        for (suffix_at, _) in run
            .windows(CLIENT_ID_SUFFIX.len())
            .enumerate()
            .filter(|(_, part)| *part == CLIENT_ID_SUFFIX.as_bytes())
        {
            for (hyphen, byte) in run[..suffix_at].iter().enumerate() {
                if *byte != b'-' {
                    continue;
                }
                let mut start = hyphen;
                while start > 0 && run[start - 1].is_ascii_digit() {
                    start -= 1;
                }
                let client_hash = &run[hyphen + 1..suffix_at];
                if hyphen - start < 10
                    || !(20..=80).contains(&client_hash.len())
                    || !client_hash
                        .iter()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
                {
                    continue;
                }
                if let Ok(client_id) =
                    std::str::from_utf8(&run[start..suffix_at + CLIENT_ID_SUFFIX.len()])
                    && !ids.iter().any(|existing| existing == client_id)
                {
                    ids.push(client_id.to_owned());
                }
            }
        }
        for (start, _) in run
            .windows(CLIENT_SECRET_PREFIX.len())
            .enumerate()
            .filter(|(_, part)| *part == CLIENT_SECRET_PREFIX.as_bytes())
        {
            let Some(candidate) = run.get(start..start + 35) else {
                continue;
            };
            if candidate
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
                && let Ok(secret) = std::str::from_utf8(candidate)
                && !secrets.iter().any(|existing| existing == secret)
            {
                secrets.push(secret.to_owned());
            }
        }
    }
    ids.into_iter()
        .flat_map(|id| {
            secrets
                .iter()
                .cloned()
                .map(move |secret| (id.clone(), secret))
        })
        .collect()
}

async fn refresh_antigravity_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Option<String> {
    let clients = installed_antigravity_oauth_clients();
    for (client_id, client_secret) in clients {
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];
        if let Ok(res) = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            && res.status().is_success()
            && let Ok(v) = res.json::<Value>().await
            && let Some(tok) = v.get("access_token").and_then(Value::as_str)
        {
            return Some(tok.trim().to_string());
        }
    }
    None
}

pub async fn antigravity(account: &Account) -> Result<Snapshot, Failure> {
    let creds = read_antigravity_token_data(account).ok_or_else(|| {
        Failure::new(
            Status::LoginRequired,
            "Google Antigravity credentials were not found. Sign in via Antigravity or provide an access token in Account settings.",
        )
    })?;

    let client = crate::http::create_client()?;

    let is_expired = creds
        .expiry
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .is_some_and(|exp| exp.with_timezone(&Utc) <= Utc::now() + chrono::Duration::seconds(60));

    let mut token = creds.access_token;
    if (token.is_empty() || is_expired)
        && let Some(ref ref_tok) = creds.refresh_token
        && let Some(new_tok) = refresh_antigravity_token(&client, ref_tok).await
    {
        token = new_tok;
    }

    if token.is_empty() {
        return Err(Failure::new(
            Status::LoginRequired,
            "Google Antigravity access token is missing or expired. Sign in via Antigravity or update settings.",
        ));
    }

    let mut auth_failed = false;
    let mut last_failure: Option<Failure> = None;

    for base_url in ANTIGRAVITY_ENDPOINTS {
        // Step 1: Discover Project ID
        let load_res = client
            .post(format!("{base_url}/v1internal:loadCodeAssist"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "antigravity")
            .json(&serde_json::json!({
                "metadata": {
                    "ideType": "ANTIGRAVITY"
                }
            }))
            .send()
            .await;

        let load_resp = match load_res {
            Ok(r) => r,
            Err(e) => {
                last_failure = Some(Failure::new(
                    Status::Offline,
                    format!("Antigravity connection failed: {e}"),
                ));
                continue;
            }
        };

        let load_status = load_resp.status();
        if load_status == reqwest::StatusCode::UNAUTHORIZED
            || load_status == reqwest::StatusCode::FORBIDDEN
        {
            auth_failed = true;
            continue;
        }

        let project_id = if load_status.is_success() {
            let bytes = crate::http::read_bounded_bytes(load_resp, 524_288)
                .await
                .ok();
            bytes
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .and_then(|v| {
                    v.get("cloudaicompanionProject")
                        .or_else(|| v.get("project"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
        } else {
            None
        };

        // Step 2: Retrieve User Quota Summary (if project discovered)
        if let Some(ref proj) = project_id {
            let summary_res = client
                .post(format!("{base_url}/v1internal:retrieveUserQuotaSummary"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .header("User-Agent", "antigravity")
                .json(&serde_json::json!({ "project": proj }))
                .send()
                .await;

            if let Ok(summary_resp) = summary_res
                && summary_resp.status().is_success()
                && let Ok(bytes) = crate::http::read_bounded_bytes(summary_resp, 1_048_576).await
                && let Ok(v) = serde_json::from_slice::<Value>(&bytes)
            {
                let limits = parsers::antigravity(&v);
                if !limits.is_empty() {
                    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
                    snapshot.message = format!("Google Antigravity · {proj}");
                    return Ok(snapshot);
                }
            }
        }

        // Step 3: Fallback to fetchAvailableModels
        let body = match &project_id {
            Some(p) => serde_json::json!({ "project": p }),
            None => serde_json::json!({}),
        };

        let models_res = client
            .post(format!("{base_url}/v1internal:fetchAvailableModels"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "antigravity")
            .json(&body)
            .send()
            .await;

        if let Ok(models_resp) = models_res {
            let status = models_resp.status();
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                auth_failed = true;
                continue;
            }
            if status.is_success()
                && let Ok(bytes) = crate::http::read_bounded_bytes(models_resp, 1_048_576).await
                && let Ok(v) = serde_json::from_slice::<Value>(&bytes)
            {
                let limits = parsers::antigravity(&v);
                if !limits.is_empty() {
                    let mut snapshot = Snapshot::ready(&account.id, account.provider, limits);
                    snapshot.message = "Google Antigravity · models".into();
                    return Ok(snapshot);
                }
            }
        }
    }

    if auth_failed {
        return Err(Failure::new(
            Status::LoginRequired,
            "Google Antigravity authentication rejected or expired. Sign in to Antigravity or update settings.",
        ));
    }

    Err(last_failure.unwrap_or_else(|| {
        Failure::new(
            Status::Unavailable,
            "Google Antigravity endpoints returned no available quota pools.",
        )
    }))
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
                "Visible Usage capture is not supported for this provider. Choose its local source or a manual snapshot.",
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
    fn missing_custom_secrets_fail_before_any_local_fallback_or_network_read() {
        let mut account = crate::config::Config::default().accounts.remove(0);
        account.id = uuid::Uuid::new_v4().to_string();
        account.has_custom_secret = true;
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            for result in [
                opencode(&account).await,
                cursor(&account).await,
                grok(&account).await,
                copilot(&account).await,
            ] {
                assert!(matches!(
                    result,
                    Err(Failure {
                        status: Status::LoginRequired,
                        ..
                    })
                ));
            }
        });
        assert!(read_antigravity_token_data(&account).is_none());
    }
    #[test]
    fn custom_copilot_tokens_reject_missing_and_invalid_secrets() {
        for secret in [
            None,
            Some(""),
            Some("{}"),
            Some("{\"token\":\"\"}"),
            Some("bad\ntoken"),
        ] {
            assert!(matches!(
                copilot_custom_token(secret),
                Err(Failure {
                    status: Status::LoginRequired,
                    ..
                })
            ));
        }
        for secret in [
            " test-token ",
            "{\"token\":\"test-token\"}",
            "{\"oauth_token\":\"test-token\"}",
            "{\"key\":\"test-token\"}",
        ] {
            assert_eq!(
                copilot_custom_token(Some(secret))
                    .unwrap_or_else(|_| panic!("valid token rejected")),
                "test-token"
            );
        }
    }
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
