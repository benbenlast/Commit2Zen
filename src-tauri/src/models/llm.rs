use serde::{Serialize, Deserialize};

/// LLM 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LLMProviderType {
    OpenAI,
    Claude,
    Gemini,
    Ollama,
}

impl std::fmt::Display for LLMProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderType::OpenAI => write!(f, "openai"),
            LLMProviderType::Claude => write!(f, "claude"),
            LLMProviderType::Gemini => write!(f, "gemini"),
            LLMProviderType::Ollama => write!(f, "ollama"),
        }
    }
}

/// 聊天消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProvider {
    /// 提供商类型
    #[serde(rename = "type")]
    pub provider_type: LLMProviderType,
    /// 是否启用
    pub enabled: bool,
    /// API Key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// API 基础 URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 请求超时时间（毫秒）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 最大重试次数
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// 额外配置
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

/// LLM 任务分配配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMTaskAssignment {
    /// 任务类型标识
    pub task_type: String,
    /// 使用的提供商类型
    pub provider_type: LLMProviderType,
    /// 任务描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 系统提示词
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// 是否启用
    pub enabled: bool,
}

/// Token 使用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// LLM 调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// 响应内容
    pub content: String,
    /// 使用的模型
    pub model: String,
    /// 使用的提供商
    pub provider: String,
    /// Token 使用信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// 提供商配置映射
    pub providers: serde_json::Value,
    /// 任务分配规则
    pub task_assignments: Vec<LLMTaskAssignment>,
    /// 默认提供商
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    /// 全局最大重试次数
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// 请求间隔（毫秒）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_interval: Option<u64>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        LLMConfig {
            providers: serde_json::json!({
                "openai": {
                    "type": "openai",
                    "enabled": false,
                    "apiKey": "",
                    "baseUrl": "https://api.openai.com/v1",
                    "model": "gpt-4o-mini",
                    "timeout": 30000,
                    "maxRetries": 3
                },
                "claude": {
                    "type": "claude",
                    "enabled": false,
                    "apiKey": "",
                    "baseUrl": "https://api.anthropic.com/v1",
                    "model": "claude-3-5-sonnet-20241022",
                    "timeout": 60000,
                    "maxRetries": 3
                },
                "gemini": {
                    "type": "gemini",
                    "enabled": false,
                    "apiKey": "",
                    "baseUrl": "https://generativelanguage.googleapis.com/v1beta",
                    "model": "gemini-2.0-flash",
                    "timeout": 30000,
                    "maxRetries": 3
                },
                "ollama": {
                    "type": "ollama",
                    "enabled": false,
                    "baseUrl": "http://localhost:11434",
                    "model": "llama3.2",
                    "timeout": 120000,
                    "maxRetries": 2
                }
            }),
            task_assignments: Vec::new(),
            default_provider: Some("openai".to_string()),
            max_retries: Some(3),
            request_interval: Some(1000),
        }
    }
}
