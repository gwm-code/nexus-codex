use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui;

use crate::{
    cache::CacheState,
    interface::{serve, SharedState, StatusSnapshot},
    mcp::IntegrationConfig,
    storage::{
        audit_path, cache_path, incidents_path, integrations_path, kill_switch_path, load_audit,
        load_cache, load_incidents, load_integrations, load_kill_switch, load_memory,
        memory_path, save_audit, save_cache, save_incidents, save_integrations,
        save_kill_switch, save_memory,
    },
    watcher::analyze_log,
    Config,
};

#[derive(Clone)]
pub struct DesktopState {
    pub status: StatusSnapshot,
    pub log: Vec<String>,
    pub memory_entries: Vec<(String, String)>,
    pub server_running: bool,
    pub memory_key: String,
    pub memory_value: String,
    pub cache_root: String,
    pub server_addr: String,
    pub incident_log_path: String,
    pub incidents: Vec<String>,
    pub audit_performance: bool,
    pub audit_security: bool,
    pub audit_docs: bool,
    pub kill_switch: bool,
    pub integrations: Vec<IntegrationConfig>,
}

pub struct DesktopApp {
    state: Arc<Mutex<DesktopState>>,
}

impl DesktopApp {
    pub fn new() -> Self {
        let config = Config::load();
        let kill_switch = kill_switch_path()
            .ok()
            .and_then(|path| load_kill_switch(&path).ok())
            .unwrap_or(false);
        let status = StatusSnapshot {
            provider: config.provider,
            dry_run: config.dry_run,
            cache_entries: 0,
            memory_entries: 0,
            kill_switch,
        };
        Self {
            state: Arc::new(Mutex::new(DesktopState {
                status,
                log: vec!["Nexus Desktop ready.".to_string()],
                memory_entries: Vec::new(),
                server_running: false,
                memory_key: String::new(),
                memory_value: String::new(),
                cache_root: ".".to_string(),
                server_addr: "127.0.0.1:8888".to_string(),
                incident_log_path: "dev.log".to_string(),
                incidents: Vec::new(),
                audit_performance: false,
                audit_security: false,
                audit_docs: false,
                kill_switch,
                integrations: Vec::new(),
            })),
        }
    }

    fn refresh(&self) {
        let cache = cache_path()
            .ok()
            .and_then(|path| load_cache(&path).ok())
            .unwrap_or_default();
        let memory = memory_path()
            .ok()
            .and_then(|path| load_memory(&path).ok())
            .unwrap_or_default();

        if let Ok(mut state) = self.state.lock() {
            state.status.cache_entries = cache.files.len();
            state.status.memory_entries = memory.entries.len();
            state.memory_entries = memory
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            if let Ok(path) = incidents_path() {
                state.incidents = load_incidents(&path)
                    .unwrap_or_default()
                    .iter()
                    .map(|incident| {
                        format!(
                            "[{}] {}",
                            incident.kind,
                            incident.summary
                        )
                    })
                    .collect();
            }
            if let Ok(path) = audit_path() {
                let report = load_audit(&path).unwrap_or_default();
                state.audit_performance = report.performance_benchmark;
                state.audit_security = report.security_audit;
                state.audit_docs = report.docs_complete;
            }
            if let Ok(path) = kill_switch_path() {
                let enabled = load_kill_switch(&path).unwrap_or(false);
                state.kill_switch = enabled;
                state.status.kill_switch = enabled;
            }
            if let Ok(path) = integrations_path() {
                state.integrations = load_integrations(&path).unwrap_or_default();
            }
            state.log.push("Status refreshed.".to_string());
        }
    }

    fn warm_cache(&self, root: &str) {
        let mut cache = CacheState::new(root.into());
        let result = cache.warm().and_then(|_| {
            let path = cache_path()?;
            save_cache(&cache, &path)?;
            Ok(())
        });

        if let Ok(mut state) = self.state.lock() {
            match result {
                Ok(_) => state.log.push(format!("Cache warmed: {} files.", cache.files.len())),
                Err(err) => state.log.push(format!("Cache warm failed: {}", err)),
            }
        }
        self.refresh();
    }

    fn add_memory(&self, key: String, value: String) {
        let path = match memory_path() {
            Ok(path) => path,
            Err(err) => {
                self.push_log(format!("Memory path error: {}", err));
                return;
            }
        };

        let mut vault = load_memory(&path).unwrap_or_default();
        vault.set(key, value);
        if let Err(err) = save_memory(&vault, &path) {
            self.push_log(format!("Memory save failed: {}", err));
        } else {
            self.push_log("Memory updated.".to_string());
        }
        self.refresh();
    }

    fn scan_incidents(&self, log_path: &str) {
        let contents = match std::fs::read_to_string(log_path) {
            Ok(contents) => contents,
            Err(err) => {
                self.push_log(format!("Log scan failed: {}", err));
                return;
            }
        };
        let incidents = analyze_log(&contents, log_path);
        let path = match incidents_path() {
            Ok(path) => path,
            Err(err) => {
                self.push_log(format!("Incident path error: {}", err));
                return;
            }
        };
        if let Err(err) = save_incidents(&incidents, &path) {
            self.push_log(format!("Incident save failed: {}", err));
        } else {
            self.push_log(format!("Incidents stored: {}", incidents.len()));
        }
        self.refresh();
    }

    fn save_audit(&self, performance: bool, security: bool, docs: bool) {
        let report = crate::health::AuditReport {
            performance_benchmark: performance,
            security_audit: security,
            docs_complete: docs,
        };
        let path = match audit_path() {
            Ok(path) => path,
            Err(err) => {
                self.push_log(format!("Audit path error: {}", err));
                return;
            }
        };
        if let Err(err) = save_audit(&report, &path) {
            self.push_log(format!("Audit save failed: {}", err));
        } else {
            self.push_log("Audit updated.".to_string());
        }
        self.refresh();
    }

    fn save_integrations(&self, integrations: Vec<IntegrationConfig>) {
        let path = match integrations_path() {
            Ok(path) => path,
            Err(err) => {
                self.push_log(format!("Integrations path error: {}", err));
                return;
            }
        };
        if let Err(err) = save_integrations(&integrations, &path) {
            self.push_log(format!("Integrations save failed: {}", err));
        } else {
            self.push_log("Integrations updated.".to_string());
        }
        self.refresh();
    }

    fn toggle_kill_switch(&self, enabled: bool) {
        let path = match kill_switch_path() {
            Ok(path) => path,
            Err(err) => {
                self.push_log(format!("Kill switch path error: {}", err));
                return;
            }
        };
        if let Err(err) = save_kill_switch(enabled, &path) {
            self.push_log(format!("Kill switch update failed: {}", err));
        } else {
            self.push_log(format!(
                "Kill switch {}.",
                if enabled { "armed" } else { "disarmed" }
            ));
        }
        self.refresh();
    }

    fn start_server(&self, addr: String) {
        let addr_log = addr.clone();
        let shared_state = {
            let state = self.state.lock().unwrap();
            SharedState::new(state.status.clone())
        };
        let state_handle = self.state.clone();

        thread::spawn(move || {
            if let Err(err) = serve(shared_state, &addr) {
                if let Ok(mut state) = state_handle.lock() {
                    state.log.push(format!("Interface error: {}", err));
                    state.server_running = false;
                }
            }
        });

        if let Ok(mut state) = self.state.lock() {
            state.server_running = true;
            state.log.push(format!("Interface server starting on {}", addr_log));
        }
    }

    fn push_log(&self, message: String) {
        if let Ok(mut state) = self.state.lock() {
            state.log.push(message);
        }
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut start_server: Option<String> = None;
        let mut warm_cache: Option<String> = None;
        let mut add_memory: Option<(String, String)> = None;
        let mut scan_incidents: Option<String> = None;
        let mut update_audit: Option<(bool, bool, bool)> = None;
        let mut update_kill_switch: Option<bool> = None;
        let mut update_integrations: Option<Vec<IntegrationConfig>> = None;
        let mut refresh = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nexus Desktop");
            ui.label("Safety-first AI agent orchestrator");
            ui.separator();

            let mut state_snapshot = self.state.lock().unwrap();

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.label(format!("Provider: {:?}", state_snapshot.status.provider));
                    ui.label(format!("Dry run: {}", state_snapshot.status.dry_run));
                    ui.label(format!("Cache entries: {}", state_snapshot.status.cache_entries));
                    ui.label(format!("Memory entries: {}", state_snapshot.status.memory_entries));
                    ui.label(format!("Kill switch: {}", state_snapshot.status.kill_switch));
                });

                ui.group(|ui| {
                    if ui.button("Refresh status").clicked() {
                        refresh = true;
                    }
                    ui.add_space(8.0);
                    ui.label("Interface server");
                    ui.text_edit_singleline(&mut state_snapshot.server_addr);
                    if ui
                        .add_enabled(!state_snapshot.server_running, egui::Button::new("Start"))
                        .clicked()
                    {
                        start_server = Some(state_snapshot.server_addr.clone());
                    }
                });
            });

            ui.separator();
            ui.heading("Cache");
            ui.horizontal(|ui| {
                ui.label("Root");
                ui.text_edit_singleline(&mut state_snapshot.cache_root);
                if ui.button("Warm cache").clicked() {
                    warm_cache = Some(state_snapshot.cache_root.clone());
                }
            });

            ui.separator();
            ui.heading("Memory vault");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state_snapshot.memory_key);
                ui.text_edit_singleline(&mut state_snapshot.memory_value);
                if ui.button("Save preference").clicked() && !state_snapshot.memory_key.is_empty() {
                    add_memory = Some((
                        state_snapshot.memory_key.clone(),
                        state_snapshot.memory_value.clone(),
                    ));
                }
            });

            ui.collapsing("Stored preferences", |ui| {
                for (key, value) in state_snapshot.memory_entries.iter() {
                    ui.label(format!("{} = {}", key, value));
                }
            });

            ui.separator();
            ui.heading("Self-healing watcher");
            ui.horizontal(|ui| {
                ui.label("Log path");
                ui.text_edit_singleline(&mut state_snapshot.incident_log_path);
                if ui.button("Scan").clicked() {
                    scan_incidents = Some(state_snapshot.incident_log_path.clone());
                }
            });
            ui.collapsing("Detected incidents", |ui| {
                for incident in state_snapshot.incidents.iter() {
                    ui.label(incident);
                }
            });

            ui.separator();
            ui.heading("Safety controls");
            ui.horizontal(|ui| {
                ui.checkbox(&mut state_snapshot.kill_switch, "Arm kill switch");
                if ui.button("Apply").clicked() {
                    update_kill_switch = Some(state_snapshot.kill_switch);
                }
            });

            ui.separator();
            ui.heading("Audit checklist");
            ui.horizontal(|ui| {
                ui.checkbox(&mut state_snapshot.audit_performance, "Performance benchmark");
                ui.checkbox(&mut state_snapshot.audit_security, "Security audit");
                ui.checkbox(&mut state_snapshot.audit_docs, "Docs complete");
                if ui.button("Save audit").clicked() {
                    update_audit = Some((
                        state_snapshot.audit_performance,
                        state_snapshot.audit_security,
                        state_snapshot.audit_docs,
                    ));
                }
            });

            ui.separator();
            ui.heading("Integrations");
            ui.label("Enable MCP integrations and adapters.");
            for integration in state_snapshot.integrations.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut integration.enabled, &integration.name);
                    ui.label(format!("{:?}", integration.kind));
                });
            }
            if ui.button("Save integrations").clicked() {
                update_integrations = Some(state_snapshot.integrations.clone());
            }

            ui.separator();
            ui.heading("Activity log");
            for entry in state_snapshot.log.iter().rev().take(8) {
                ui.label(entry);
            }

        });

        if let Some(addr) = start_server {
            self.start_server(addr);
        }
        if let Some(root) = warm_cache {
            self.warm_cache(&root);
        }
        if let Some((key, value)) = add_memory {
            self.add_memory(key, value);
        }
        if let Some(log_path) = scan_incidents {
            self.scan_incidents(&log_path);
        }
        if let Some((performance, security, docs)) = update_audit {
            self.save_audit(performance, security, docs);
        }
        if let Some(enabled) = update_kill_switch {
            self.toggle_kill_switch(enabled);
        }
        if let Some(integrations) = update_integrations {
            self.save_integrations(integrations);
        }
        if refresh {
            self.refresh();
        }
    }
}
