use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: usize,
    pub summary: String,
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
        });
    }
    results
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
            detail: format!("[{}] {}", result.id, result.summary),
        })
        .collect()
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
