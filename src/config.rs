use crate::error::GxsyncError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct AccountConfig {
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

#[derive(Debug, Deserialize)]
pub struct GxsyncConfig {
    pub accounts: Vec<AccountConfig>,
}

pub async fn load_config() -> Result<GxsyncConfig, GxsyncError> {
    let path = PathBuf::from("~/.config/gxsync/config.toml");
    let contents = fs::read_to_string(shellexpand::tilde(path.to_str().unwrap()).to_string())?;
    Ok(toml::from_str(&contents)?)
}
