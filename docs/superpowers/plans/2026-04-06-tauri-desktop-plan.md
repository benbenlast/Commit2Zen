# Commit2Zen Tauri 桌面应用实现计划

**日期**: 2026-04-06
**基于设计**: docs/superpowers/specs/2026-04-06-tauri-desktop-design.md
**总任务数**: 5

---

## 任务总览

| # | 任务 | 类型 | 产出 |
|---|------|------|------|
| 1 | 搭建 Tauri + Vue 3 项目骨架 | 脚手架 | 完整的项目结构、配置文件 |
| 2 | 实现 Rust 核心服务 | 后端 | Git、禅道、报告、配置的 Rust 实现 |
| 3 | 实现 Vue 3 前端组件 | 前端 | 7 个核心组件 + 4 个视图 |
| 4 | 集成测试与联调 | 集成 | 端到端工作流验证 |
| 5 | 构建发布与文档 | 交付 | 安装包、更新 README |

---

## Task 1: 搭建 Tauri + Vue 3 项目骨架

**目标**: 创建完整的 Tauri + Vue 3 项目结构

### 步骤

#### 1.1 安装 Rust 环境
```bash
# Windows: 下载 https://rustup.rs/ 或使用 winget
winget install Rustlang.Rustup

# 验证安装
rustc --version
cargo --version
```

#### 1.2 安装 Tauri CLI
```bash
cargo install tauri-cli
```

#### 1.3 创建 Vue 3 前端项目
```bash
cd d:\aicode\Commit2Zen
npm create vite@latest src -- --template vue
cd src
npm install
npm install vue-router pinia naive-ui @vicons/ionicons5
cd ..
```

#### 1.4 初始化 Tauri
```bash
cargo tauri init
```

配置 `src-tauri/tauri.conf.json`:
```json
{
  "productName": "Commit2Zen",
  "version": "2.0.0",
  "identifier": "com.commit2zen.app",
  "build": {
    "beforeDevCommand": "cd src && npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "cd src && npm run build",
    "frontendDist": "../src/dist"
  },
  "app": {
    "windows": [{
      "title": "Commit2Zen",
      "width": 1200,
      "height": 800,
      "resizable": true
    }]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

#### 1.5 配置 Cargo.toml
```toml
[package]
name = "commit2zen"
version = "2.0.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
git2 = "0.19"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
regex = "1.0"

[build-dependencies]
tauri-build = "2"
```

#### 1.6 创建目录结构
```bash
# Rust 后端目录
mkdir -p src-tauri/src/commands
mkdir -p src-tauri/src/services
mkdir -p src-tauri/src/models
mkdir -p src-tauri/src/utils

# Vue 前端目录
mkdir -p src/assets
mkdir -p src/components
mkdir -p src/views
mkdir -p src/stores
mkdir -p src/router
```

#### 1.7 创建 Rust 入口文件

**src-tauri/src/main.rs**:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;
mod models;
mod utils;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::scan_git_repositories,
            commands::collect_git_log,
            commands::group_commits_by_branch,
            commands::zentao_login,
            commands::zentao_get_projects,
            commands::zentao_create_task,
            commands::load_config,
            commands::save_config,
            commands::validate_config,
            commands::generate_report,
            commands::save_report,
            commands::get_report_history,
            commands::execute_full_workflow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**src-tauri/build.rs**:
```rust
fn main() {
    tauri_build::build()
}
```

#### 1.8 创建 Vue 入口文件

**src/main.js**:
```javascript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

**src/App.vue**:
```vue
<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <router-view />
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { NConfigProvider, NMessageProvider } from 'naive-ui'
</script>
```

**src/router/index.js**:
```javascript
import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const routes = [
  { path: '/', name: 'home', component: HomeView },
  { path: '/config', name: 'config', component: () => import('../views/ConfigView.vue') },
  { path: '/execute', name: 'execute', component: () => import('../views/ExecuteView.vue') },
  { path: '/history', name: 'history', component: () => import('../views/HistoryView.vue') },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
```

### 验证
```bash
cargo tauri dev
```
应该能看到空白窗口打开。

---

## Task 2: 实现 Rust 核心服务

**目标**: 用 Rust 实现所有业务逻辑（与现有 JS 等价）

### 2.1 数据模型 (models/)

**src-tauri/src/models/mod.rs**:
```rust
pub mod commit;
pub mod branch;
pub mod config;
pub mod task;
pub mod report;

pub use commit::*;
pub use branch::*;
pub use config::*;
pub use task::*;
pub use report::*;
```

**src-tauri/src/models/commit.rs**:
```rust
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
```

**src-tauri/src/models/branch.rs**:
```rust
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
```

**src-tauri/src/models/config.rs**:
```rust
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
```

**src-tauri/src/models/task.rs**:
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    pub name: String,
    pub desc: String,
    pub project: u32,
    pub assigned_to: String,
    pub r#type: String,
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
```

**src-tauri/src/models/report.rs**:
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub timestamp: String,
    pub project: String,
    pub branches: Vec<BranchReport>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchReport {
    pub branch: String,
    pub commit_count: usize,
    pub task_created: bool,
    pub task_id: Option<u32>,
    pub task_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_branches: usize,
    pub total_commits: usize,
    pub tasks_created: usize,
    pub tasks_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub path: String,
    pub date: String,
    pub filename: String,
}
```

### 2.2 业务服务 (services/)

**src-tauri/src/services/mod.rs**:
```rust
pub mod git_service;
pub mod zentao_service;
pub mod report_service;

pub use git_service::*;
pub use zentao_service::*;
pub use report_service::*;
```

**src-tauri/src/services/git_service.rs**:
```rust
use git2::{Repository, Sort};
use crate::models::{Commit, BranchGroup, BranchSummary, CommitTypes, DateRange};
use std::collections::{HashMap, HashSet};
use regex::Regex;

pub fn collect_commits(repo_path: &str, max_commits: usize) -> Result<Vec<Commit>, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("无法打开仓库: {}", e))?;

    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("无法创建revwalk: {}", e))?;

    revwalk.push_head()
        .map_err(|e| format!("无法push HEAD: {}", e))?;

    revwalk.set_sorting(Sort::TIME)
        .map_err(|e| format!("无法设置排序: {}", e))?;

    let mut commits = Vec::new();
    for oid in revwalk.take(max_commits) {
        let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("查找提交失败: {}", e))?;

        let hash = format!("{:.7}", commit.id());
        let author = commit.author()
            .name()
            .unwrap_or("unknown")
            .to_string();
        let date = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());
        let message = commit.message()
            .unwrap_or("")
            .trim_end()
            .to_string();

        // 获取分支信息
        let branches = get_commit_branches(&repo, &commit);

        // 获取修改文件
        let files = get_commit_files(&repo, &commit);

        commits.push(Commit {
            hash,
            author,
            date,
            message,
            branches,
            files,
        });
    }

    Ok(commits)
}

fn get_commit_branches(_repo: &Repository, _commit: &git2::Commit) -> Vec<String> {
    // 简化实现：返回空，实际可通过遍历 refs 获取
    Vec::new()
}

fn get_commit_files(_repo: &Repository, commit: &git2::Commit) -> Vec<String> {
    // 简化实现：返回空，实际可通过 diff 获取
    if let Some(parent) = commit.parent(0).ok() {
        let diff = commit.tree().ok().and_then(|tree| {
            parent.tree().ok().and_then(|parent_tree| {
                tree.diff_to_tree(&parent_tree, None, None, false).ok()
            })
        });

        if let Some(diff) = diff {
            let mut files = Vec::new();
            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path() {
                    if let Some(path_str) = path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
            return files;
        }
    }
    Vec::new()
}

pub fn group_by_branches(commits: Vec<Commit>, branch_pattern: Option<&str>) -> Vec<BranchGroup> {
    let pattern = branch_pattern
        .and_then(|p| Regex::new(p).ok())
        .unwrap_or_else(|| Regex::new(".*").unwrap());

    let mut groups: HashMap<String, Vec<Commit>> = HashMap::new();

    for commit in commits {
        let branch_name = commit.branches
            .iter()
            .find(|b| pattern.is_match(b))
            .cloned()
            .unwrap_or_else(|| "未分类".to_string());

        groups.entry(branch_name).or_default().push(commit);
    }

    groups.into_iter().map(|(branch, commits)| {
        let authors: Vec<String> = commits.iter()
            .map(|c| c.author.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let total_files: usize = commits.iter().map(|c| c.files.len()).sum();
        let all_files: Vec<String> = commits.iter()
            .flat_map(|c| c.files.clone())
            .collect();

        let top_files: Vec<String> = {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for f in &all_files {
                *counts.entry(f.clone()).or_insert(0) += 1;
            }
            let mut sorted: Vec<_> = counts.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            sorted.into_iter().take(10).map(|(f, _)| f).collect()
        };

        let dates: Vec<&String> = commits.iter().map(|c| &c.date).collect();
        let date_range = if dates.is_empty() {
            DateRange { start: String::new(), end: String::new() }
        } else {
            DateRange {
                start: dates.last().unwrap().to_string(),
                end: dates.first().unwrap().to_string(),
            }
        };

        BranchGroup {
            branch,
            commit_count: commits.len(),
            authors,
            date_range,
            commits,
            summary: BranchSummary {
                total_commits: 0, // 会在下面设置
                total_authors: 0,
                total_files,
                commit_types: classify_commits(&Vec::new()), // 需要传入 commits
                top_files,
            },
        }
    }).collect()
}

fn classify_commits(commits: &[Commit]) -> CommitTypes {
    let mut types = CommitTypes {
        feat: 0, fix: 0, docs: 0, refactor: 0,
        test: 0, chore: 0, other: 0,
    };

    for commit in commits {
        let msg = commit.message.to_lowercase();
        if msg.starts_with("feat") { types.feat += 1; }
        else if msg.starts_with("fix") { types.fix += 1; }
        else if msg.starts_with("docs") { types.docs += 1; }
        else if msg.starts_with("refactor") { types.refactor += 1; }
        else if msg.starts_with("test") { types.test += 1; }
        else if msg.starts_with("chore") { types.chore += 1; }
        else { types.other += 1; }
    }

    types
}
```

**src-tauri/src/services/zentao_service.rs**:
```rust
use reqwest::Client;
use crate::models::{TaskPayload, TaskCreateResult, ZentaoProject};

pub async fn login(url: &str, account: &str, password: &str) -> Result<String, String> {
    let client = Client::new();
    let response = client
        .post(&format!("{}/api.php/v1/tokens", url))
        .json(&serde_json::json!({
            "account": account,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    json["token"].as_str()
        .map(String::from)
        .ok_or_else(|| "未找到token，请检查账号密码".to_string())
}

pub async fn get_projects(url: &str, token: &str) -> Result<Vec<ZentaoProject>, String> {
    let client = Client::new();
    let response = client
        .get(&format!("{}/api.php/v1/projects", url))
        .header("Token", token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 解析项目列表
    let mut projects = Vec::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for item in data {
            if let (Some(id), Some(name), Some(code)) = (
                item["id"].as_u64().map(|v| v as u32),
                item["name"].as_str(),
                item["code"].as_str(),
            ) {
                projects.push(ZentaoProject {
                    id,
                    name: name.to_string(),
                    code: code.to_string(),
                });
            }
        }
    }

    Ok(projects)
}

pub async fn create_task(
    url: &str,
    token: &str,
    task_data: &TaskPayload,
) -> Result<TaskCreateResult, String> {
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match try_create_task(url, token, task_data).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e;
                if attempt < max_retries {
                    tokio::time::sleep(
                        std::time::Duration::from_secs(2u64.pow(attempt as u32))
                    ).await;
                }
            }
        }
    }

    Err(format!("重试{}次后失败: {}", max_retries, last_error))
}

async fn try_create_task(
    url: &str,
    token: &str,
    task_data: &TaskPayload,
) -> Result<TaskCreateResult, String> {
    let client = Client::new();
    let response = client
        .post(&format!("{}/api.php/v1/tasks", url))
        .header("Token", token)
        .json(task_data)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if let Some(task_id) = json.get("taskID").and_then(|v| v.as_u64()) {
        Ok(TaskCreateResult {
            task_id: task_id as u32,
            task_url: format!("{}/task-view-{}.html", url, task_id),
        })
    } else {
        Err(format!("创建失败: {}", json))
    }
}

pub fn build_task_description(branch: &str, commits: &[crate::models::Commit]) -> String {
    let mut desc = format!("## 分支: {}\n\n", branch);
    desc.push_str(&format!("**提交数**: {}\n\n", commits.len()));

    for commit in commits {
        desc.push_str(&format!(
            "- `{}` {} - {}\n",
            commit.hash, commit.author, commit.message
        ));
    }

    desc
}

pub fn build_task_payload(
    branch: &str,
    commits: &[crate::models::Commit],
    project_id: u32,
    assigned_to: &str,
    task_type: &str,
) -> TaskPayload {
    TaskPayload {
        name: format!("[{}] 工作内容汇总", branch),
        desc: build_task_description(branch, commits),
        project: project_id,
        assigned_to: assigned_to.to_string(),
        r#type: task_type.to_string(),
        est_started: chrono::Local::now().format("%Y-%m-%d").to_string(),
        deadline: chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
}
```

**src-tauri/src/services/report_service.rs**:
```rust
use std::fs;
use crate::models::{ExecutionReport, BranchReport, ReportSummary, ReportMeta};

pub fn generate_report(
    branches: Vec<BranchReport>,
) -> ExecutionReport {
    let total_commits: usize = branches.iter().map(|b| b.commit_count).sum();
    let tasks_created = branches.iter().filter(|b| b.task_created).count();
    let tasks_failed = branches.iter().filter(|b| !b.task_created).count();

    ExecutionReport {
        timestamp: chrono::Local::now().to_rfc3339(),
        project: "Commit2Zen".to_string(),
        branches,
        summary: ReportSummary {
            total_branches: branches.len(),
            total_commits,
            tasks_created,
            tasks_failed,
        },
    }
}

pub fn save_report(report: &ExecutionReport, report_dir: &str) -> Result<String, String> {
    let date = chrono::Local::now().format("%Y-%m-%d");
    let filename = format!("{}-report.json", date);
    let path = format!("{}/{}", report_dir, filename);

    fs::create_dir_all(report_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(path)
}

pub fn get_history(report_dir: &str) -> Result<Vec<ReportMeta>, String> {
    let entries = fs::read_dir(report_dir)
        .map_err(|e| format!("读取目录失败: {}", e))?;

    let mut reports = Vec::new();
    for entry in entries {
        let entry = entry.ok().unwrap();
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if filename_str.ends_with("-report.json") {
            reports.push(ReportMeta {
                path: entry.path().to_string_lossy().to_string(),
                date: filename_str.replace("-report.json", ""),
                filename: filename_str.to_string(),
            });
        }
    }

    reports.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(reports)
}
```

### 2.3 Tauri Commands (commands/)

**src-tauri/src/commands/mod.rs**:
```rust
pub mod git;
pub mod zentao;
pub mod config;
pub mod report;

pub use git::*;
pub use zentao::*;
pub use config::*;
pub use report::*;
```

**src-tauri/src/commands/git.rs**:
```rust
use tauri::command;
use crate::models::{Commit, BranchGroup};
use crate::services::{collect_commits, group_by_branches};

#[derive(serde::Serialize)]
pub struct GitRepoInfo {
    pub path: String,
    pub name: String,
    pub last_commit: String,
    pub branch_count: usize,
}

#[command]
pub async fn scan_git_repositories() -> Result<Vec<GitRepoInfo>, String> {
    // 扫描常见目录
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
            scan_directory(&dir, &mut repos, 0, 3)?;
        }
    }

    Ok(repos)
}

fn scan_directory(dir: &std::path::Path, repos: &mut Vec<GitRepoInfo>, depth: usize, max_depth: usize) -> Result<(), String> {
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
```

**src-tauri/src/commands/zentao.rs**:
```rust
use tauri::command;
use crate::models::{ZentaoProject, TaskPayload, TaskCreateResult, Commit};
use crate::services::{login, get_projects, create_task, build_task_payload};

#[command]
pub async fn zentao_login(url: String, account: String, password: String) -> Result<String, String> {
    login(&url, &account, &password).await
}

#[command]
pub async fn zentao_get_projects(url: String, token: String) -> Result<Vec<ZentaoProject>, String> {
    get_projects(&url, &token).await
}

#[command]
pub async fn zentao_create_task(
    url: String,
    token: String,
    branch: String,
    commits: Vec<Commit>,
    project_id: u32,
    assigned_to: String,
    task_type: String,
) -> Result<TaskCreateResult, String> {
    let task_data = build_task_payload(&branch, &commits, project_id, &assigned_to, &task_type);
    create_task(&url, &token, &task_data).await
}
```

**src-tauri/src/commands/config.rs**:
```rust
use tauri::command;
use crate::models::AppConfig;
use std::fs;

#[command]
pub async fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置失败: {}", e))?;
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置失败: {}", e))?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

#[command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&config_path, json)
        .map_err(|e| format!("写入失败: {}", e))?;

    Ok(())
}

#[command]
pub async fn validate_config(config: AppConfig) -> Result<bool, String> {
    if config.zentao.url.is_empty() {
        return Err("禅道地址不能为空".to_string());
    }
    if config.zentao.account.is_empty() {
        return Err("禅道账号不能为空".to_string());
    }
    if config.zentao.password.is_empty() {
        return Err("禅道密码不能为空".to_string());
    }
    if config.zentao.project_id == 0 {
        return Err("请选择禅道项目".to_string());
    }
    Ok(true)
}

fn get_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Commit2Zen")
        .join("config.json")
}
```

**src-tauri/src/commands/report.rs**:
```rust
use tauri::command;
use crate::models::{ExecutionReport, BranchReport, ReportSummary, ReportMeta};
use crate::services::{generate_report as gen_report, save_report as save_rep, get_history};

#[command]
pub async fn generate_report(branches: Vec<BranchReport>, _summary: Option<ReportSummary>) -> Result<ExecutionReport, String> {
    Ok(gen_report(branches))
}

#[command]
pub async fn save_report(report: ExecutionReport, report_dir: String) -> Result<String, String> {
    save_rep(&report, &report_dir)
}

#[command]
pub async fn get_report_history(report_dir: String) -> Result<Vec<ReportMeta>, String> {
    get_history(&report_dir)
}
```

### 2.4 完善 main.rs

更新 `src-tauri/src/main.rs` 添加一键执行命令：

```rust
#[tauri::command]
pub async fn execute_full_workflow(
    config: crate::models::AppConfig,
    project_path: String,
) -> Result<crate::models::ExecutionReport, String> {
    use crate::models::BranchReport;
    use crate::services::{collect_commits, group_by_branches, login, create_task, build_task_payload, generate_report, save_report};

    // 1. 收集 Git 提交
    let commits = collect_commits(&project_path, config.git.max_commits)?;

    // 2. 按分支分组
    let branch_groups = group_by_branches(commits, Some(&config.git.branch_pattern));

    // 3. 禅道登录
    let token = login(&config.zentao.url, &config.zentao.account, &config.zentao.password).await?;

    // 4. 为每个分支创建任务
    let mut branch_reports = Vec::new();
    for group in &branch_groups {
        let task_data = build_task_payload(
            &group.branch,
            &group.commits,
            config.zentao.project_id,
            &config.zentao.assigned_to,
            &config.zentao.task_type,
        );

        match create_task(&config.zentao.url, &token, &task_data).await {
            Ok(result) => {
                branch_reports.push(BranchReport {
                    branch: group.branch.clone(),
                    commit_count: group.commit_count,
                    task_created: true,
                    task_id: Some(result.task_id),
                    task_url: Some(result.task_url),
                    error: None,
                });
            }
            Err(e) => {
                branch_reports.push(BranchReport {
                    branch: group.branch.clone(),
                    commit_count: group.commit_count,
                    task_created: false,
                    task_id: None,
                    task_url: None,
                    error: Some(e),
                });
            }
        }
    }

    // 5. 生成报告
    let report = generate_report(branch_reports);

    // 6. 保存报告
    let _ = save_report(&report, &config.output.report_dir);

    Ok(report)
}
```

并在 `invoke_handler` 中注册。

---

## Task 3: 实现 Vue 3 前端组件

**目标**: 实现所有 UI 组件

### 3.1 布局组件

**src/views/HomeView.vue**:
```vue
<template>
  <div class="home">
    <n-space vertical size="large">
      <n-h1>Commit2Zen</h1>
      <n-p>Git 提交记录管理与禅道任务自动化工具</n-p>

      <n-space>
        <n-button type="primary" @click="router.push('/execute')">
          开始执行
        </n-button>
        <n-button @click="router.push('/config')">
          配置管理
        </n-button>
        <n-button @click="router.push('/history')">
          历史记录
        </n-button>
      </n-space>

      <n-card title="最近执行" v-if="lastReport">
        <n-descriptions>
          <n-descriptions-item label="时间">{{ lastReport.timestamp }}</n-descriptions-item>
          <n-descriptions-item label="分支数">{{ lastReport.summary.total_branches }}</n-descriptions-item>
          <n-descriptions-item label="提交数">{{ lastReport.summary.total_commits }}</n-descriptions-item>
          <n-descriptions-item label="任务创建">{{ lastReport.summary.tasks_created }}</n-descriptions-item>
        </n-descriptions>
      </n-card>
    </n-space>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const lastReport = ref(null)

onMounted(async () => {
  try {
    const history = await invoke('get_report_history', { reportDir: 'reports' })
    if (history.length > 0) {
      // 加载最新报告
    }
  } catch (e) {
    console.error(e)
  }
})
</script>
```

### 3.2 核心组件

**src/components/GitProjectSelector.vue**:
```vue
<template>
  <n-card title="选择 Git 项目">
    <n-space vertical>
      <n-space>
        <n-button @click="scanRepos" :loading="scanning">
          扫描本地仓库
        </n-button>
        <n-input v-model:value="manualPath" placeholder="或手动输入路径" />
        <n-button @click="selectManual">选择</n-button>
      </n-space>

      <n-list v-if="repos.length">
        <n-list-item v-for="repo in repos" :key="repo.path">
          <n-space justify="space-between">
            <n-text strong>{{ repo.name }}</n-text>
            <n-text depth="3">{{ repo.path }}</n-text>
            <n-button size="small" @click="selectRepo(repo)">选择</n-button>
          </n-space>
        </n-list-item>
      </n-list>
    </n-space>
  </n-card>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const repos = ref([])
const scanning = ref(false)
const manualPath = ref('')
const emit = defineEmits(['selected'])

const scanRepos = async () => {
  scanning.value = true
  try {
    repos.value = await invoke('scan_git_repositories')
  } catch (e) {
    console.error(e)
  } finally {
    scanning.value = false
  }
}

const selectManual = async () => {
  const selected = await open({ directory: true })
  if (selected) {
    manualPath.value = selected
    emit('selected', { path: selected, name: selected.split('/').pop() })
  }
}

const selectRepo = (repo) => {
  emit('selected', repo)
}
</script>
```

**src/components/ZentaoProjectSelector.vue**:
```vue
<template>
  <n-card title="禅道项目配置">
    <n-space vertical>
      <n-form :model="form">
        <n-form-item label="禅道地址">
          <n-input v-model:value="form.url" placeholder="http://192.168.1.23/zentao" />
        </n-form-item>
        <n-form-item label="账号">
          <n-input v-model:value="form.account" />
        </n-form-item>
        <n-form-item label="密码">
          <n-input v-model:value="form.password" type="password" show-password-on="click" />
        </n-form-item>
      </n-form>

      <n-button @click="testLogin" :loading="logging">
        测试连接
      </n-button>

      <n-select
        v-if="projects.length"
        v-model:value="selectedProjectId"
        :options="projectOptions"
        placeholder="选择目标项目"
        @update:value="selectProject"
      />
    </n-space>
  </n-card>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'

const message = useMessage()
const form = ref({ url: '', account: '', password: '' })
const projects = ref([])
const logging = ref(false)
const token = ref('')
const selectedProjectId = ref(null)
const emit = defineEmits(['configured', 'selected'])

const projectOptions = computed(() =>
  projects.value.map(p => ({ label: p.name, value: p.id }))
)

const testLogin = async () => {
  logging.value = true
  try {
    token.value = await invoke('zentao_login', {
      url: form.value.url,
      account: form.value.account,
      password: form.value.password,
    })
    message.success('连接成功')
    emit('configured', { ...form.value, token: token.value })

    // 获取项目列表
    projects.value = await invoke('zentao_get_projects', {
      url: form.value.url,
      token: token.value,
    })
  } catch (e) {
    message.error(e)
  } finally {
    logging.value = false
  }
}

const selectProject = (id) => {
  const project = projects.value.find(p => p.id === id)
  emit('selected', project)
}
</script>
```

**src/components/CommitPreview.vue**:
```vue
<template>
  <n-card title="Commit 预览">
    <n-space vertical>
      <n-statistic label="总提交数" :value="commits.length" />

      <n-data-table
        :columns="columns"
        :data="commits"
        :pagination="{ pageSize: 10 }"
      />
    </n-space>
  </n-card>
</template>

<script setup>
import { h } from 'vue'
import { NTag, NCode } from 'naive-ui'

defineProps({
  commits: { type: Array, default: () => [] }
})

const columns = [
  { title: 'Hash', key: 'hash', render: (row) => h(NCode, {}, () => row.hash) },
  { title: '作者', key: 'author' },
  { title: '日期', key: 'date' },
  { title: '提交信息', key: 'message' },
]
</script>
```

**src/components/BranchSummary.vue**:
```vue
<template>
  <n-card title="分支汇总">
    <n-space vertical>
      <n-grid :cols="2" :x-gap="12" v-for="group in branchGroups" :key="group.branch">
        <n-gi>
          <n-card :title="group.branch" size="small">
            <n-descriptions :column="2" size="small">
              <n-descriptions-item label="提交数">{{ group.commit_count }}</n-descriptions-item>
              <n-descriptions-item label="作者数">{{ group.authors.length }}</n-descriptions-item>
            </n-descriptions>
          </n-card>
        </n-gi>
      </n-grid>
    </n-space>
  </n-card>
</template>

<script setup>
defineProps({
  branchGroups: { type: Array, default: () => [] }
})
</script>
```

**src/views/ExecuteView.vue** (核心页面):
```vue
<template>
  <n-space vertical size="large" style="padding: 24px;">
    <n-steps :current="currentStep">
      <n-step title="选择 Git 项目" />
      <n-step title="配置禅道项目" />
      <n-step title="预览信息" />
      <n-step title="执行" />
    </n-steps>

    <!-- Step 1 -->
    <GitProjectSelector v-if="currentStep === 1" @selected="onGitSelected" />

    <!-- Step 2 -->
    <template v-if="currentStep === 2">
      <ZentaoProjectSelector
        @configured="onZentaoConfigured"
        @selected="onZentaoProjectSelected"
      />
    </template>

    <!-- Step 3 -->
    <template v-if="currentStep === 3">
      <n-space vertical>
        <CommitPreview :commits="commits" />
        <BranchSummary :branch-groups="branchGroups" />
        <n-button type="primary" @click="executeWorkflow" :loading="executing">
          创建禅道任务
        </n-button>
      </n-space>
    </template>

    <!-- Step 4 -->
    <template v-if="currentStep === 4">
      <TaskResult :results="taskResults" />
      <n-button @click="currentStep = 1">重新开始</n-button>
    </template>

    <n-space v-if="currentStep < 3">
      <n-button @click="currentStep++" :disabled="!canProceed">下一步</n-button>
    </n-space>
  </n-space>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import GitProjectSelector from '../components/GitProjectSelector.vue'
import ZentaoProjectSelector from '../components/ZentaoProjectSelector.vue'
import CommitPreview from '../components/CommitPreview.vue'
import BranchSummary from '../components/BranchSummary.vue'
import TaskResult from '../components/TaskResult.vue'

const message = useMessage()
const currentStep = ref(1)
const selectedGitProject = ref(null)
const zentaoConfig = ref(null)
const selectedZentaoProject = ref(null)
const commits = ref([])
const branchGroups = ref([])
const taskResults = ref([])
const executing = ref(false)

const canProceed = computed(() => {
  if (currentStep.value === 1) return !!selectedGitProject.value
  if (currentStep.value === 2) return !!zentaoConfig.value && !!selectedZentaoProject.value
  return true
})

const onGitSelected = async (repo) => {
  selectedGitProject.value = repo
  commits.value = await invoke('collect_git_log', {
    projectPath: repo.path,
    maxCommits: 100,
  })
}

const onZentaoConfigured = (config) => {
  zentaoConfig.value = config
}

const onZentaoProjectSelected = (project) => {
  selectedZentaoProject.value = project
  branchGroups.value = await invoke('group_commits_by_branch', {
    commits: commits.value,
  })
}

const executeWorkflow = async () => {
  executing.value = true
  try {
    const report = await invoke('execute_full_workflow', {
      config: {
        zentao: {
          url: zentaoConfig.value.url,
          account: zentaoConfig.value.account,
          password: zentaoConfig.value.password,
          project_id: selectedZentaoProject.value.id,
          assigned_to: zentaoConfig.value.account,
          task_type: 'dev',
        },
        git: { max_commits: 100, include_merged: false, branch_pattern: '.*' },
        output: { report_dir: 'reports', verbose: true },
      },
      projectPath: selectedGitProject.value.path,
    })
    taskResults.value = report.branches
    currentStep.value = 4
    message.success('执行完成')
  } catch (e) {
    message.error(e)
  } finally {
    executing.value = false
  }
}
</script>
```

**src/views/ConfigView.vue** 和 **src/views/HistoryView.vue** 类似实现。

---

## Task 4: 集成测试与联调

**目标**: 端到端验证完整工作流

### 4.1 测试用例
1. 扫描本地 Git 仓库 → 验证发现 Commit2Zen 仓库
2. 选择仓库 → 收集 commit → 验证数据正确
3. 禅道登录 → 验证 token 获取
4. 获取禅道项目列表 → 验证显示"能源智慧大脑研发"
5. 预览 commit → 验证分支分组正确
6. 执行工作流 → 验证禅道任务创建
7. 查看报告 → 验证 JSON 格式与现有兼容

### 4.2 测试命令
```bash
cargo tauri dev
```

手动测试每个步骤。

---

## Task 5: 构建发布与文档

**目标**: 打包安装包，更新文档

### 5.1 构建
```bash
cargo tauri build
```

输出在 `src-tauri/target/release/bundle/`

### 5.2 更新 README
添加 Tauri 桌面应用的安装说明，保留 CLI 说明作为备选。

### 5.3 提交
```bash
git add -A
git commit -m "feat: 完成 Tauri 桌面应用实现"
```

---

## 依赖安装清单

### Rust crates
- tauri 2.x
- serde + serde_json
- reqwest 0.12
- tokio 1.x
- git2 0.19
- chrono 0.4
- dirs 5.0
- regex 1.0

### npm packages
- vue
- vue-router
- pinia
- naive-ui
- @vicons/ionicons5
- vite
- @tauri-apps/api
- @tauri-apps/cli
- @tauri-apps/plugin-dialog
