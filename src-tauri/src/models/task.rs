use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    pub name: String,
    pub desc: String,
    pub project: u32,
    #[serde(rename = "assignedTo")]
    pub assigned_to: String,
    #[serde(rename = "type")]
    pub task_type: String,
    #[serde(rename = "estStarted")]
    pub est_started: String,
    pub deadline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateResult {
    pub task_id: u32,
    pub task_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZentaoProject {
    pub id: u32,
    pub name: String,
    pub code: String,
}
