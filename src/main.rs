use clap::{Parser, Subcommand};
use std::io::Read;

use nexus::{
    analyze_log, architect_plan, build_provider, cache::CacheState, memory::MemoryVault,
    serve_interface, shadow_run, shadow_run_with_options, Config, SharedState, StatusSnapshot,
    audit_path, cache_path, incidents_path, integrations_path, kill_switch_path, load_audit,
    load_cache, load_incidents, load_integrations, load_kill_switch, load_notifications,
    load_swarm_events, memory_path, notifications_path, plan_events, result_events, save_audit,
    save_cache, save_incidents, save_integrations, save_kill_switch, save_notifications,
    save_swarm_events, run_daemon, set_detail, set_enabled, swarm_events_path,
};

#[derive(Parser, Debug)]
#[command(name = "nexus", version, about = "Nexus CLI - Phase 1-5 Shell")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show or write configuration
    Config {
        #[arg(long)]
        show_path: bool,
        #[arg(long)]
        write_default: bool,
    },
    /// Intercept commands from stdin and dry-run them
    Run,
    /// Launch the minimal Ratatui status view
    Tui,
    /// Launch the Ratatui diff viewer
    DiffView {
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Dry-run a prompt through the configured provider
    Prompt {
        #[arg(long)]
        input: Option<String>,
        #[arg(long, default_value_t = false)]
        live: bool,
    },
    /// Warm the cache by scanning a repository
    CacheWarm {
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Diff the cache against current disk state
    CacheDiff {
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Manage long-term memory entries
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    /// Shadow-run a command in the sandbox layer
    Sandbox {
        #[arg(long)]
        command: String,
        #[arg(long, default_value_t = false)]
        allow_exec: bool,
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long, default_value = "ubuntu:22.04")]
        image: String,
        #[arg(long, default_value_t = false)]
        hydrate: bool,
    },
    /// Plan or run swarm tasks
    Swarm {
        #[command(subcommand)]
        command: SwarmCommand,
    },
    /// Start the local interface server (Phase 5)
    Serve {
        #[arg(long, default_value = "127.0.0.1:8888")]
        addr: String,
    },
    /// Run headless daemon mode (Phase 5)
    Daemon {
        #[arg(long, default_value = "127.0.0.1:8888")]
        addr: String,
        #[arg(long)]
        log_path: Option<String>,
        #[arg(long, default_value_t = 2000)]
        poll_ms: u64,
        #[arg(long)]
        watch_root: Option<String>,
    },
    /// Scan logs and list incidents
    Heal {
        #[command(subcommand)]
        command: HealCommand,
    },
    /// Capture audit checklist status
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    /// Run lightweight benchmarks
    Bench {
        #[command(subcommand)]
        command: BenchCommand,
    },
    /// Arm or disarm the kill switch
    KillSwitch {
        #[arg(long)]
        on: bool,
        #[arg(long)]
        off: bool,
    },
    /// Manage MCP integrations
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
    /// View notification history
    Notify {
        #[command(subcommand)]
        command: NotifyCommand,
    },
}

#[derive(Subcommand, Debug)]
enum MemoryCommand {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

#[derive(Subcommand, Debug)]
enum SwarmCommand {
    Plan { input: String },
    Run { input: String },
}

#[derive(Subcommand, Debug)]
enum HealCommand {
    Scan { log_path: String },
    List,
}

#[derive(Subcommand, Debug)]
enum AuditCommand {
    Report,
    Mark {
        #[arg(long)]
        performance: bool,
        #[arg(long)]
        security: bool,
        #[arg(long)]
        docs: bool,
    },
}

#[derive(Subcommand, Debug)]
enum BenchCommand {
    Cache { root: String },
}

#[derive(Subcommand, Debug)]
enum McpCommand {
    List,
    Enable { name: String },
    Disable { name: String },
    SetDetail { name: String, key: String, value: String },
}

#[derive(Subcommand, Debug)]
enum NotifyCommand {
    List,
    Clear,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Config {
            show_path,
            write_default,
        } => {
            if show_path {
                println!(
                    "{}",
                    Config::path()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "n/a".to_string())
                );
            }

            if write_default {
                let path = Config::path().ok_or("No config path")?;
                let default = Config::default();
                default.save(&path)?;
                println!("Wrote default config to {}", path.display());
            }

            if !show_path && !write_default {
                println!("{:#?}", config);
            }
        }
        Commands::Run => {
            run_interceptor(&config)?;
        }
        Commands::Tui => {
            nexus::tui::run(&config)?;
        }
        Commands::DiffView { root } => {
            nexus::tui::run_diff(&root)?;
        }
        Commands::Prompt { input, live } => {
            let settings = config.provider_settings();
            let provider = build_provider(&config.provider, settings);
            let prompt = input.unwrap_or_else(|| "Hello Nexus".to_string());
            if config.dry_run && !live {
                println!("{}", provider.dry_run_prompt(&prompt));
            } else {
                println!("{}", provider.send_prompt(&prompt)?);
            }
        }
        Commands::CacheWarm { root } => {
            let mut cache = CacheState::new(root.into());
            cache.warm()?;
            let path = cache_path()?;
            save_cache(&cache, &path)?;
            println!("Cache warmed with {} files.", cache.files.len());
        }
        Commands::CacheDiff { root } => {
            let mut current = CacheState::new(root.into());
            current.warm()?;
            let previous = load_cache(cache_path()?.as_path())?;
            let diff = previous.diff(&current);
            println!("Changed: {}", diff.changed.len());
            for item in diff.changed {
                println!("+ {}", item);
            }
            println!("Removed: {}", diff.removed.len());
            for item in diff.removed {
                println!("- {}", item);
            }
        }
        Commands::Memory { command } => {
            let path = memory_path()?;
            let mut vault = MemoryVault::load(path.clone())?;
            match command {
                MemoryCommand::Set { key, value } => {
                    vault.set(key, value);
                    vault.save(path)?;
                    println!("Memory updated.");
                }
                MemoryCommand::Get { key } => {
                    println!("{}", vault.get(&key).cloned().unwrap_or_default());
                }
                MemoryCommand::List => {
                    for (key, value) in vault.list() {
                        println!("{} = {}", key, value);
                    }
                }
            }
        }
        Commands::Sandbox {
            command,
            allow_exec,
            root,
            image,
            hydrate,
        } => {
            let result = shadow_run_with_options(
                &command,
                nexus::sandbox::ShadowOptions {
                    root: root.into(),
                    image,
                    allow_exec,
                    hydrate,
                },
            )
            .or_else(|_| shadow_run(&command, allow_exec))?;
            println!("{}", result.output);
            if let Some(status) = result.status {
                println!("Exit status: {}", status);
            }
        }
        Commands::Swarm { command } => match command {
            SwarmCommand::Plan { input } => {
                let tasks = architect_plan(&input);
                if let Ok(path) = swarm_events_path() {
                    let mut events = load_swarm_events(&path).unwrap_or_default();
                    events.extend(plan_events(&tasks));
                    let _ = save_swarm_events(&events, &path);
                }
                println!("Planned {} task(s).", tasks.len());
                for task in tasks {
                    println!("[{}] {}", task.id, task.description);
                }
            }
            SwarmCommand::Run { input } => {
                let tasks = architect_plan(&input);
                let results = nexus::run_workers(&tasks);
                if let Ok(path) = swarm_events_path() {
                    let mut events = load_swarm_events(&path).unwrap_or_default();
                    events.extend(result_events(&results));
                    let _ = save_swarm_events(&events, &path);
                }
                for result in results {
                    println!("[{}] {}", result.id, result.summary);
                }
            }
        },
        Commands::Serve { addr } => {
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
            serve_interface(shared, &addr)?;
        }
        Commands::Daemon {
            addr,
            log_path,
            poll_ms,
            watch_root,
        } => {
            run_daemon(
                &config,
                &addr,
                log_path.as_deref(),
                poll_ms,
                watch_root.as_deref(),
            )?;
        }
        Commands::Heal { command } => match command {
            HealCommand::Scan { log_path } => {
                let contents = std::fs::read_to_string(&log_path)?;
                let incidents = analyze_log(&contents, &log_path);
                let path = incidents_path()?;
                save_incidents(&incidents, &path)?;
                println!("Stored {} incident(s).", incidents.len());
            }
            HealCommand::List => {
                let path = incidents_path()?;
                let incidents = load_incidents(&path)?;
                for incident in incidents {
                    let mut line = format!("[{}:{}] {}", incident.source, incident.kind, incident.summary);
                    if let Some(suggestion) = incident.suggestion {
                        line.push_str(&format!(" -> {}", suggestion));
                    }
                    println!("{}", line);
                }
            }
        },
        Commands::Audit { command } => match command {
            AuditCommand::Report => {
                let report = load_audit(&audit_path()?)?;
                println!("{:#?}", report);
            }
            AuditCommand::Mark {
                performance,
                security,
                docs,
            } => {
                let mut report = load_audit(&audit_path()?)?;
                if performance {
                    report.performance_benchmark = true;
                }
                if security {
                    report.security_audit = true;
                }
                if docs {
                    report.docs_complete = true;
                }
                save_audit(&report, &audit_path()?)?;
                println!("Audit updated.");
            }
        },
        Commands::Bench { command } => match command {
            BenchCommand::Cache { root } => {
                let start = std::time::Instant::now();
                let mut cache = CacheState::new(root.into());
                cache.warm()?;
                let elapsed = start.elapsed();
                println!(
                    "Cache warmed: {} files in {:.2?}",
                    cache.files.len(),
                    elapsed
                );
            }
        },
        Commands::KillSwitch { on, off } => {
            let enabled = if on { true } else if off { false } else { true };
            save_kill_switch(enabled, &kill_switch_path()?)?;
            println!(
                "Kill switch {}.",
                if enabled { "armed" } else { "disarmed" }
            );
        }
        Commands::Mcp { command } => match command {
            McpCommand::List => {
                let path = integrations_path()?;
                let integrations = load_integrations(&path)?;
                for integration in integrations {
                    println!(
                        "{} ({:?}) - {}",
                        integration.name,
                        integration.kind,
                        if integration.enabled { "enabled" } else { "disabled" }
                    );
                }
            }
            McpCommand::Enable { name } => {
                let path = integrations_path()?;
                let mut integrations = load_integrations(&path)?;
                if set_enabled(&mut integrations, &name, true) {
                    save_integrations(&integrations, &path)?;
                    println!("Integration enabled.");
                } else {
                    println!("Unknown integration.");
                }
            }
            McpCommand::Disable { name } => {
                let path = integrations_path()?;
                let mut integrations = load_integrations(&path)?;
                if set_enabled(&mut integrations, &name, false) {
                    save_integrations(&integrations, &path)?;
                    println!("Integration disabled.");
                } else {
                    println!("Unknown integration.");
                }
            }
            McpCommand::SetDetail { name, key, value } => {
                let path = integrations_path()?;
                let mut integrations = load_integrations(&path)?;
                if set_detail(&mut integrations, &name, &key, &value) {
                    save_integrations(&integrations, &path)?;
                    println!("Integration detail updated.");
                } else {
                    println!("Unknown integration.");
                }
            }
        },
        Commands::Notify { command } => match command {
            NotifyCommand::List => {
                let path = notifications_path()?;
                let notifications = load_notifications(&path)?;
                for notification in notifications {
                    println!(
                        "[{}] {} - {}",
                        notification.level, notification.source, notification.message
                    );
                }
            }
            NotifyCommand::Clear => {
                let path = notifications_path()?;
                save_notifications(&[], &path)?;
                println!("Notifications cleared.");
            }
        },
    }

    Ok(())
}

fn run_interceptor(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if load_kill_switch(&kill_switch_path()?).unwrap_or(false) {
        println!("Kill switch armed: commands blocked.");
        return Ok(());
    }
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    if buffer.trim().is_empty() {
        println!("No input provided. Pipe commands into `nexus run`.");
        return Ok(());
    }

    let commands: Vec<String> = buffer
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    println!("Intercepted {} command(s).", commands.len());
    for (idx, command) in commands.iter().enumerate() {
        println!("[{}] {}", idx + 1, command);
    }

    if config.dry_run {
        println!("Dry-run mode: commands not executed.");
    } else {
        println!("Execution disabled in Phase 1; set dry_run=true.");
    }

    Ok(())
}
