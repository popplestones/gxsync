use std::path::Path;

use chrono::{Duration, Utc};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;

use crate::cli::CliArgs;
use crate::client::GraphClient;
use crate::config::{AccountConfig, NormalizedAccountConfig, load_config};
use crate::error::GxsyncError;
use crate::maildir;
use crate::sync_state::SyncState;

#[derive(Debug, Deserialize)]
struct MailFolderList {
    value: Vec<MailFolder>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MailFolder {
    _id: String,
    display_name: String,
    total_item_count: u32,
    unread_item_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageList {
    value: Vec<Message>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    id: String,
    subject: Option<String>,
    received_date_time: String,
    internet_message_id: Option<String>,
}

pub async fn sync_all(args: CliArgs) -> Result<(), GxsyncError> {
    let accounts: Vec<NormalizedAccountConfig> = match &args.mailbox {
        Some(mailbox) => {
            let target = args
                .target
                .clone()
                .unwrap_or_else(|| format!("~/Mail/{mailbox}"));
            let days = args.days.unwrap_or(30);
            vec![NormalizedAccountConfig {
                mailbox: mailbox.clone(),
                target,
                days,
                include_folders: args.include_folders.clone(),
                exclude_folders: args.exclude_folders.clone(),
                auth_profile: "default".into(),
            }]
        }
        None => load_config().await?,
    };

    let mp = MultiProgress::new();
    let total_accounts = accounts.len() as u64;
    let master_pb = mp.add(ProgressBar::new(total_accounts));
    master_pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    master_pb.set_message("Syncing accounts");
    for account in accounts {
        let sync_path =
            Path::new(&shellexpand::tilde(&account.target).to_string()).join(".gxsync-state");
        let mut sync_state = SyncState::load(&sync_path)?;
        master_pb.set_message(format!("Syncing account {}", account.mailbox));
        let graph = GraphClient::new(&account.auth_profile).await?;
        let include_set = account.include_folders.as_ref().map(|s| {
            s.split(',')
                .map(|f| f.trim().to_lowercase())
                .collect::<Vec<_>>()
        });
        let exclude_set = account.exclude_folders.as_ref().map(|s| {
            s.split(',')
                .map(|f| f.trim().to_lowercase())
                .collect::<Vec<_>>()
        });
        let url = format!(
            "https://graph.microsoft.com/v1.0/users/{}/mailFolders",
            account.mailbox
        );
        let resp = graph.get(&url).send().await?;
        let folders: MailFolderList = resp.json().await?;

        for folder in folders.value {
            let name = folder.display_name.to_lowercase();

            let included = include_set
                .as_ref()
                .map(|set| set.contains(&name))
                .unwrap_or(true);
            let excluded = exclude_set
                .as_ref()
                .map(|set| set.contains(&name))
                .unwrap_or(false);

            if included && !excluded {
                sync_folder_messages(
                    &graph,
                    &account,
                    &folder,
                    args.dry_run,
                    &mp,
                    &mut sync_state,
                )
                .await?;
            }
        }
        sync_state.save(&sync_path)?;
        println!("Saved sync state to {sync_path:?}");
        master_pb.inc(1);
    }
    master_pb.finish_with_message("All accounts synced.");

    Ok(())
}

async fn sync_folder_messages(
    graph: &GraphClient,
    account: &NormalizedAccountConfig,
    folder: &MailFolder,
    dry_run: bool,
    mp: &MultiProgress,
    sync_state: &mut SyncState,
) -> Result<(), GxsyncError> {
    let since = Utc::now() - Duration::days(account.days as i64);
    let since_iso = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let url = format!(
        "https://graph.microsoft.com/v1.0/users/{}/mailFolders/{}/messages?$filter=receivedDateTime ge {}",
        account.mailbox, folder._id, since_iso
    );
    let resp = graph.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(GxsyncError::Other(format!(
            "Graph query failed: {status}: {body}"
        )));
    }
    let list: MessageList = serde_json::from_str(&body)?;
    let total = list.value.len();
    let pb = mp.add(ProgressBar::new(total as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(folder.display_name.clone());
    for msg in list.value {
        pb.inc(1);

        if sync_state.is_synced(&folder.display_name, &msg.id) {
            continue;
        }

        if dry_run {
            continue;
        }

        let raw_url = format!(
            "https://graph.microsoft.com/v1.0/users/{}/messages/{}/$value",
            account.mailbox, msg.id
        );

        let mime_resp = graph.get_raw(&raw_url).send().await?;
        let mime_data = mime_resp.bytes().await?;

        maildir::write_mail(&account.mailbox, &folder.display_name, &msg.id, &mime_data)?;
        sync_state.mark_synced(&folder.display_name, &msg.id);
    }
    pb.finish_with_message(format!(
        "✅ {:25} / {:30}",
        account.mailbox, folder.display_name,
    ));

    Ok(())
}
