use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u64,
    pub timestamp: u64,
    pub level: String,
    pub source: String,
    pub message: String,
}

pub fn new_notification(level: &str, source: &str, message: &str) -> Notification {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Notification {
        id: timestamp,
        timestamp,
        level: level.to_string(),
        source: source.to_string(),
        message: message.to_string(),
    }
}
