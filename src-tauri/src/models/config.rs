use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(rename = "zentaoAccounts", alias = "zentao_accounts")]
    pub zentao_accounts: Vec<ZentaoAccount>,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

/// 禅道账号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZentaoAccount {
    pub id: String,
    pub name: String,
    pub url: String,
    pub account: String,
    pub password: String,
    #[serde(rename = "assignedTo", alias = "assigned_to")]
    pub assigned_to: String,
    #[serde(rename = "taskType", alias = "task_type")]
    pub task_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    #[serde(rename = "maxCommits", alias = "max_commits")]
    pub max_commits: usize,
    #[serde(rename = "includeMerged", alias = "include_merged")]
    pub include_merged: bool,
    #[serde(rename = "branchPattern", alias = "branch_pattern")]
    pub branch_pattern: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        GitConfig {
            max_commits: 100,
            include_merged: false,
            branch_pattern: ".*".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    #[serde(rename = "reportDir", alias = "report_dir")]
    pub report_dir: String,
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            report_dir: "reports".to_string(),
            verbose: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            zentao_accounts: Vec::new(),
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
