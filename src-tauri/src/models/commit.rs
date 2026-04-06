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
