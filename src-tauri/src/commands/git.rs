use tauri::command;
use crate::models::{Commit, BranchGroup};
use crate::services::{collect_commits, group_by_branches};
use crate::services::scanner;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

// 取消标志的存储类型
pub type ScanCancelMap = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;

#[derive(serde::Serialize, Clone)]
pub struct GitRepoInfo {
    pub path: String,
    pub name: String,
    pub last_commit: String,
    pub branch_count: usize,
}

#[command]
pub async fn scan_git_repositories() -> Result<Vec<GitRepoInfo>, String> {
    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = vec![
        home.join("projects"),
        home.join("code"),
        home.join("workspace"),
        home.join("Desktop"),
        home.join("Documents"),
        home.join("aicode"),
        home.join("source"),
        home.join("repos"),
        home.join("git"),
        // Windows 常见路径
        Path::new(r"D:\aicode").to_path_buf(),
        Path::new(r"D:\code").to_path_buf(),
        Path::new(r"D:\projects").to_path_buf(),
        Path::new(r"D:\workspace").to_path_buf(),
    ];

    let mut repos = Vec::new();

    eprintln!("[扫描] 开始扫描 Git 仓库...");
    for dir in &search_dirs {
        eprintln!("[扫描] 检查目录: {}", dir.display());
        if dir.exists() && dir.is_dir() {
            eprintln!("[扫描] 目录存在，开始扫描: {}", dir.display());
            let _ = scan_directory(dir, &mut repos, 0, 3);
        } else {
            eprintln!("[扫描] 目录不存在: {}", dir.display());
        }
    }

    // 去重
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    repos.dedup_by(|a, b| a.path == b.path);

    eprintln!("扫描完成，找到 {} 个仓库", repos.len());
    for repo in &repos {
        eprintln!("  - {} ({})", repo.name, repo.path);
    }

    Ok(repos)
}

fn scan_directory(dir: &Path, repos: &mut Vec<GitRepoInfo>, depth: usize, max_depth: usize) -> Result<(), String> {
    if depth > max_depth {
        return Ok(());
    }

    eprintln!("[扫描] 深度 {}, 目录: {}", depth, dir.display());

    let git_path = dir.join(".git");
    if git_path.exists() {
        eprintln!("[扫描] 找到 Git 仓库: {}", dir.display());
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

/// 扫描指定文件夹
#[command]
pub async fn start_folder_scan(
    folder_path: String,
    app: tauri::AppHandle,
    cancel_map: tauri::State<'_, ScanCancelMap>,
) -> Result<String, String> {
    eprintln!("[扫描] 开始文件夹扫描: {}", folder_path);
    
    // 生成扫描 ID
    let scan_id = uuid::Uuid::new_v4().to_string();
    
    // 创建取消标志
    let cancel_flag = Arc::new(AtomicBool::new(false));
    cancel_map.lock().unwrap().insert(scan_id.clone(), cancel_flag.clone());
    
    // 验证路径
    let path = std::path::PathBuf::from(&folder_path);
    if !path.exists() || !path.is_dir() {
        eprintln!("[扫描] 目录不存在: {}", folder_path);
        return Err(format!("目录不存在: {}", folder_path));
    }
    
    // 在后台启动扫描
    let app_ref = app.clone();
    tauri::async_runtime::spawn(async move {
        eprintln!("[扫描] 启动后台扫描任务");
        let repos = tokio::task::spawn_blocking(move || {
            scanner::scan_folder(&path, &app_ref, &cancel_flag)
        }).await.unwrap_or_default();
        
        eprintln!("[扫描] 扫描完成，找到 {} 个仓库", repos.len());
        let _ = repos;
    });
    
    Ok(scan_id)
}

/// 取消扫描
#[command]
pub async fn cancel_scan(
    scan_id: String,
    cancel_map: tauri::State<'_, ScanCancelMap>,
) -> Result<(), String> {
    let map = cancel_map.lock().unwrap();
    if let Some(flag) = map.get(&scan_id) {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    } else {
        Err(format!("未找到扫描任务: {}", scan_id))
    }
}
