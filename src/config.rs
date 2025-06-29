use crate::error::GxsyncError;
use serde::Deserialize;
use serde::de::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub mailbox: String,
    pub target: Option<String>,
    pub days: Option<u32>,
    pub include_folders: Option<String>,
    pub exclude_folders: Option<String>,
    #[serde(default = "default_profile")]
    pub auth_profile: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NormalizedAccountConfig {
    pub mailbox: String,
    pub target: String,
    pub days: u32,
    pub include_folders: Option<String>,
    pub exclude_folders: Option<String>,
    #[serde(default = "default_profile")]
    pub auth_profile: String,
}
fn default_profile() -> String {
    "default".to_string()
}
impl From<AccountConfig> for NormalizedAccountConfig {
    fn from(config: AccountConfig) -> Self {
        let target = config
            .target
            .unwrap_or_else(|| format!("~/Mail/{}", config.mailbox));

        let days = config.days.unwrap_or(30);
        Self {
            mailbox: config.mailbox,
            target,
            days,
            include_folders: config.include_folders,
            exclude_folders: config.exclude_folders,
            auth_profile: config.auth_profile,
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct GxsyncConfig {
    pub accounts: Vec<AccountConfig>,
}

pub async fn load_config() -> Result<Vec<NormalizedAccountConfig>, GxsyncError> {
    let raw_path = "~/.config/gxsync/config.toml";
    let expanded_path = shellexpand::tilde(raw_path).to_string();
    let path = PathBuf::from(&expanded_path.clone());

    let contents = fs::read_to_string(&path)
        .map_err(|e| std::io::Error::new(e.kind(), format!("{expanded_path}: {e}")))?;

    let config: GxsyncConfig = toml::from_str(&contents)
        .map_err(|e| toml::de::Error::custom(format!("{expanded_path}: {e}")))?;

    Ok(config.accounts.into_iter().map(Into::into).collect())
}
