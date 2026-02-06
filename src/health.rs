use serde::{Deserialize, Serialize};

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
