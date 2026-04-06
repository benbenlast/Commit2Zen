use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use walkdir::WalkDir;

use crate::commands::git::GitRepoInfo;
use crate::models::{ScanProgress, ScanStatus, RepoInfo};
use crate::services::skip_list::should_skip_directory;
use tauri::Emitter;

/// 扫描指定文件夹（使用 walkdir 优化）
pub fn scan_folder(
    root_path: &Path,
    app: &tauri::AppHandle,
    cancel_flag: &AtomicBool,
) -> Vec<GitRepoInfo> {
    let mut repos = Vec::new();
    let mut directories_scanned: usize = 0;
    let mut last_emit = Instant::now();

    // 发送开始事件
    let _ = app.emit("scan-progress", ScanProgress {
        status: ScanStatus::Started,
        current_directory: root_path.to_string_lossy().to_string(),
        repos_found: 0,
        directories_scanned: 0,
        percentage: 0.0,
        repo: None,
    });

    // 使用 walkdir 进行遍历，自带剪枝优化
    for entry in WalkDir::new(root_path)
        .follow_links(false) // 不跟随符号链接，防止无限循环
        .into_iter()
        .filter_entry(|e| {
            // 剪枝：跳过 .git 内容和已知的非项目目录
            if e.file_type().is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    if name == ".git" || should_skip_directory(name) {
                        return false;
                    }
                }
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        // 检查取消
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = app.emit("scan-progress", ScanProgress {
                status: ScanStatus::Cancelled,
                current_directory: entry.path().to_string_lossy().to_string(),
                repos_found: repos.len(),
                directories_scanned,
                percentage: 0.0,
                repo: None,
            });
            return repos;
        }

        // 只处理目录
        if entry.file_type().is_dir() {
            directories_scanned += 1;

            // 检查是否为 Git 仓库
            if entry.path().join(".git").exists() {
                if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    let repo_path = entry.path().to_string_lossy().to_string();
                    repos.push(GitRepoInfo {
                        path: repo_path.clone(),
                        name: name.to_string(),
                        last_commit: String::new(),
                        branch_count: 0,
                    });

                    // 发送发现仓库事件（包含仓库信息）
                    let _ = app.emit("scan-progress", ScanProgress {
                        status: ScanStatus::Found,
                        current_directory: repo_path.clone(),
                        repos_found: repos.len(),
                        directories_scanned,
                        percentage: 0.0,
                        repo: Some(RepoInfo {
                            path: repo_path,
                            name: name.to_string(),
                        }),
                    });
                }
            }
        }

        // 速率限制：每 100 个目录或每 500ms 发送一次进度
        if directories_scanned % 100 == 0 || last_emit.elapsed().as_millis() >= 500 {
            let _ = app.emit("scan-progress", ScanProgress {
                status: ScanStatus::Scanning,
                current_directory: entry.path().to_string_lossy().to_string(),
                repos_found: repos.len(),
                directories_scanned,
                percentage: 0.0,
                repo: None,
            });
            last_emit = Instant::now();
        }
    }

    // 去重
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    repos.dedup_by(|a, b| a.path == b.path);

    // 发送完成事件
    let _ = app.emit("scan-progress", ScanProgress {
        status: ScanStatus::Completed,
        current_directory: String::new(),
        repos_found: repos.len(),
        directories_scanned,
        percentage: 100.0,
        repo: None,
    });

    repos
}
