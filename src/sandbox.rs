use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_IMAGE: &str = "ubuntu:22.04";

#[derive(Debug, Clone)]
pub struct ShadowResult {
    pub command: String,
    pub output: String,
    pub status: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ShadowOptions {
    pub root: PathBuf,
    pub image: String,
    pub allow_exec: bool,
    pub hydrate: bool,
}

impl Default for ShadowOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            image: DEFAULT_IMAGE.to_string(),
            allow_exec: false,
            hydrate: false,
        }
    }
}

pub fn shadow_run(command: &str, allow_exec: bool) -> anyhow::Result<ShadowResult> {
    shadow_run_with_options(
        command,
        ShadowOptions {
            allow_exec,
            ..ShadowOptions::default()
        },
    )
}

pub fn shadow_run_with_options(command: &str, options: ShadowOptions) -> anyhow::Result<ShadowResult> {
    if !options.allow_exec {
        return Ok(ShadowResult {
            command: command.to_string(),
            output: "Shadow run only: execution disabled by default.".to_string(),
            status: None,
        });
    }

    if !docker_available() {
        return Ok(ShadowResult {
            command: command.to_string(),
            output: "Docker not available: cannot perform shadow run.".to_string(),
            status: None,
        });
    }

    let temp_root = stage_workspace(&options.root)?;
    let workdir = temp_root.to_string_lossy().to_string();
    let docker_output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/workspace", workdir),
            "-w",
            "/workspace",
            &options.image,
            "bash",
            "-lc",
            command,
        ])
        .output()?;

    let status_code = docker_output.status.code();
    let output = String::from_utf8_lossy(&docker_output.stdout).to_string()
        + &String::from_utf8_lossy(&docker_output.stderr);

    if options.hydrate && status_code == Some(0) {
        hydrate_workspace(&temp_root, &options.root)?;
    }

    Ok(ShadowResult {
        command: command.to_string(),
        output,
        status: status_code,
    })
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn stage_workspace(root: &Path) -> anyhow::Result<PathBuf> {
    let temp_root = std::env::temp_dir().join(format!(
        "nexus-shadow-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));
    std::fs::create_dir_all(&temp_root)?;
    copy_dir_filtered(root, &temp_root)?;
    Ok(temp_root)
}

fn hydrate_workspace(staged: &Path, target: &Path) -> anyhow::Result<()> {
    copy_dir_filtered(staged, target)?;
    Ok(())
}

fn copy_dir_filtered(src: &Path, dest: &Path) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(src).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let rel = match path.strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() || should_skip(rel) {
            continue;
        }
        let target_path = dest.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &target_path)?;
        }
    }
    Ok(())
}

fn should_skip(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            ".git" | "target" | "node_modules" | ".venv"
        )
    })
}
