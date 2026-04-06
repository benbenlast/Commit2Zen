use tauri::command;
use crate::models::{Commit, BranchGroup};
use crate::services::{collect_commits, group_by_branches};
use std::path::Path;

#[derive(serde::Serialize)]
pub struct GitRepoInfo {
    pub path: String,
    pub name: String,
    pub last_commit: String,
    pub branch_count: usize,
}

#[command]
pub async fn scan_git_repositories() -> Result<Vec<GitRepoInfo>, String> {
    let search_dirs = vec![
        dirs::home_dir().unwrap_or_default().join("projects"),
        dirs::home_dir().unwrap_or_default().join("code"),
        dirs::home_dir().unwrap_or_default().join("workspace"),
        dirs::home_dir().unwrap_or_default().join("Desktop"),
        dirs::home_dir().unwrap_or_default().join("Documents"),
    ];

    let mut repos = Vec::new();
    for dir in search_dirs {
        if dir.exists() {
            let _ = scan_directory(&dir, &mut repos, 0, 3);
        }
    }

    Ok(repos)
}

fn scan_directory(dir: &Path, repos: &mut Vec<GitRepoInfo>, depth: usize, max_depth: usize) -> Result<(), String> {
    if depth > max_depth {
        return Ok(());
    }

    let git_path = dir.join(".git");
    if git_path.exists() {
        let name = dir.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        repos.push(GitRepoInfo {
            path: dir.to_string_lossy().to_string(),
            name,
            last_commit: String::new(),
            branch_count: 0,
        });
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = scan_directory(&path, repos, depth + 1, max_depth);
            }
        }
    }

    Ok(())
}

#[command]
pub async fn collect_git_log(project_path: String, max_commits: usize) -> Result<Vec<Commit>, String> {
    collect_commits(&project_path, max_commits)
}

#[command]
pub async fn group_commits_by_branch(commits: Vec<Commit>, branch_pattern: Option<String>) -> Result<Vec<BranchGroup>, String> {
    Ok(group_by_branches(commits, branch_pattern.as_deref()))
}
