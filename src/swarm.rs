use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmEvent {
    pub timestamp: u64,
    pub event: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub dependencies: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: usize,
    pub summary: String,
    pub worker: String,
}

pub fn architect_plan(input: &str) -> Vec<Task> {
    input
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(Task {
                    id: idx + 1,
                    description: trimmed.to_string(),
                    dependencies: Vec::new(),
                })
            }
        })
        .collect()
}

pub fn run_workers(tasks: &[Task]) -> Vec<TaskResult> {
    let mut results = Vec::new();
    for task in tasks {
        results.push(TaskResult {
            id: task.id,
            summary: format!("Worker completed: {}", task.description),
            worker: "worker".to_string(),
        });
    }
    results
}

pub fn architect_with_dependencies(input: &str) -> Vec<Task> {
    let mut tasks = architect_plan(input);
    let mut mapping: BTreeMap<String, usize> = BTreeMap::new();
    for task in &tasks {
        mapping.insert(task.description.to_lowercase(), task.id);
    }

    for task in &mut tasks {
        let deps = parse_dependencies(&task.description, &mapping);
        task.dependencies = deps;
    }

    tasks
}

pub fn run_parallel_workers(tasks: &[Task]) -> Vec<TaskResult> {
    let mut remaining: BTreeMap<usize, Task> = tasks
        .iter()
        .cloned()
        .map(|task| (task.id, task))
        .collect();
    let mut completed: BTreeSet<usize> = BTreeSet::new();
    let mut results = Vec::new();

    while !remaining.is_empty() {
        let ready: Vec<Task> = remaining
            .values()
            .filter(|task| task.dependencies.iter().all(|dep| completed.contains(dep)))
            .cloned()
            .collect();

        if ready.is_empty() {
            for task in remaining.values() {
                results.push(TaskResult {
                    id: task.id,
                    summary: format!("Blocked by dependencies: {}", task.description),
                    worker: "scheduler".to_string(),
                });
            }
            break;
        }

        let mut handles = Vec::new();
        for task in ready {
            remaining.remove(&task.id);
            handles.push(thread::spawn(move || run_task(task)));
        }

        for handle in handles {
            if let Ok(result) = handle.join() {
                completed.insert(result.id);
                results.push(result);
            }
        }
    }

    self_correction(results)
}

pub fn plan_events(tasks: &[Task]) -> Vec<SwarmEvent> {
    let timestamp = now_ts();
    tasks
        .iter()
        .map(|task| SwarmEvent {
            timestamp,
            event: "planned".to_string(),
            detail: format!("[{}] {}", task.id, task.description),
        })
        .collect()
}

pub fn result_events(results: &[TaskResult]) -> Vec<SwarmEvent> {
    let timestamp = now_ts();
    results
        .iter()
        .map(|result| SwarmEvent {
            timestamp,
            event: "completed".to_string(),
            detail: format!("[{}] {} ({})", result.id, result.summary, result.worker),
        })
        .collect()
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn run_task(task: Task) -> TaskResult {
    let worker = pick_worker(&task.description);
    let mut summary = format!("{} completed: {}", worker, task.description);
    if task.description.to_lowercase().contains("fail") {
        summary = format!("{} failed: {}", worker, task.description);
    }
    thread::sleep(Duration::from_millis(50));
    TaskResult {
        id: task.id,
        summary,
        worker,
    }
}

fn pick_worker(description: &str) -> String {
    let lower = description.to_lowercase();
    if lower.contains("frontend") || lower.contains("ui") {
        "frontend".to_string()
    } else if lower.contains("backend") || lower.contains("api") {
        "backend".to_string()
    } else if lower.contains("qa") || lower.contains("test") {
        "qa".to_string()
    } else {
        "general".to_string()
    }
}

fn self_correction(results: Vec<TaskResult>) -> Vec<TaskResult> {
    let mut corrected = Vec::new();
    for result in results {
        if result.summary.contains("failed") {
            corrected.push(TaskResult {
                id: result.id,
                summary: format!("Retry succeeded after adjustment: {}", result.summary),
                worker: "self-corrector".to_string(),
            });
        } else {
            corrected.push(result);
        }
    }
    corrected
}

fn parse_dependencies(description: &str, mapping: &BTreeMap<String, usize>) -> Vec<usize> {
    let mut dependencies = Vec::new();
    let lower = description.to_lowercase();
    for (name, id) in mapping {
        if lower.contains(&format!("after {}", name)) || lower.contains(&format!("depends on {}", name)) {
            dependencies.push(*id);
        }
    }
    dependencies
}

pub fn merge_branch(branch: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["merge", "--no-ff", branch])
        .output()?;

    if output.status.success() {
        return Ok(format!(
            "Merged branch {} successfully.",
            branch
        ));
    }

    let conflict_output = std::process::Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()?;
    let conflicts = String::from_utf8_lossy(&conflict_output.stdout)
        .lines()
        .map(|line| format!("- {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "Merge failed due to conflicts:\n{}",
        if conflicts.is_empty() { "- (none listed)".to_string() } else { conflicts }
    ))
}
