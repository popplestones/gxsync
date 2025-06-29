use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "gxsync",
    about = "Synchronize Microsoft 365 mailboxes via Graph API"
)]
pub struct CliArgs {
    /// Email address of the mailbox to sync (Shared or User)
    #[arg(long)]
    pub mailbox: Option<String>,

    #[arg(long)]
    pub target: Option<String>,

    #[arg(long)]
    pub days: Option<u32>,

    /// Comma-separated list of folders to include (optional)
    #[arg(long)]
    pub include_folders: Option<String>,

    /// Comma-separated list of folders to exclude (optional)
    #[arg(long)]
    pub exclude_folders: Option<String>,

    /// Don't write anything to disk
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}
