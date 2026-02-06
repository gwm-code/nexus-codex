use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Incident {
    pub source: String,
    pub summary: String,
    pub detail: Option<String>,
    pub kind: String,
    pub suggestion: Option<String>,
}

impl Default for Incident {
    fn default() -> Self {
        Self {
            source: String::new(),
            summary: String::new(),
            detail: None,
            kind: "error".to_string(),
            suggestion: None,
        }
    }
}

pub fn analyze_log(contents: &str, source: &str) -> Vec<Incident> {
    let mut incidents = Vec::new();
    let error_re = Regex::new(r"(?i)(panic|exception|error|traceback|fatal)").unwrap();
    let stack_start_re = Regex::new(r"(?i)(stack backtrace:|traceback)").unwrap();
    let detail_re = Regex::new(r#"(?i)(at |file "|line \d+)"#).unwrap();
    let mut stack_lines: Vec<String> = Vec::new();
    let mut in_stack = false;

    for line in contents.lines() {
        if stack_start_re.is_match(line) {
            in_stack = true;
            stack_lines.push(line.trim().to_string());
            continue;
        }

        if in_stack {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                let summary = stack_lines
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Stack trace".to_string());
                let detail = Some(stack_lines.join("\n"));
                incidents.push(Incident {
                    source: source.to_string(),
                    summary,
                    detail,
                    kind: "stack-trace".to_string(),
                    suggestion: auto_investigate(&stack_lines.join("\n")),
                });
                stack_lines.clear();
                in_stack = false;
                continue;
            }

            if detail_re.is_match(line)
                || line.starts_with(' ')
                || line.starts_with('\t')
                || error_re.is_match(line)
            {
                stack_lines.push(trimmed.to_string());
                continue;
            }

            let summary = stack_lines
                .first()
                .cloned()
                .unwrap_or_else(|| "Stack trace".to_string());
            let detail = Some(stack_lines.join("\n"));
            incidents.push(Incident {
                source: source.to_string(),
                summary,
                detail,
                kind: "stack-trace".to_string(),
                suggestion: auto_investigate(&stack_lines.join("\n")),
            });
            stack_lines.clear();
            in_stack = false;
        }

        if error_re.is_match(line) {
            let suggestion = auto_investigate(line);
            incidents.push(Incident {
                source: source.to_string(),
                summary: line.trim().to_string(),
                detail: None,
                kind: "error".to_string(),
                suggestion,
            });
        }
    }

    if in_stack && !stack_lines.is_empty() {
        let summary = stack_lines
            .first()
            .cloned()
            .unwrap_or_else(|| "Stack trace".to_string());
        let detail = Some(stack_lines.join("\n"));
        incidents.push(Incident {
            source: source.to_string(),
            summary,
            detail,
            kind: "stack-trace".to_string(),
            suggestion: auto_investigate(&stack_lines.join("\n")),
        });
    }

    incidents
}

pub fn monitor_log(
    path: &Path,
    last_len: &mut u64,
) -> anyhow::Result<Option<Vec<Incident>>> {
    let metadata = std::fs::metadata(path)?;
    let len = metadata.len();
    if len == *last_len {
        return Ok(None);
    }

    *last_len = len;
    let contents = std::fs::read_to_string(path)?;
    Ok(Some(analyze_log(&contents, &path.display().to_string())))
}

pub fn watch_filesystem(root: &Path, tx: Sender<Incident>) -> notify::Result<RecommendedWatcher> {
    let root_display = root.display().to_string();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            for path in event.paths {
                let summary = format!("Filesystem change: {}", path.display());
                let incident = Incident {
                    source: root_display.clone(),
                    summary,
                    detail: Some(format!("Event: {:?}", event.kind)),
                    kind: "fs-change".to_string(),
                    suggestion: None,
                };
                let _ = tx.send(incident);
            }
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok(watcher)
}

fn auto_investigate(context: &str) -> Option<String> {
    let lower = context.to_lowercase();
    if lower.contains("connection refused") || lower.contains("econnrefused") {
        return Some("Check that the dependent service is running and reachable.".to_string());
    }
    if lower.contains("timeout") || lower.contains("timed out") {
        return Some("Inspect network latency or upstream availability.".to_string());
    }
    if lower.contains("permission denied") || lower.contains("eacces") {
        return Some("Verify filesystem permissions and execution rights.".to_string());
    }
    if lower.contains("not found") || lower.contains("no such file") {
        return Some("Confirm the file path or binary exists in the environment.".to_string());
    }
    if lower.contains("panic") {
        return Some("Review recent code changes and add guards around unwraps.".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_errors() {
        let log = "INFO ok\nError: failed to start\npanic at main";
        let incidents = analyze_log(log, "dev.log");
        assert_eq!(incidents.len(), 2);
        assert!(incidents[0].summary.contains("Error"));
    }

    #[test]
    fn detects_stack_traces() {
        let log = "Traceback (most recent call last):\n  File \"app.py\", line 1\nValueError: boom";
        let incidents = analyze_log(log, "app.log");
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].kind, "stack-trace");
        assert!(incidents[0].detail.as_ref().unwrap().contains("ValueError"));
    }
}
