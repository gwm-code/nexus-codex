pub mod cache;
pub mod config;
pub mod daemon;
pub mod desktop;
pub mod health;
pub mod interface;
pub mod mcp;
pub mod memory;
pub mod provider;
pub mod sandbox;
pub mod storage;
pub mod swarm;
pub mod tui;
pub mod watcher;

pub use cache::{CacheDiff, CacheState};
pub use config::Config;
pub use daemon::run_daemon;
pub use interface::{serve as serve_interface, SharedState, StatusSnapshot};
pub use health::AuditReport;
pub use memory::MemoryVault;
pub use mcp::{default_integrations, set_detail, set_enabled, IntegrationConfig, IntegrationKind};
pub use provider::{build_provider, Provider, ProviderConfig, ProviderKind, ProviderSettings};
pub use sandbox::{shadow_run, shadow_run_with_options, ShadowOptions, ShadowResult};
pub use storage::{
    audit_path, cache_path, incidents_path, integrations_path, kill_switch_path, load_audit,
    load_cache, load_incidents, load_integrations, load_kill_switch, load_memory, memory_path,
    save_audit, save_cache, save_incidents, save_integrations, save_kill_switch, save_memory,
};
pub use swarm::{architect_plan, run_workers, Task, TaskResult};
pub use watcher::{analyze_log, watch_filesystem, Incident};
