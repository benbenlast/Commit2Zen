use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub zentao: ZentaoConfig,
    pub git: GitConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZentaoConfig {
    pub url: String,
    pub account: String,
    pub password: String,
    pub project_id: u32,
    pub assigned_to: String,
    pub task_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub max_commits: usize,
    pub include_merged: bool,
    pub branch_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub report_dir: String,
    pub verbose: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            zentao: ZentaoConfig {
                url: String::new(),
                account: String::new(),
                password: String::new(),
                project_id: 0,
                assigned_to: String::new(),
                task_type: "dev".to_string(),
            },
            git: GitConfig {
                max_commits: 100,
                include_merged: false,
                branch_pattern: ".*".to_string(),
            },
            output: OutputConfig {
                report_dir: "reports".to_string(),
                verbose: true,
            },
        }
    }
}
