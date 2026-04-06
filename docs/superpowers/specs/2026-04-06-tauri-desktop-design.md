# Commit2Zen Tauri 桌面应用设计文档

**日期**: 2026-04-06
**状态**: 草稿
**版本**: v2.0

---

## 1. 概述

将 Commit2Zen 从 Node.js CLI 工具完全重构为 **Tauri 桌面应用**，使用 Rust 重写核心逻辑，Vue 3 构建现代化 GUI。

### 1.1 目标
- 保留现有所有功能（Git 采集、分支分组、禅道任务创建、报告生成）
- 提供可视化桌面界面
- 提升性能和安全性（Rust 原生）
- 支持本地 Git 仓库扫描发现

### 1.2 技术栈
| 层级 | 技术 | 说明 |
|------|------|------|
| 前端框架 | Vue 3 + Vite | 组件化开发 |
| UI 库 | Naive UI / Element Plus | 组件库 |
| 状态管理 | Pinia | 轻量状态管理 |
| 桌面框架 | Tauri 2.x | Rust 后端 + Web 前端 |
| Rust HTTP | reqwest | 异步 HTTP 客户端 |
| Rust Git | git2 (libgit2) | Git 操作 |
| Rust 序列化 | serde + serde_json | JSON 处理 |
| Rust 日期 | chrono | 时间处理 |

---

## 2. 项目结构

```
Commit2Zen/
├── src-tauri/                    # Tauri Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri 应用配置
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # Tauri app setup
│   │   ├── commands/             # Tauri commands (API 端点)
│   │   │   ├── mod.rs
│   │   │   ├── git.rs            # Git 操作命令
│   │   │   ├── zentao.rs         # 禅道 API 命令
│   │   │   ├── config.rs         # 配置管理命令
│   │   │   └── report.rs         # 报告生成命令
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── mod.rs
│   │   │   ├── git_service.rs    # Git 采集与分支分组
│   │   │   ├── zentao_service.rs # 禅道 API 客户端
│   │   │   └── report_service.rs # 报告生成
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── commit.rs         # Commit 结构体
│   │   │   ├── branch.rs         # Branch 分组结构体
│   │   │   ├── config.rs         # 配置结构体
│   │   │   └── task.rs           # 禅道任务结构体
│   │   └── utils/                # 工具函数
│   │       ├── mod.rs
│   │       └── fs.rs             # 文件操作
│   ├── icons/                    # 应用图标
│   └── build.rs
├── src/                          # Vue 3 前端
│   ├── assets/                   # 静态资源
│   ├── components/               # Vue 组件
│   │   ├── GitProjectSelector.vue    # Git 项目选择器
│   │   ├── ZentaoProjectSelector.vue # 禅道项目选择器
│   │   ├── CommitPreview.vue         # Commit 预览
│   │   ├── BranchSummary.vue         # 分支汇总
│   │   ├── ConfigForm.vue            # 配置表单
│   │   ├── TaskResult.vue            # 任务创建结果
│   │   └── ReportViewer.vue          # 报告查看器
│   ├── views/                    # 页面视图
│   │   ├── HomeView.vue              # 首页/仪表盘
│   │   ├── ConfigView.vue            # 配置页
│   │   ├── ExecuteView.vue           # 执行页（核心工作流）
│   │   └── HistoryView.vue           # 历史记录
│   ├── stores/                   # Pinia 状态
│   │   ├── config.js
│   │   ├── git.js
│   │   └── zentao.js
│   ├── router/                   # Vue Router
│   │   └── index.js
│   ├── App.vue
│   └── main.js
├── index.html
├── package.json
├── vite.config.js
├── .gitignore
└── README.md
```

---

## 3. 数据模型

### 3.1 Commit 结构体 (Rust)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,           // 7字符短hash
    pub author: String,         // 提交者
    pub date: String,           // ISO 8601
    pub message: String,        // 提交信息
    pub branches: Vec<String>,  // 所属分支
    pub files: Vec<String>,     // 修改的文件
}
```

### 3.2 BranchGroup 结构体 (Rust)
```rust
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

### 3.3 Config 结构体 (Rust)
```rust
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
```

### 3.4 TaskResult 结构体 (Rust)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub branch: String,
    pub commit_count: usize,
    pub task_created: bool,
    pub task_id: Option<u32>,
    pub task_url: Option<String>,
    pub error: Option<String>,
}
```

### 3.5 ExecutionReport 结构体 (Rust)
```rust
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
```

---

## 4. Tauri Commands (Rust API)

### 4.1 Git 相关命令
```rust
#[tauri::command]
async fn scan_git_repositories() -> Result<Vec<GitRepoInfo>, String>;

#[tauri::command]
async fn collect_git_log(project_path: String, max_commits: usize) -> Result<Vec<Commit>, String>;

#[tauri::command]
async fn group_commits_by_branch(commits: Vec<Commit>) -> Result<Vec<BranchGroup>, String>;
```

### 4.2 禅道相关命令
```rust
#[tauri::command]
async fn zentao_login(url: String, account: String, password: String) -> Result<String, String>;

#[tauri::command]
async fn zentao_get_projects(url: String, token: String) -> Result<Vec<ZentaoProject>, String>;

#[tauri::command]
async fn zentao_create_task(
    url: String,
    token: String,
    task_data: TaskPayload
) -> Result<TaskCreateResult, String>;
```

### 4.3 配置相关命令
```rust
#[tauri::command]
async fn load_config() -> Result<AppConfig, String>;

#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String>;

#[tauri::command]
async fn validate_config(config: AppConfig) -> Result<bool, String>;
```

### 4.4 报告相关命令
```rust
#[tauri::command]
async fn generate_report(
    branches: Vec<BranchReport>,
    summary: ReportSummary
) -> Result<ExecutionReport, String>;

#[tauri::command]
async fn save_report(report: ExecutionReport, report_dir: String) -> Result<String, String>;

#[tauri::command]
async fn get_report_history(report_dir: String) -> Result<Vec<ReportMeta>, String>;
```

### 4.5 一键执行命令
```rust
#[tauri::command]
async fn execute_full_workflow(
    config: AppConfig,
    project_path: String
) -> Result<ExecutionReport, String>;
```

---

## 5. 前端组件设计

### 5.1 页面路由

| 路由 | 组件 | 说明 |
|------|------|------|
| `/` | HomeView | 仪表盘：快速入口、最近执行记录 |
| `/config` | ConfigView | 禅道配置、Git 配置 |
| `/execute` | ExecuteView | 核心工作流：选择项目 → 预览 → 执行 |
| `/history` | HistoryView | 历史报告查看 |

### 5.2 核心组件

#### GitProjectSelector.vue
- 扫描本地常见目录（~/projects, ~/code, ~/workspace, Desktop 等）
- 发现 `.git` 目录即为 Git 仓库
- 支持手动输入路径
- 显示仓库信息（名称、路径、最近提交、分支数）
- 支持多选（批量处理）

#### ZentaoProjectSelector.vue
- 输入禅道地址、账号密码
- 测试连接按钮
- 获取并显示项目/执行列表
- 选择目标项目
- 显示项目负责人、成员等

#### CommitPreview.vue
- 表格展示 commit 列表
- 支持按分支筛选
- 显示作者、日期、提交信息
- 统计信息卡片（总提交数、作者数、文件数）

#### BranchSummary.vue
- 卡片式展示每个分支的统计
- commit 类型分布图表
- 修改文件 Top 10
- 时间范围

#### ConfigForm.vue
- 禅道配置表单（URL、账号、密码、项目ID、指派给、任务类型）
- Git 配置表单（最大提交数、分支过滤模式）
- 输出配置（报告目录）
- 保存/测试连接按钮
- 表单验证

#### TaskResult.vue
- 表格展示任务创建结果
- 成功/失败状态
- 任务链接可点击跳转
- 错误信息展示

#### ReportViewer.vue
- 选择历史报告
- JSON 格式化展示
- 导出功能

### 5.3 状态管理 (Pinia)

#### configStore.js
```javascript
{
  state: {
    zentao: { url, account, password, projectId, assignedTo, taskType },
    git: { maxCommits, includeMerged, branchPattern },
    output: { reportDir, verbose }
  },
  actions: { load, save, validate }
}
```

#### gitStore.js
```javascript
{
  state: {
    selectedProject: null,
    commits: [],
    branchGroups: [],
    scanning: false
  },
  actions: { scanRepos, collectLog, groupBranches }
}
```

#### zentaoStore.js
```javascript
{
  state: {
    token: null,
    projects: [],
    selectedProject: null,
    taskResults: [],
    creating: false
  },
  actions: { login, getProjects, createTask }
}
```

---

## 6. 核心工作流

### 6.1 一键执行流程

```
用户选择 Git 项目
    ↓
收集 Git 提交记录 (collect_git_log)
    ↓
按分支分组 (group_commits_by_branch)
    ↓
预览界面展示
    ↓
用户确认执行
    ↓
禅道登录 (zentao_login)
    ↓
为每个分支创建任务 (zentao_create_task)
    ↓
生成报告 (generate_report + save_report)
    ↓
展示结果
```

### 6.2 Git 项目扫描流程

```
扫描常见目录 (~/projects, ~/code, ~/workspace)
    ↓
递归查找 .git 目录 (深度 3 层)
    ↓
提取仓库信息 (名称、路径、最近提交)
    ↓
展示在 GitProjectSelector 中
    ↓
用户选择/手动输入
```

### 6.3 禅道项目获取流程

```
输入禅道 URL + 账号密码
    ↓
测试连接 (zentao_login)
    ↓
获取项目列表 (zentao_get_projects)
    ↓
展示项目选择器
    ↓
用户选择目标项目
```

---

## 7. Rust 核心服务实现

### 7.1 Git Service (git_service.rs)

使用 `git2` crate 替代 shell 命令：

```rust
use git2::{Repository, Commit, Sort};

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
        // 解析 commit 信息
        // ...
    }

    Ok(commits)
}

pub fn group_by_branches(commits: Vec<Commit>) -> Vec<BranchGroup> {
    // 与现有 JS 逻辑等价
    // ...
}
```

### 7.2 Zentao Service (zentao_service.rs)

使用 `reqwest` 进行 HTTP 调用：

```rust
use reqwest::Client;

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
        .ok_or_else(|| "未找到token".to_string())
}

pub async fn create_task(
    url: &str,
    token: &str,
    task_data: &TaskPayload
) -> Result<TaskCreateResult, String> {
    // 重试逻辑 + 指数退避
    // ...
}
```

### 7.3 Report Service (report_service.rs)

```rust
use std::fs;
use chrono::Local;

pub fn generate_report(
    branches: Vec<BranchReport>,
    summary: ReportSummary
) -> ExecutionReport {
    ExecutionReport {
        timestamp: Local::now().to_rfc3339(),
        project: "Commit2Zen".to_string(),
        branches,
        summary,
    }
}

pub fn save_report(report: &ExecutionReport, report_dir: &str) -> Result<String, String> {
    let date = Local::now().format("%Y-%m-%d");
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
```

---

## 8. UI 设计要点

### 8.1 布局
- 左侧导航栏（Home、Config、Execute、History）
- 主内容区域
- 顶部状态栏（显示当前项目、连接状态）

### 8.2 执行页布局
```
┌─────────────────────────────────────────┐
│  Step 1: 选择 Git 项目                   │
│  [扫描按钮] [手动输入路径]                │
│  [项目列表/卡片]                         │
├─────────────────────────────────────────┤
│  Step 2: 配置禅道项目                    │
│  [连接测试] [项目选择器]                  │
├─────────────────────────────────────────┤
│  Step 3: 预览信息                        │
│  [Commit 列表] [分支统计] [类型分布]      │
├─────────────────────────────────────────┤
│  Step 4: 执行                            │
│  [创建禅道任务按钮] [进度条]              │
│  [结果展示] [报告链接]                    │
└─────────────────────────────────────────┘
```

### 8.3 配色
- 主色调：#18A058（禅道绿）
- 背景：#FAFAFA
- 文字：#333333 / #666666
- 成功：#18A058
- 失败：#D03050
- 警告：#F5A623

---

## 9. 配置持久化

### 9.1 配置文件位置
- Windows: `%APPDATA%\Commit2Zen\config.json`
- macOS: `~/Library/Application Support/Commit2Zen/config.json`
- Linux: `~/.config/commit2zen/config.json`

### 9.2 Tauri 路径 API
```rust
let config_path = app_handle
    .path()
    .app_config_dir()
    .unwrap()
    .join("config.json");
```

---

## 10. 错误处理策略

| 错误类型 | 处理方式 | 用户提示 |
|----------|----------|----------|
| Git 仓库无效 | 返回错误 | "请选择有效的 Git 仓库" |
| 禅道连接失败 | 重试 3 次 | "无法连接禅道服务器" |
| 认证失败 | 立即返回 | "账号或密码错误" |
| 任务创建失败 | 记录错误，继续其他 | "分支 xxx 任务创建失败: 原因" |
| 报告写入失败 | 警告 | "报告保存失败，数据已在内存中" |

---

## 11. 安装与构建

### 11.1 开发环境
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Tauri CLI
cargo install tauri-cli

# 安装前端依赖
npm install

# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 11.2 打包输出
- Windows: `.msi` 安装包 + `.exe` 可执行文件
- macOS: `.app` + `.dmg`
- Linux: `.deb` + `.AppImage`

---

## 12. 迁移策略

### 12.1 阶段划分
1. **阶段 1**: 搭建 Tauri + Vue 3 项目骨架
2. **阶段 2**: 实现 Rust 核心服务（Git、禅道、报告）
3. **阶段 3**: 实现 Vue 前端组件
4. **阶段 4**: 集成测试与 UI 优化
5. **阶段 5**: 构建发布

### 12.2 兼容性
- 保留 `commit2zen.mjs` 作为 CLI 备用入口
- 配置文件格式保持一致（JSON）
- 报告格式完全兼容

---

## 13. 成功标准

- [ ] Rust 核心逻辑与现有 JS 行为 100% 等价
- [ ] GUI 可完成完整工作流（选项目 → 预览 → 创建任务 → 查看报告）
- [ ] Git 项目扫描正常工作
- [ ] 禅道 API 集成通过测试
- [ ] 报告格式与现有兼容
- [ ] 安装包可正常安装运行
- [ ] 无内存泄漏或崩溃

---

## 14. 风险评估

| 风险 | 影响 | 缓解 |
|------|------|------|
| Rust 学习曲线 | 开发进度 | 现有 JS 逻辑清晰，Rust 等价实现不难 |
| git2 crate 兼容性 | 部分 Git 操作可能行为差异 | 保留 shell 命令作为 fallback |
| Tauri 2.x 成熟度 | API 可能变化 | 使用稳定版本，关注更新日志 |
| 禅道 API 变化 | 任务创建失败 | 保持与当前禅道版本对齐，添加版本检测 |
