use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub value: String,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

impl MemoryEntry {
    pub fn new(value: String, tags: Vec<String>) -> Self {
        Self {
            value,
            updated_at: now_ts(),
            tags,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryVault {
    pub entries: BTreeMap<String, MemoryEntry>,
}

impl MemoryVault {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        if let Ok(vault) = serde_json::from_str::<Self>(&raw) {
            return Ok(vault);
        }
        if let Ok(entries) = serde_json::from_str::<BTreeMap<String, String>>(&raw) {
            let migrated = entries
                .into_iter()
                .map(|(key, value)| (key, MemoryEntry::new(value, Vec::new())))
                .collect();
            return Ok(Self { entries: migrated });
        }
        Ok(Self::default())
    }

    pub fn save(&self, path: PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) {
        self.set_with_tags(key, value, Vec::new());
    }

    pub fn set_with_tags(&mut self, key: String, value: String, tags: Vec<String>) {
        self.entries.insert(key, MemoryEntry::new(value, tags));
    }

    pub fn get(&self, key: &str) -> Option<&MemoryEntry> {
        self.entries.get(key)
    }

    pub fn list(&self) -> Vec<(String, MemoryEntry)> {
        self.entries
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
