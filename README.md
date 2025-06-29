# gxsync

`gxsync` is a command-line tool for downloading email from Microsoft 365 mailboxes using the Graph API. It stores messages in local Maildir format and supports syncing multiple accounts and folders, making it suitable for use with email clients like `neomutt` or `aerc`.

## Features

- Supports syncing both primary and shared Microsoft 365 mailboxes
- Downloads messages from selected folders within a configurable date range
- Stores messages in Maildir format for compatibility with clients like `neomutt`
- Reads configuration from a TOML file to sync multiple accounts in a single run
- Integrates with `msoauth` for token management via device login flow

## Installation

You can install both `gxsync` and its companion token management tool `msoauth` using `cargo`:

```sh
cargo install gxsync
cargo install msoauth
```

Make sure `~/.cargo/bin` is in your `$PATH` so you can run `gxsync` from anywhere.

### Token Management

`gxsync` relies on `msoauth` to handle authentication with Microsoft 365 via the OAuth 2.0 device login flow. You must install and configure `msoauth` before using `gxsync`.
See the [msoauth README](https://github.com/popplestones/msoauth) for setup instructions.

## Configuration

Create a TOML config at `~/.config/gxsync/config.toml`:

```toml
[[accounts]]
mailbox = "support@example.com"
target = "~/Mail/support"
days = 30
auth_profile = "support"
include_folders = "Inbox,Helpdesk Tickets"
exclude_folders = "Junk Email,Deleted Items"

[[accounts]]
mailbox = "admin@example.com"
target = "~/Mail/admin"
days = 60
auth_profile = "admin"
```

Each auth_profile must have a corresponding [profile] section in ~/.config/msoauth/config.toml used by msoauth:

```toml
[admin]
client_id = "..."
client_secret = "..."
tenant_id = "..."
scope = "https://graph.microsoft.com/.default"

[support]
client_id = "..."
client_secret = "..."
tenant_id = "..."
scope = "https://graph.microsoft.com/.default"
```

## Usage

```sh
gxsync
```
By default, gxsync reads from ~/.config/gxsync/config.toml and synchronizes all configured accounts.

You can override configuration using CLI arguments:

```sh
gxsync --mailbox user@example.com --days 10 --target ~/Mail/user
```

Supported flags:
- `--mailbox <address>` – override mailbox address
- `--target <dir>` – override destination maildir path
- `--days <N>` – number of days to sync from (default: 30)
- `--dry-run` – skip writing messages to disk
- `--include-folders <list>` – only sync matching folder names (comma-separated)
- `--exclude-folders <list>` – skip folders matching names (comma-separated)

## Runnning with systemd

To automate sync, you can run `gxsync` periodically using `systemd`.

### Example timer

Create the following unit files:

`~/.config/systemd/user/gxsync.service`
```ini
[Unit]
Description=Sync Microsoft 365 mailboxes with gxsync

[Service]
Type=oneshot
ExecStart=%h/.cargo/bin/gxsync
```

`~/.config/systemd/user/gxsync.timer`
```ini
[Unit]
Description=Run gxsync every 15 minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=15min
OnCalendar=daily
Unit=gxsync.service

[Install]
WantedBy=timers.target
```

Enable and start the timer:

```sh
systemctl --user enable --now gxsync.timer
```
Check logs with:
```sh
journalctl --user -u gxsync.service
```

## License

This project is licensed under the MIT license.
