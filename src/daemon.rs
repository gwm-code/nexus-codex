use std::{path::Path, sync::mpsc, thread, time::Duration};

use crate::{
    interface::{serve, SharedState, StatusSnapshot},
    memory::MemoryVault,
    storage::{
        cache_path, incidents_path, kill_switch_path, load_cache, load_incidents,
        load_kill_switch, memory_path, save_incidents,
    },
    watcher::{monitor_log, watch_filesystem},
    Config,
};

pub fn run_daemon(
    config: &Config,
    addr: &str,
    log_path: Option<&str>,
    poll_ms: u64,
    watch_root: Option<&str>,
) -> anyhow::Result<()> {
    let cache = load_cache(cache_path()?.as_path()).unwrap_or_default();
    let memory = MemoryVault::load(memory_path()?).unwrap_or_default();
    let kill_switch = load_kill_switch(&kill_switch_path()?).unwrap_or(false);

    let snapshot = StatusSnapshot {
        provider: config.provider.clone(),
        dry_run: config.dry_run,
        cache_entries: cache.files.len(),
        memory_entries: memory.entries.len(),
        kill_switch,
    };
    let shared = SharedState::new(snapshot);
    shared.update(&cache, &memory);

    if let Some(path) = log_path {
        let path = Path::new(path).to_path_buf();
        thread::spawn(move || {
            let mut last_len = 0;
            loop {
                if let Ok(Some(incidents)) = monitor_log(&path, &mut last_len) {
                    if let Ok(existing_path) = incidents_path() {
                        let mut existing = load_incidents(&existing_path).unwrap_or_default();
                        for incident in incidents {
                            let already = existing.iter().any(|item| {
                                item.summary == incident.summary && item.kind == incident.kind
                            });
                            if !already {
                                existing.push(incident);
                            }
                        }
                        let _ = save_incidents(&existing, &existing_path);
                    }
                }
                thread::sleep(Duration::from_millis(poll_ms));
            }
        });
    }

    if let Some(root) = watch_root {
        let root = Path::new(root).to_path_buf();
        thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let _watcher = watch_filesystem(&root, tx);
            while let Ok(incident) = rx.recv() {
                if let Ok(existing_path) = incidents_path() {
                    let mut existing = load_incidents(&existing_path).unwrap_or_default();
                    existing.push(incident);
                    let _ = save_incidents(&existing, &existing_path);
                }
            }
        });
    }

    serve(shared, addr)
}
