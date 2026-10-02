pub mod config;
pub mod credentials;
pub mod http;
pub mod providers;
pub mod winsqlite;
/// Operator-facing diagnostic; callers must obtain local-session consent first.
pub async fn diagnose_codex() -> Result<serde_json::Value, String> {
    let c = config::Config::default();
    let a = c
        .accounts
        .into_iter()
        .find(|a| a.provider == tokenfuel_core::Provider::Openai)
        .unwrap();
    let s = providers::codex(&a).await.map_err(|e| e.message)?;
    let quotas:Vec<_>=s.limits.iter().map(|q|serde_json::json!({"name":q.name,"product":q.product,"remainingPercent":q.remaining_percent,"resetsAt":q.resets_at,"unlimited":q.unlimited})).collect();
    Ok(serde_json::json!({"status":s.status,"quotas":quotas}))
}
use chrono::Utc;
use config::{Account, Config, Connection, Settings};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;
use tokenfuel_core::{Snapshot, Source, Status, UsageLimit};

fn local_ui(window: &tauri::WebviewWindow) -> Result<(), String> {
    let url = window.url().map_err(|_| "Widget origin unavailable.")?;
    let local = url.scheme() == "tauri"
        || url.host_str() == Some("tauri.localhost")
        || cfg!(debug_assertions)
            && [Some("localhost"), Some("127.0.0.1")].contains(&url.host_str())
            && url.port() == Some(1420);
    if window.label() != "main" || !local {
        return Err("Provider windows cannot access TokenFuel commands.".into());
    }
    Ok(())
}

struct State {
    config: Mutex<Config>,
    path: PathBuf,
    busy: AtomicBool,
    failures: Mutex<BTreeMap<String, u32>>,
    movement: AtomicU64,
    provider_loads: Mutex<BTreeMap<String, u64>>,
}
#[tauri::command]
fn read_config(window: tauri::WebviewWindow, state: tauri::State<State>) -> Result<Config, String> {
    local_ui(&window)?;
    let mut config = state.config.lock().unwrap().clone();
    for a in &config.accounts {
        if let Some(snapshot) = config.cached.get_mut(&a.id) {
            snapshot.mark_stale(
                Utc::now(),
                config.settings.interval_secs,
                a.connection == Connection::Browser,
            );
        }
    }
    Ok(config)
}
#[tauri::command]
fn save_settings(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<State>,
    mut settings: Settings,
) -> Result<(), String> {
    local_ui(&window)?;
    if !["bars", "rings"].contains(&settings.view.as_str())
        || !["system", "light", "dark"].contains(&settings.theme.as_str())
    {
        return Err("Invalid appearance setting.".into());
    }
    settings.interval_secs = settings.interval_secs.clamp(30, 3600);
    if let Some(w) = app.get_webview_window("main") {
        w.set_always_on_top(settings.always_on_top)
            .map_err(|_| "Could not change window position preference.")?;
    }
    if settings.startup {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|_| "Could not change startup preference.")?;
    let mut c = state.config.lock().unwrap();
    c.settings = settings;
    config::write(&state.path, &c)
}
fn is_local_session(conn: &Connection) -> bool {
    matches!(
        conn,
        Connection::CodexCli
            | Connection::ClaudeCli
            | Connection::CursorLocal
            | Connection::GrokCli
            | Connection::CopilotCli
            | Connection::AntigravityLocal
    )
}

fn uses_local_session(account: &Account) -> bool {
    is_local_session(&account.connection)
        && !(account.has_custom_secret && account.connection.supports_custom_secret())
}

fn saved_secret_flag(account: &Account, existing: Option<&Account>) -> bool {
    account.enabled && existing.is_some_and(|old| old.has_custom_secret)
}

#[cfg(test)]
mod account_secret_tests {
    use super::*;

    #[test]
    fn disconnect_and_ui_payload_cannot_restore_a_secret_flag() {
        let mut account = config::Config::default().accounts.remove(0);
        account.has_custom_secret = true;
        let old = account.clone();
        account.enabled = false;
        assert!(!saved_secret_flag(&account, Some(&old)));
        account.enabled = true;
        assert!(saved_secret_flag(&account, Some(&old)));
        assert!(!saved_secret_flag(&account, None));
        let mut cleared = old.clone();
        cleared.has_custom_secret = false;
        assert!(!saved_secret_flag(&account, Some(&cleared)));
    }

    #[test]
    fn secrets_do_not_exempt_cli_connections_that_ignore_them() {
        let mut account = config::Config::default().accounts.remove(0);
        account.has_custom_secret = true;
        for connection in [Connection::CodexCli, Connection::ClaudeCli] {
            account.connection = connection;
            assert!(uses_local_session(&account));
        }
        for connection in [
            Connection::CursorLocal,
            Connection::GrokCli,
            Connection::CopilotCli,
            Connection::AntigravityLocal,
        ] {
            account.connection = connection;
            assert!(!uses_local_session(&account));
            account.has_custom_secret = false;
            assert!(uses_local_session(&account));
            account.has_custom_secret = true;
        }
    }
}

#[tauri::command]
fn save_account(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<State>,
    mut account: Account,
) -> Result<(), String> {
    local_ui(&window)?;
    if account.label.len() > 80 || account.workspace.len() > 120 {
        return Err("Account label is too long.".into());
    }
    let mut c = state.config.lock().unwrap();
    // Secret metadata comes from native storage commands, never the UI payload.
    account.has_custom_secret =
        saved_secret_flag(&account, c.accounts.iter().find(|a| a.id == account.id));
    if account.enabled
        && uses_local_session(&account)
        && c.accounts.iter().any(|a| {
            a.id != account.id
                && a.enabled
                && a.provider == account.provider
                && uses_local_session(a)
        })
    {
        let remedy = if account.connection.supports_custom_secret() {
            "Disconnect the existing automatic account or configure an account-specific token/cookie first."
        } else {
            "Disconnect the existing automatic account first; this connection cannot use account-specific tokens."
        };
        return Err(format!(
            "{} uses the current local sign-in. {remedy}",
            account.provider.display_name(),
        ));
    }
    let clear = c
        .accounts
        .iter()
        .find(|a| a.id == account.id)
        .is_some_and(|a| {
            a.provider != account.provider
                || a.connection != account.connection
                || a.enabled != account.enabled
        });
    if clear {
        c.cached.remove(&account.id);
    }
    if !account.enabled {
        let _ = credentials::delete_account_secret(&account.id);
        account.has_custom_secret = false;
        if let Some(w) = app.get_webview_window(&format!("provider-{}", account.id)) {
            let _ = w.close();
        }
    }
    if let Some(old) = c.accounts.iter_mut().find(|a| a.id == account.id) {
        account.revision = old.revision.saturating_add(1);
        account.manual_limits = old.manual_limits.clone();
        *old = account;
    } else {
        account.id = uuid::Uuid::new_v4().to_string();
        account.revision = 0;
        account.manual_limits.clear();
        c.accounts.push(account);
    }
    config::write(&state.path, &c)
}
#[tauri::command]
fn remove_account(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<State>,
    id: String,
) -> Result<(), String> {
    local_ui(&window)?;
    let mut c = state.config.lock().unwrap();
    c.accounts.retain(|a| a.id != id);
    c.cached.remove(&id);
    let _ = credentials::delete_account_secret(&id);
    if let Some(w) = app.get_webview_window(&format!("provider-{id}")) {
        let _ = w.close();
    }
    config::write(&state.path, &c)
}
#[tauri::command]
fn save_account_secret(
    window: tauri::WebviewWindow,
    state: tauri::State<State>,
    id: String,
    secret: String,
) -> Result<(), String> {
    local_ui(&window)?;
    let mut c = state.config.lock().unwrap();
    let a = c
        .accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or("Account missing.")?;
    if !a.connection.supports_custom_secret() {
        return Err(
            "This connection uses its local sign-in and does not support account-specific secrets."
                .into(),
        );
    }
    credentials::save_account_secret(&id, &secret)?;
    a.has_custom_secret = true;
    a.revision = a.revision.saturating_add(1);
    c.cached.remove(&id);
    config::write(&state.path, &c)
}
#[tauri::command]
fn clear_account_secret(
    window: tauri::WebviewWindow,
    state: tauri::State<State>,
    id: String,
) -> Result<(), String> {
    local_ui(&window)?;
    let mut c = state.config.lock().unwrap();
    let a = c
        .accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or("Account missing.")?;
    let _ = credentials::delete_account_secret(&id);
    a.has_custom_secret = false;
    a.revision = a.revision.saturating_add(1);
    c.cached.remove(&id);
    config::write(&state.path, &c)
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manual {
    name: String,
    unit: String,
    period: String,
    used: Option<String>,
    total: Option<String>,
    remaining_percent: Option<f64>,
    unlimited: bool,
    resets_at: Option<String>,
}
#[tauri::command]
fn set_manual(
    window: tauri::WebviewWindow,
    state: tauri::State<State>,
    id: String,
    input: Manual,
) -> Result<(), String> {
    local_ui(&window)?;
    use std::str::FromStr;
    let mut c = state.config.lock().unwrap();
    let a = c
        .accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or("Account missing.")?;
    if input.name.is_empty()
        || input.name.len() > 80
        || input.unit.len() > 16
        || input.period.len() > 40
    {
        return Err("Invalid quota description.".into());
    }
    let mut q = if let Some(p) = input.remaining_percent {
        if !(0.0..=100.0).contains(&p) {
            return Err("Remaining percentage must be 0–100.".into());
        }
        UsageLimit::percentage(
            "manual",
            &input.name,
            "Manual snapshot",
            &input.period,
            100.0 - p,
            Source::Manual,
        )
        .ok_or("Invalid percentage.")?
    } else {
        let used = rust_decimal::Decimal::from_str(input.used.as_deref().unwrap_or("0"))
            .map_err(|_| "Invalid used amount.")?;
        let total = input
            .total
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(rust_decimal::Decimal::from_str)
            .transpose()
            .map_err(|_| "Invalid limit amount.")?;
        UsageLimit::amounts(
            "manual",
            &input.name,
            "Manual snapshot",
            &input.unit,
            &input.period,
            used,
            total,
            Source::Manual,
        )?
    };
    q.unlimited = input.unlimited;
    if q.unlimited {
        q.remaining_percent = None;
        q.remaining = None;
        q.total = None;
    }
    q.resets_at = input
        .resets_at
        .filter(|s| !s.is_empty())
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
        .transpose()
        .map_err(|_| "Reset must include a timezone (ISO 8601).")?;
    a.manual_limits = vec![q];
    a.revision = a.revision.saturating_add(1);
    a.connection = Connection::Manual;
    a.enabled = true;
    c.cached.remove(&id);
    config::write(&state.path, &c)
}
#[tauri::command]
fn open_provider(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<State>,
    id: String,
) -> Result<(), String> {
    local_ui(&window)?;
    let c = state.config.lock().unwrap();
    let a = c
        .accounts
        .iter()
        .find(|a| a.id == id)
        .ok_or("Account missing.")?;
    if !a.enabled || !a.experimental {
        return Err("Enable experimental browser access first.".into());
    }
    let label = format!("provider-{}", a.id);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let p = a.provider;
    tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::External(providers::provider_url(p).parse().unwrap()),
    )
    .title("TokenFuel · isolated provider sign-in")
    .inner_size(1050., 780.)
    .data_directory(
        app.path()
            .app_config_dir()
            .map_err(|_| "Provider profile directory unavailable.")?
            .join("webviews")
            .join(&a.id),
    )
    .incognito(true)
    .on_page_load(|window, payload| {
        if payload.event() == tauri::webview::PageLoadEvent::Finished {
            let state = window.state::<State>();
            let mut loads = state.provider_loads.lock().unwrap();
            let count = loads.entry(window.label().to_owned()).or_default();
            *count = count.saturating_add(1);
        }
    })
    .on_navigation(move |url| providers::safe_navigation(url, p))
    .build()
    .map_err(|_| "Could not open isolated provider window.")?;
    Ok(())
}
#[tauri::command]
async fn refresh(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    local_ui(&window)?;
    poll(&app, true).await;
    Ok(())
}
async fn poll(app: &tauri::AppHandle, force: bool) {
    let state = app.state::<State>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return;
    }
    let c = state.config.lock().unwrap().clone();
    let mut changed = false;
    let mut to_poll = Vec::new();
    for a in &c.accounts {
        let previous = c.cached.get(&a.id);
        if !force
            && previous.is_some()
            && (!a.enabled || matches!(a.connection, Connection::Manual | Connection::Browser))
        {
            if let Some(s) = state.config.lock().unwrap().cached.get_mut(&a.id) {
                changed |= s.mark_stale(
                    Utc::now(),
                    c.settings.interval_secs,
                    a.connection == Connection::Browser,
                );
            }
            continue;
        }
        if a.enabled
            && previous.is_some_and(|s| {
                (!force || s.status == Status::RateLimited)
                    && s.retry_at.is_some_and(|t| t > Utc::now())
            })
        {
            continue;
        }
        if !force
            && previous.is_some_and(|s| {
                s.fetched_at.is_some_and(|t| {
                    (Utc::now() - t).num_seconds() < c.settings.interval_secs as i64
                })
            })
        {
            continue;
        }
        // Browser captures are explicit snapshots: a static page is not an automatic refresh source.
        if !force && a.connection == Connection::Browser {
            if previous.is_some_and(|s| {
                s.fetched_at
                    .is_some_and(|t| (Utc::now() - t).num_seconds() >= 600)
            }) && let Some(s) = state.config.lock().unwrap().cached.get_mut(&a.id)
            {
                s.status = Status::Stale;
                changed = true;
            }
            continue;
        }
        to_poll.push(a.clone());
    }

    if !to_poll.is_empty() {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
        let (tx, mut rx) = tokio::sync::mpsc::channel(to_poll.len());
        for a in to_poll {
            let sem = semaphore.clone();
            let app_handle = app.clone();
            let tx = tx.clone();
            tauri::async_runtime::spawn(async move {
                let _permit = sem.acquire().await;
                let res = providers::fetch(&app_handle, &a).await;
                let _ = tx.send((a, res)).await;
            });
        }
        drop(tx);

        while let Some((a, res)) = rx.recv().await {
            let previous = c.cached.get(&a.id);
            let mut snapshot = match res {
                Ok(s) => {
                    state.failures.lock().unwrap().remove(&a.id);
                    s
                }
                Err(e) => {
                    let mut f = state.failures.lock().unwrap();
                    let failures = f.entry(a.id.clone()).or_default();
                    *failures = failures.saturating_add(1);
                    let mut s = previous.cloned().unwrap_or_else(|| {
                        Snapshot::empty(&a.id, a.provider, e.status, &e.message)
                    });
                    s.status = e.status;
                    s.message = e.message;
                    s.retry_at = Some(tokenfuel_core::scheduler::next_attempt(
                        Utc::now(),
                        c.settings.interval_secs,
                        *failures,
                        e.retry_after,
                    ));
                    s
                }
            };
            if a.connection == Connection::Manual {
                snapshot.fetched_at = snapshot.limits.first().map(|q| q.observed_at);
                snapshot.message = "Manual snapshot · not verified automatic tracking.".into();
            }
            snapshot.mark_stale(
                Utc::now(),
                c.settings.interval_secs,
                a.connection == Connection::Browser,
            );
            // Commit and notify only if the account still matches this in-flight request.
            let mut live = state.config.lock().unwrap();
            if !live.accounts.iter().any(|account| {
                account.id == a.id && account.revision == a.revision && account.enabled == a.enabled
            }) {
                continue;
            }
            if c.settings.alerts && snapshot.status == Status::Available {
                for q in &snapshot.limits {
                    if q.source == Source::Manual {
                        continue;
                    }
                    for threshold in [20., 10.] {
                        if q.remaining_percent.is_some_and(|p| p <= threshold)
                            && previous
                                .and_then(|s| s.limits.iter().find(|old| old.id == q.id))
                                .and_then(|q| q.remaining_percent)
                                .is_some_and(|p| p > threshold)
                        {
                            let _ = app
                                .notification()
                                .builder()
                                .title("TokenFuel · low remaining allowance")
                                .body(format!(
                                    "{}: {} has {:.0}% remaining",
                                    a.label,
                                    q.name,
                                    q.remaining_percent.unwrap()
                                ))
                                .show();
                        }
                    }
                }
            }
            if live.accounts.iter().any(|account| {
                account.id == a.id
                    && account.provider == a.provider
                    && account.connection == a.connection
                    && account.enabled == a.enabled
                    && account.revision == a.revision
            }) {
                live.cached.insert(a.id.clone(), snapshot);
                changed = true;
            }
        }
    }
    if changed {
        let live = state.config.lock().unwrap();
        let _ = config::write(&state.path, &live);
        let _ = app.emit("usage-updated", live.clone());
    }
    state.busy.store(false, Ordering::SeqCst);
}
#[tauri::command]
fn snap_window(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<State>,
) -> Result<(), String> {
    local_ui(&window)?;
    let w = app.get_webview_window("main").ok_or("Widget missing.")?;
    let mut p = w.outer_position().map_err(|_| "Position unavailable.")?;
    let original = p;
    let snap = state.config.lock().unwrap().settings.snap_to_edges;
    if snap && let (Ok(Some(m)), Ok(s)) = (w.current_monitor(), w.outer_size()) {
        let origin = &m.work_area().position;
        let size = &m.work_area().size;
        let right = origin.x + size.width as i32 - s.width as i32;
        let bottom = origin.y + size.height as i32 - s.height as i32;
        for x in [origin.x, right] {
            if (p.x - x).abs() < 24 {
                p.x = x
            }
        }
        for y in [origin.y, bottom] {
            if (p.y - y).abs() < 24 {
                p.y = y
            }
        }
        if p != original {
            let _ = w.set_position(p);
        }
    }
    let mut c = state.config.lock().unwrap();
    c.position = Some((p.x, p.y));
    config::write(&state.path, &c)
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_config,
            save_settings,
            save_account,
            remove_account,
            save_account_secret,
            clear_account_secret,
            set_manual,
            open_provider,
            refresh,
            snap_window
        ])
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let config = config::read(&path);
            if let Some(w) = app.get_webview_window("main") {
                w.set_always_on_top(config.settings.always_on_top)?;
                if let Some((x, y)) = config.position
                    && w.available_monitors()?.iter().any(|m| {
                        x >= m.position().x
                            && y >= m.position().y
                            && x < m.position().x + m.size().width as i32
                            && y < m.position().y + m.size().height as i32
                    })
                {
                    w.set_position(tauri::PhysicalPosition::new(x, y))?;
                }
            }
            app.manage(State {
                config: Mutex::new(config),
                path,
                busy: AtomicBool::new(false),
                failures: Mutex::new(BTreeMap::new()),
                movement: AtomicU64::new(0),
                provider_loads: Mutex::new(BTreeMap::new()),
            });
            use tauri::{
                menu::{Menu, MenuItem},
                tray::TrayIconBuilder,
            };
            let show = MenuItem::with_id(app, "show", "Show / hide widget", true, None::<&str>)?;
            let update = MenuItem::with_id(app, "refresh", "Refresh usage", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit TokenFuel", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &update, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("TokenFuel")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "refresh" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            poll(&app, true).await;
                        });
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    poll(&handle, false).await;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            });
            Ok(())
        })
        .on_window_event(|w, event| {
            if w.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = w.hide();
                }
                if let tauri::WindowEvent::Moved(p) = event {
                    let state = w.state::<State>();
                    state.config.lock().unwrap().position = Some((p.x, p.y));
                    let generation = state.movement.fetch_add(1, Ordering::SeqCst) + 1;
                    let app = w.app_handle().clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let state = app.state::<State>();
                        if state.movement.load(Ordering::SeqCst) == generation
                            && let Some(window) = app.get_webview_window("main")
                        {
                            let _ = snap_window(window, app.clone(), state);
                        }
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("TokenFuel could not start");
}
