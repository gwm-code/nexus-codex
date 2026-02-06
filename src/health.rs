use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub performance_benchmark: bool,
    pub security_audit: bool,
    pub docs_complete: bool,
}

impl Default for AuditReport {
    fn default() -> Self {
        Self {
            performance_benchmark: false,
            security_audit: false,
            docs_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub path: String,
    pub issue: String,
}

pub fn run_security_audit(root: &Path) -> anyhow::Result<Vec<SecurityFinding>> {
    let mut findings = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if matches!(name, ".env" | ".env.local" | "id_rsa" | "id_ed25519") || name.ends_with(".pem")
        {
            findings.push(SecurityFinding {
                path: path.display().to_string(),
                issue: "Sensitive file detected".to_string(),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                let mode = meta.permissions().mode();
                if mode & 0o002 != 0 {
                    findings.push(SecurityFinding {
                        path: path.display().to_string(),
                        issue: "World-writable file".to_string(),
                    });
                }
            }
        }
    }
    Ok(findings)
}
