use serde::{Serialize, Deserialize};
use super::commit::Commit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchGroup {
    pub branch: String,
    pub commit_count: usize,
    pub authors: Vec<String>,
    pub date_range: DateRange,
    pub commits: Vec<Commit>,
    pub summary: BranchSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSummary {
    pub total_commits: usize,
    pub total_authors: usize,
    pub total_files: usize,
    pub commit_types: CommitTypes,
    pub top_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTypes {
    pub feat: usize,
    pub fix: usize,
    pub docs: usize,
    pub refactor: usize,
    pub test: usize,
    pub chore: usize,
    pub other: usize,
}
