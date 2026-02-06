use std::path::{Path, PathBuf};

use crate::{
    cache::CacheState,
    context::{ContextPayload, Handshake},
    health::AuditReport,
    mcp::default_integrations,
    mcp::IntegrationConfig,
    memory::MemoryVault,
    notifications::Notification,
    swarm::SwarmEvent,
    vector::VectorStoreSnapshot,
    watcher::Incident,
};

pub fn cache_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("cache.json"))
}

pub fn save_cache(cache: &CacheState, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cache)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_cache(path: &Path) -> anyhow::Result<CacheState> {
    if !path.exists() {
        return Ok(CacheState::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn memory_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("memory.json"))
}

pub fn handshake_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("handshake.json"))
}

pub fn load_handshake(path: &Path) -> anyhow::Result<Handshake> {
    if !path.exists() {
        return Ok(Handshake {
            root: String::new(),
            generated_at: 0,
            file_count: 0,
            total_bytes: 0,
            digest: String::new(),
        });
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or(Handshake {
        root: String::new(),
        generated_at: 0,
        file_count: 0,
        total_bytes: 0,
        digest: String::new(),
    }))
}

pub fn save_handshake(handshake: &Handshake, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(handshake)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn context_payload_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("context-payload.json"))
}

pub fn save_context_payload(payload: &ContextPayload, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(payload)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn vector_store_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("vector-store.json"))
}

pub fn load_vector_store(path: &Path) -> anyhow::Result<VectorStoreSnapshot> {
    if !path.exists() {
        return Ok(VectorStoreSnapshot::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_vector_store(snapshot: &VectorStoreSnapshot, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_memory(path: &Path) -> anyhow::Result<MemoryVault> {
    MemoryVault::load(path.to_path_buf())
}

pub fn save_memory(vault: &MemoryVault, path: &Path) -> anyhow::Result<()> {
    vault.save(path.to_path_buf())
}

pub fn incidents_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("incidents.json"))
}

pub fn load_incidents(path: &Path) -> anyhow::Result<Vec<Incident>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_incidents(incidents: &[Incident], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(incidents)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn audit_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("audit.json"))
}

pub fn load_audit(path: &Path) -> anyhow::Result<AuditReport> {
    if !path.exists() {
        return Ok(AuditReport::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_audit(report: &AuditReport, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(report)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn kill_switch_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("kill-switch.json"))
}

pub fn load_kill_switch(path: &Path) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or(false))
}

pub fn save_kill_switch(enabled: bool, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(&enabled)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn integrations_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("integrations.json"))
}

pub fn load_integrations(path: &Path) -> anyhow::Result<Vec<IntegrationConfig>> {
    if !path.exists() {
        return Ok(default_integrations());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_else(|_| default_integrations()))
}

pub fn save_integrations(
    integrations: &[IntegrationConfig],
    path: &Path,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(integrations)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn notifications_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("notifications.json"))
}

pub fn load_notifications(path: &Path) -> anyhow::Result<Vec<Notification>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_notifications(notifications: &[Notification], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(notifications)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn swarm_events_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
    Ok(base.join("nexus").join("swarm-events.json"))
}

pub fn load_swarm_events(path: &Path) -> anyhow::Result<Vec<SwarmEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save_swarm_events(events: &[SwarmEvent], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(events)?;
    std::fs::write(path, data)?;
    Ok(())
}
