use tauri::command;
use crate::models::{Message, LLMResponse, LLMConfig};
use crate::services::{
    call_openai_compatible, call_claude, call_gemini, call_ollama,
    load_llm_config, save_llm_config,
};

/// 调用 OpenAI 兼容 API
#[command]
pub async fn llm_call_openai(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    timeout: Option<u64>,
) -> Result<LLMResponse, String> {
    call_openai_compatible(&api_key, &base_url, &model, &messages, timeout).await
}

/// 调用 Claude API
#[command]
pub async fn llm_call_claude(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    timeout: Option<u64>,
) -> Result<LLMResponse, String> {
    call_claude(&api_key, &base_url, &model, &messages, timeout).await
}

/// 调用 Gemini API
#[command]
pub async fn llm_call_gemini(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    timeout: Option<u64>,
) -> Result<LLMResponse, String> {
    call_gemini(&api_key, &base_url, &model, &messages, timeout).await
}

/// 调用 Ollama API
#[command]
pub async fn llm_call_ollama(
    base_url: String,
    model: String,
    messages: Vec<Message>,
    timeout: Option<u64>,
) -> Result<LLMResponse, String> {
    call_ollama(&base_url, &model, &messages, timeout).await
}

/// 加载 LLM 配置
#[command]
pub async fn llm_load_config() -> Result<LLMConfig, String> {
    load_llm_config().await
}

/// 保存 LLM 配置
#[command]
pub async fn llm_save_config(config: LLMConfig) -> Result<(), String> {
    save_llm_config(&config).await
}

/// 测试 LLM 提供商连接
#[command]
pub async fn llm_test_connection(
    provider_type: String,
    api_key: Option<String>,
    base_url: String,
    model: String,
    timeout: Option<u64>,
) -> Result<bool, String> {
    let test_messages = vec![Message {
        role: crate::models::MessageRole::User,
        content: "Hi".to_string(),
    }];

    match provider_type.as_str() {
        "openai" => {
            let api_key = api_key.ok_or_else(|| "OpenAI 需要 API Key".to_string())?;
            let result = call_openai_compatible(&api_key, &base_url, &model, &test_messages, timeout).await;
            Ok(result.is_ok())
        }
        "claude" => {
            let api_key = api_key.ok_or_else(|| "Claude 需要 API Key".to_string())?;
            let result = call_claude(&api_key, &base_url, &model, &test_messages, timeout).await;
            Ok(result.is_ok())
        }
        "gemini" => {
            let api_key = api_key.ok_or_else(|| "Gemini 需要 API Key".to_string())?;
            let result = call_gemini(&api_key, &base_url, &model, &test_messages, timeout).await;
            Ok(result.is_ok())
        }
        "ollama" => {
            let result = call_ollama(&base_url, &model, &test_messages, timeout).await;
            Ok(result.is_ok())
        }
        _ => Err(format!("不支持的提供商类型: {}", provider_type)),
    }
}
