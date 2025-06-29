use chrono::{Duration, Utc};
use serde::Deserialize;

use crate::cli::CliArgs;
use crate::client::GraphClient;
use crate::config::{AccountConfig, load_config};
use crate::error::GxsyncError;
use crate::maildir;

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
    let accounts: Vec<AccountConfig> = match &args.mailbox {
        Some(mailbox) => vec![AccountConfig {
            mailbox: mailbox.clone(),
            target: args
                .target
                .clone()
                .unwrap_or_else(|| format!("~/Mail/{mailbox}")),
            days: args.days.unwrap_or(30),
            include_folders: args.include_folders.clone(),
            exclude_folders: args.exclude_folders.clone(),
            auth_profile: "default".into(),
        }],
        None => load_config().await?.accounts,
    };

    for account in accounts {
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
                tracing::info!(
                    "Found folder: {} ({} items, {} unread)",
                    folder.display_name,
                    folder.total_item_count,
                    folder.unread_item_count
                );
                sync_folder_messages(&graph, &account, &folder, args.dry_run).await?;
            }
        }
    }

    Ok(())
}

async fn sync_folder_messages(
    graph: &GraphClient,
    account: &AccountConfig,
    folder: &MailFolder,
    dry_run: bool,
) -> Result<(), GxsyncError> {
    let since = Utc::now() - Duration::days(account.days as i64);
    let since_iso = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let url = format!(
        "https://graph.microsoft.com/v1.0/users/{}/mailFolders/{}/messages?$filter=receivedDateTime ge {}",
        account.mailbox, folder._id, since_iso
    );
    tracing::info!(
        "Fetching messages for `{}` since {}",
        folder.display_name,
        since_iso
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
    for msg in list.value {
        tracing::info!(
            "Message: [{}] {} @ {}",
            msg.internet_message_id.as_deref().unwrap_or("<no id>"),
            msg.subject.as_deref().unwrap_or("<no subject>"),
            msg.received_date_time
        );
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
    }

    tracing::debug!("{} response: {}", folder.display_name, body);

    Ok(())
}
