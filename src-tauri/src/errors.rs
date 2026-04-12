use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("仓库未找到: {0}")]
    RepositoryNotFound(String),

    #[error("Git 操作错误: {0}")]
    GitError(String),

    #[error("日期筛选错误: {0}")]
    DateFilterError(String),

    #[error("禅道 API 错误: status={status}, message={message}")]
    ZentaoApiError { status: u16, message: String },

    #[error("禅道连接失败: {0}")]
    ZentaoConnectionError(String),

    #[error("LLM 调用失败: {0}")]
    LlmError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("文件操作错误: {0}")]
    FileError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),
}

impl AppError {
    pub fn code(&self) -> &str {
        match self {
            Self::RepositoryNotFound(_) => "REPOSITORY_NOT_FOUND",
            Self::GitError(_) => "GIT_ERROR",
            Self::DateFilterError(_) => "DATE_FILTER_ERROR",
            Self::ZentaoApiError { .. } => "ZENTAO_API_ERROR",
            Self::ZentaoConnectionError(_) => "ZENTAO_CONNECTION_ERROR",
            Self::LlmError(_) => "LLM_ERROR",
            Self::ConfigError(_) => "CONFIG_ERROR",
            Self::FileError(_) => "FILE_ERROR",
            Self::NetworkError(_) => "NETWORK_ERROR",
        }
    }
}
