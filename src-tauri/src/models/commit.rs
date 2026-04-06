use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub branches: Vec<String>,
    pub files: Vec<String>,
}

/// 日期范围筛选器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    pub start: Option<i64>,  // Unix timestamp (seconds)
    pub end: Option<i64>,
}
