use serde::Serialize;

/// 扫描进度状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    /// 当前状态
    pub status: ScanStatus,
    /// 当前正在扫描的目录
    pub current_directory: String,
    /// 已发现的仓库数量
    pub repos_found: usize,
    /// 已扫描的目录数量
    pub directories_scanned: usize,
    /// 进度百分比 (0-100)
    pub percentage: f64,
    /// 最新发现的仓库信息（仅 Found 状态）
    pub repo: Option<RepoInfo>,
}

/// 仓库信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub path: String,
    pub name: String,
}

/// 扫描状态枚举
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus {
    Started,
    Scanning,
    Found,
    Completed,
    Cancelled,
    Error,
}
