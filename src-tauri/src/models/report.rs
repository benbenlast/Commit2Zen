use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub timestamp: String,
    pub project: String,
    pub branches: Vec<BranchReport>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchReport {
    pub branch: String,
    pub commit_count: usize,
    pub task_created: bool,
    pub task_id: Option<u32>,
    pub task_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_branches: usize,
    pub total_commits: usize,
    pub tasks_created: usize,
    pub tasks_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub path: String,
    pub date: String,
    pub filename: String,
}
