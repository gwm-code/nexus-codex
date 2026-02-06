use serde::{Deserialize, Serialize};

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
