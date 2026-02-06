use crate::cache::CacheState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub root: String,
    pub generated_at: u64,
    pub file_count: usize,
    pub total_bytes: u64,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub bytes: usize,
    pub truncated: bool,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPayload {
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    pub files: Vec<ContextFile>,
    pub total_bytes: usize,
    pub truncated: bool,
}

pub fn build_handshake(cache: &CacheState) -> Handshake {
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0u64;
    for (path, meta) in &cache.files {
        hasher.update(path.as_bytes());
        hasher.update(meta.hash.as_bytes());
        hasher.update(meta.size.to_string().as_bytes());
        total_bytes = total_bytes.saturating_add(meta.size);
    }

    Handshake {
        root: cache.root.display().to_string(),
        generated_at: now_ts(),
        file_count: cache.files.len(),
        total_bytes,
        digest: hasher.finalize().to_hex().to_string(),
    }
}

pub fn build_payload(
    previous: &CacheState,
    current: &CacheState,
    max_bytes: usize,
) -> anyhow::Result<ContextPayload> {
    let diff = previous.diff(current);
    let mut total_bytes = 0usize;
    let mut truncated = false;
    let mut files = Vec::new();

    for path in &diff.changed {
        if total_bytes >= max_bytes {
            truncated = true;
            break;
        }

        let full_path = current.root.join(path);
        let contents = match std::fs::read_to_string(&full_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let remaining = max_bytes.saturating_sub(total_bytes);
        let mut file_content = contents;
        let mut was_truncated = false;
        if file_content.len() > remaining {
            file_content.truncate(remaining);
            was_truncated = true;
            truncated = true;
        }
        total_bytes += file_content.len();
        files.push(ContextFile {
            path: path.clone(),
            bytes: file_content.len(),
            truncated: was_truncated,
            content: file_content,
        });
    }

    Ok(ContextPayload {
        changed: diff.changed,
        removed: diff.removed,
        files,
        total_bytes,
        truncated,
    })
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl CacheState {
    pub fn diff_payload(
        &self,
        current: &CacheState,
        max_bytes: usize,
    ) -> anyhow::Result<ContextPayload> {
        build_payload(self, current, max_bytes)
    }
}
