use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub zentao_accounts: Vec<ZentaoAccount>,
    pub git: GitConfig,
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
    pub assigned_to: String,
    pub task_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub max_commits: usize,
    pub include_merged: bool,
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
pub struct OutputConfig {
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
