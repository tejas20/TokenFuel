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
    OpencodeGo,
    CursorLocal,
    GrokCli,
    CopilotCli,
    AntigravityLocal,
    #[serde(other)]
    Unknown,
}

impl Connection {
    pub fn requires_experimental_opt_in(self) -> bool {
        matches!(
            self,
            Self::ClaudeCli
                | Self::Browser
                | Self::GeminiWeb
                | Self::OpencodeGo
                | Self::CursorLocal
                | Self::GrokCli
                | Self::AntigravityLocal
        )
    }

    pub fn supports_custom_secret(self) -> bool {
        matches!(
            self,
            Self::OpencodeGo
                | Self::CursorLocal
                | Self::GrokCli
                | Self::CopilotCli
                | Self::AntigravityLocal
        )
    }
}

#[cfg(test)]
mod connection_tests {
    use super::Connection;

    #[test]
    fn experimental_connections_require_separate_consent() {
        for connection in [
            Connection::ClaudeCli,
            Connection::Browser,
            Connection::GeminiWeb,
            Connection::OpencodeGo,
            Connection::CursorLocal,
            Connection::GrokCli,
            Connection::AntigravityLocal,
        ] {
            assert!(connection.requires_experimental_opt_in());
        }
        for connection in [
            Connection::CodexCli,
            Connection::CopilotCli,
            Connection::Manual,
        ] {
            assert!(!connection.requires_experimental_opt_in());
        }
    }
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
    #[serde(default)]
    pub has_custom_secret: bool,
    pub manual_limits: Vec<UsageLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub focus_account_id: Option<String>,
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
            focus_account_id: None,
            view: "bars".into(),
            theme: "system".into(),
            opaque: false,
            always_on_top: true,
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
    pub discovered_providers: Vec<Provider>,
    #[serde(skip_deserializing)]
    pub settings_issues: Vec<String>,
    pub schema_version: u32,
    pub settings: Settings,
    pub accounts: Vec<Account>,
    pub cached: BTreeMap<String, Snapshot>,
    pub position: Option<(i32, i32)>,
}
impl Default for Config {
    fn default() -> Self {
        let accounts = [
            Provider::Openai,
            Provider::Claude,
            Provider::Gemini,
            Provider::Cursor,
            Provider::Copilot,
            Provider::Grok,
            Provider::Opencode,
            Provider::Antigravity,
        ]
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
                match provider {
                    Provider::Cursor => Connection::CursorLocal,
                    Provider::Copilot => Connection::CopilotCli,
                    Provider::Grok => Connection::GrokCli,
                    Provider::Opencode => Connection::OpencodeGo,
                    Provider::Antigravity => Connection::AntigravityLocal,
                    _ => Connection::GeminiWeb,
                }
            },
            enabled: false,
            experimental: false,
            pinned_limit: None,
            cli_path: None,
            credential_path: None,
            manual_limits: vec![],
            has_custom_secret: false,
        })
        .collect();
        Self {
            discovered_providers: vec![],
            settings_issues: vec![],
            schema_version: 1,
            settings: Settings::default(),
            accounts,
            cached: BTreeMap::new(),
            position: None,
        }
    }
}

pub fn read(path: &Path) -> Result<Config, String> {
    let mut config: Config = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "Configuration is invalid. Restore config.json from a backup; your preferences have not been overwritten.".to_string())?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
        Err(_) => return Err("Cannot read saved configuration; your preferences have not been overwritten.".into()),
    };
    if config.schema_version < 2 {
        config.schema_version = 2;
        write(path, &config)?;
    }
    Ok(config)
}

pub fn patch_settings(current: &Settings, patch: serde_json::Value) -> Result<Settings, String> {
    let mut value = serde_json::to_value(current).map_err(|_| "Cannot encode preferences.")?;
    let fields = patch.as_object().ok_or("Invalid preferences.")?;
    for (key, value_patch) in fields {
        if value.get(key).is_none() {
            return Err("Unknown preference.".into());
        }
        value[key] = value_patch.clone();
    }
    serde_json::from_value(value).map_err(|_| "Invalid preference value.".into())
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

#[cfg(test)]
mod persistence_tests {
    use super::*;

    #[test]
    fn independent_preferences_survive_replacement_and_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut config = Config::default();
        write(&path, &config).unwrap();
        for patch in [
            serde_json::json!({"startup": true}),
            serde_json::json!({"alwaysOnTop": true}),
            serde_json::json!({"theme": "light"}),
            serde_json::json!({"focusAccountId": "personal"}),
        ] {
            config.settings = patch_settings(&config.settings, patch).unwrap();
            write(&path, &config).unwrap();
            config = read(&path).unwrap();
        }
        assert!(config.settings.startup);
        assert!(config.settings.always_on_top);
        assert_eq!(config.settings.theme, "light");
        assert_eq!(
            config.settings.focus_account_id.as_deref(),
            Some("personal")
        );
        config.settings = patch_settings(
            &config.settings,
            serde_json::json!({"focusAccountId": null}),
        )
        .unwrap();
        write(&path, &config).unwrap();
        assert!(read(&path).unwrap().settings.focus_account_id.is_none());
    }

    #[test]
    fn invalid_config_is_not_silently_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, b"broken").unwrap();
        assert!(read(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"broken");
    }

    #[test]
    fn old_preferences_are_preserved_and_invalid_patches_rejected() {
        let config: Config = serde_json::from_value(
            serde_json::json!({"settings":{"alwaysOnTop":false,"startup":true}}),
        )
        .unwrap();
        assert!(!config.settings.always_on_top);
        assert!(config.settings.startup);
        assert!(config.settings.focus_account_id.is_none());
        assert!(patch_settings(&config.settings, serde_json::json!({"startup":"yes"})).is_err());
        assert!(patch_settings(&config.settings, serde_json::json!({"typo":true})).is_err());
    }
}
