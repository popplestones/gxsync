use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::Result;
use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageState {
    pub folder_id: String,
    pub flags: u8,
    pub synced_at: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncState {
    pub messages: HashMap<String, MessageState>, // Keyed by Graph message ID
}

impl SyncState {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::default());
        }

        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let (decoded, _) = decode_from_slice::<Self, _>(&buf, standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(decoded)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let encoded = encode_to_vec(self, standard()).map_err(std::io::Error::other)?;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn is_synced(&self, folder_name: &str, message_id: &str) -> bool {
        match self.messages.get(message_id) {
            Some(state) => state.folder_id == folder_name,
            None => false,
        }
    }

    pub fn mark_synced(&mut self, folder_name: &str, message_id: &str) {
        self.messages.insert(
            message_id.to_string(),
            MessageState {
                folder_id: folder_name.to_string(),
                flags: 0,
                synced_at: Utc::now().timestamp() as u64,
            },
        );
    }

    pub fn insert(&mut self, message_id: &str, state: MessageState) {
        self.messages.insert(message_id.to_string(), state);
    }

    pub fn contains(&self, message_id: &str) -> bool {
        self.messages.contains_key(message_id)
    }
}
