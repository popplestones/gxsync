pub mod auth;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod maildir;
pub mod sync;
pub mod sync_state;

use crate::cli::CliArgs;

pub async fn run(args: CliArgs) -> anyhow::Result<()> {
    Ok(sync::sync_all(args).await?)
}
