use std::path::{Path, PathBuf};

use crate::{
    cache::CacheState, health::AuditReport, mcp::default_integrations, mcp::IntegrationConfig,
    memory::MemoryVault, watcher::Incident,
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
