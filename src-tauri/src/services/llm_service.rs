use reqwest::Client;
use crate::models::{Message, LLMResponse, TokenUsage};

/// 调用 OpenAI 兼容 API
pub async fn call_openai_compatible(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match try_call_openai(api_key, base_url, model, messages, timeout_ms).await {
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

async fn try_call_openai(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let client = build_client(timeout_ms);

    // 构建 OpenAI 格式的消息
    let openai_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            let role = match m.role {
                crate::models::MessageRole::System => "system",
                crate::models::MessageRole::User => "user",
                crate::models::MessageRole::Assistant => "assistant",
            };
            serde_json::json!({
                "role": role,
                "content": m.content
            })
        })
        .collect();

    eprintln!("[LLM] OpenAI 调用: model={}, url={}/chat/completions", model, base_url.trim_end_matches('/'));

    let response = client
        .post(&format!("{}/chat/completions", base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "temperature": 0.7,
        }))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("请求失败 ({}): {}", status, error_body));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[LLM] OpenAI 响应: {}", json);

    // 提取内容
    let content = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err("响应内容为空".to_string());
    }

    // 提取 usage
    let usage = json.get("usage").map(|u| TokenUsage {
        prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
    });

    let response_model = json.get("model").and_then(|v| v.as_str()).unwrap_or(model).to_string();

    Ok(LLMResponse {
        content,
        model: response_model,
        provider: "openai".to_string(),
        usage,
    })
}

/// 调用 Claude API
pub async fn call_claude(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match try_call_claude(api_key, base_url, model, messages, timeout_ms).await {
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

async fn try_call_claude(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let client = build_client(timeout_ms);

    // 提取 system prompt 和分离 messages
    let mut system_prompt = None;
    let claude_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter_map(|m| {
            let (role, content) = match m.role {
                crate::models::MessageRole::System => {
                    system_prompt = Some(m.content.clone());
                    return None;
                }
                crate::models::MessageRole::User => ("user", m.content.clone()),
                crate::models::MessageRole::Assistant => ("assistant", m.content.clone()),
            };
            Some(serde_json::json!({
                "role": role,
                "content": content
            }))
        })
        .collect();

    let base = base_url.trim_end_matches('/');
    eprintln!("[LLM] Claude 调用: model={}, url={}/messages", model, base);

    let mut request_body = serde_json::json!({
        "model": model,
        "messages": claude_messages,
        "max_tokens": 4096,
    });

    if let Some(sys) = system_prompt {
        request_body["system"] = serde_json::json!(sys);
    }

    let response = client
        .post(&format!("{}/messages", base))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("请求失败 ({}): {}", status, error_body));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[LLM] Claude 响应: {}", json);

    // 提取内容
    let content = json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|block| block.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err("响应内容为空".to_string());
    }

    // Claude 不直接返回 usage 的标准格式，尝试提取
    let usage = json.get("usage").map(|u| TokenUsage {
        prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        total_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
            + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
    });

    Ok(LLMResponse {
        content,
        model: model.to_string(),
        provider: "claude".to_string(),
        usage,
    })
}

/// 调用 Gemini API
pub async fn call_gemini(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match try_call_gemini(api_key, base_url, model, messages, timeout_ms).await {
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

async fn try_call_gemini(
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let client = build_client(timeout_ms);

    // 构建 Gemini 格式的内容
    let contents: Vec<serde_json::Value> = messages
        .iter()
        .filter_map(|m| {
            // Gemini 使用 user/model 角色
            let role = match m.role {
                crate::models::MessageRole::System => return None, // system 放在 system_instruction 中
                crate::models::MessageRole::User => "user",
                crate::models::MessageRole::Assistant => "model",
            };
            Some(serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }]
            }))
        })
        .collect();

    // 提取 system prompt
    let system_instruction = messages
        .iter()
        .find(|m| matches!(m.role, crate::models::MessageRole::System))
        .map(|m| serde_json::json!({ "parts": [{ "text": &m.content }] }));

    let base = base_url.trim_end_matches('/');
    eprintln!("[LLM] Gemini 调用: model={}, url={}/models/{}:generateContent", model, base, model);

    let mut request_body = serde_json::json!({
        "contents": contents,
    });

    if let Some(sys) = system_instruction {
        request_body["system_instruction"] = sys;
    }

    let response = client
        .post(&format!("{}/models/{}:generateContent", base, model))
        .query(&[("key", api_key)])
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("请求失败 ({}): {}", status, error_body));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[LLM] Gemini 响应: {}", json);

    // 提取内容
    let content = json
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err("响应内容为空".to_string());
    }

    // 提取 usage metadata
    let usage = json.get("usageMetadata").map(|u| TokenUsage {
        prompt_tokens: u.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
        completion_tokens: u.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
        total_tokens: u.get("totalTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
    });

    Ok(LLMResponse {
        content,
        model: model.to_string(),
        provider: "gemini".to_string(),
        usage,
    })
}

/// 调用 Ollama API
pub async fn call_ollama(
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let max_retries = 2;
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match try_call_ollama(base_url, model, messages, timeout_ms).await {
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

async fn try_call_ollama(
    base_url: &str,
    model: &str,
    messages: &[Message],
    timeout_ms: Option<u64>,
) -> Result<LLMResponse, String> {
    let client = build_client(timeout_ms);

    // 构建 Ollama 格式的消息
    let ollama_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            let role = match m.role {
                crate::models::MessageRole::System => "system",
                crate::models::MessageRole::User => "user",
                crate::models::MessageRole::Assistant => "assistant",
            };
            serde_json::json!({
                "role": role,
                "content": m.content
            })
        })
        .collect();

    let base = base_url.trim_end_matches('/');
    eprintln!("[LLM] Ollama 调用: model={}, url={}/api/chat", model, base);

    let response = client
        .post(&format!("{}/api/chat", base))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": ollama_messages,
            "stream": false,
        }))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("请求失败 ({}): {}", status, error_body));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[LLM] Ollama 响应: {}", json);

    // 提取内容
    let content = json
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err("响应内容为空".to_string());
    }

    // Ollama 返回的 done_reason 和 total_duration 等信息
    let usage = None; // Ollama 本地部署通常不提供 token 统计

    Ok(LLMResponse {
        content,
        model: model.to_string(),
        provider: "ollama".to_string(),
        usage,
    })
}

/// 构建 HTTP 客户端，支持超时配置
fn build_client(timeout_ms: Option<u64>) -> Client {
    let mut builder = Client::builder();
    if let Some(ms) = timeout_ms {
        builder = builder.timeout(std::time::Duration::from_millis(ms));
    }
    builder.build().unwrap_or_else(|_| Client::new())
}

/// 加载 LLM 配置
pub async fn load_llm_config() -> Result<crate::models::LLMConfig, String> {
    let config_path = get_llm_config_path();
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取 LLM 配置失败: {}", e))?;

        let config: Result<crate::models::LLMConfig, _> = serde_json::from_str(&content);
        match config {
            Ok(c) => Ok(c),
            Err(e) => {
                eprintln!("[LLM] 解析配置文件失败: {}，使用默认配置", e);
                Ok(crate::models::LLMConfig::default())
            }
        }
    } else {
        Ok(crate::models::LLMConfig::default())
    }
}

/// 保存 LLM 配置
pub async fn save_llm_config(config: &crate::models::LLMConfig) -> Result<(), String> {
    let config_path = get_llm_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化 LLM 配置失败: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("写入 LLM 配置失败: {}", e))?;

    eprintln!("[LLM] 配置已保存到: {:?}", config_path);
    Ok(())
}

fn get_llm_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Commit2Zen")
        .join("llm_config.json")
}
