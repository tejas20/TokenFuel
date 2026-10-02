use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Write, path::Path};
use tokenfuel_core::{Provider, Snapshot, UsageLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Connection {
    CodexCli,
    ClaudeCli,
    Browser,
    GeminiWeb,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(default)]
    pub revision: u64,
    pub id: String,
    pub provider: Provider,
    pub label: String,
    pub workspace: String,
    pub connection: Connection,
    pub enabled: bool,
    pub experimental: bool,
    pub pinned_limit: Option<String>,
    pub cli_path: Option<String>,
    pub credential_path: Option<String>,
    pub manual_limits: Vec<UsageLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub view: String,
    pub theme: String,
    pub opaque: bool,
    pub always_on_top: bool,
    pub startup: bool,
    pub alerts: bool,
    pub interval_secs: u64,
    pub snap_to_edges: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            view: "bars".into(),
            theme: "system".into(),
            opaque: false,
            always_on_top: false,
            startup: false,
            alerts: false,
            interval_secs: 120,
            snap_to_edges: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub schema_version: u32,
    pub settings: Settings,
    pub accounts: Vec<Account>,
    pub cached: BTreeMap<String, Snapshot>,
    pub position: Option<(i32, i32)>,
}
impl Default for Config {
    fn default() -> Self {
        let accounts = [Provider::Claude, Provider::Openai, Provider::Gemini]
            .into_iter()
            .map(|provider| Account {
                revision: 0,
                id: uuid::Uuid::new_v4().to_string(),
                provider,
                label: if provider == Provider::Claude {
                    "Work".into()
                } else {
                    "Personal".into()
                },
                workspace: String::new(),
                connection: if provider == Provider::Openai {
                    Connection::CodexCli
                } else if provider == Provider::Claude {
                    Connection::ClaudeCli
                } else {
                    Connection::Browser
                },
                enabled: false,
                experimental: false,
                pinned_limit: None,
                cli_path: None,
                credential_path: None,
                manual_limits: vec![],
            })
            .collect();
        Self {
            schema_version: 1,
            settings: Settings::default(),
            accounts,
            cached: BTreeMap::new(),
            position: None,
        }
    }
}

pub fn read(path: &Path) -> Config {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
pub fn write(path: &Path, config: &Config) -> Result<(), String> {
    let dir = path.parent().ok_or("Invalid configuration directory.")?;
    std::fs::create_dir_all(dir).map_err(|_| "Cannot create local configuration directory.")?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|_| "Cannot save configuration.")?;
    tmp.write_all(&serde_json::to_vec_pretty(config).map_err(|_| "Cannot encode configuration.")?)
        .map_err(|_| "Cannot write configuration.")?;
    tmp.as_file()
        .sync_all()
        .map_err(|_| "Cannot flush configuration.")?;
    tmp.persist(path)
        .map_err(|_| "Cannot replace configuration.")?;
    Ok(())
}
