use tauri::command;
use crate::models::AppConfig;
use std::fs;

#[command]
pub async fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置失败: {}", e))?;
        
        // 尝试解析为新格式
        let config: Result<AppConfig, _> = serde_json::from_str(&content);
        
        match config {
            Ok(c) => Ok(c),
            Err(_) => {
                // 尝试解析旧格式并迁移
                migrate_old_config(&content)
            }
        }
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
    if config.zentao_accounts.is_empty() {
        return Err("至少需要配置一个禅道账号".to_string());
    }
    
    // 验证每个账号的必填字段
    for (i, account) in config.zentao_accounts.iter().enumerate() {
        if account.name.is_empty() {
            return Err(format!("第 {} 个账号的名称不能为空", i + 1));
        }
        if account.url.is_empty() {
            return Err(format!("账号 '{}' 的地址不能为空", account.name));
        }
        if account.account.is_empty() {
            return Err(format!("账号 '{}' 的账号不能为空", account.name));
        }
        if account.password.is_empty() {
            return Err(format!("账号 '{}' 的密码不能为空", account.name));
        }
    }
    
    Ok(true)
}

/// 迁移旧配置格式
fn migrate_old_config(content: &str) -> Result<AppConfig, String> {
    use serde_json::Value;
    use crate::models::ZentaoAccount;
    
    let json: Value = serde_json::from_str(content)
        .map_err(|e| format!("解析旧配置失败: {}", e))?;
    
    // 检查是否有旧的 zentao 字段
    if let Some(zentao) = json.get("zentao") {
        // 提取旧字段
        let url = zentao.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let account = zentao.get("account").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let password = zentao.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let assigned_to = zentao.get("assigned_to").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let task_type = zentao.get("task_type").and_then(|v| v.as_str()).unwrap_or("dev").to_string();
        
        // 创建新账号
        let new_account = ZentaoAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: if !url.is_empty() {
                // 从 URL 提取域名作为名称
                url.split("://").nth(1).unwrap_or(&url).split('/').next().unwrap_or("默认禅道").to_string()
            } else {
                "默认禅道".to_string()
            },
            url,
            account,
            password,
            assigned_to,
            task_type,
        };
        
        // 解析其他配置
        let git = json.get("git").and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let output = json.get("output").and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        
        let new_config = AppConfig {
            zentao_accounts: vec![new_account],
            git,
            output,
        };
        
        // 保存新格式
        let _ = save_config_sync(&new_config);
        
        eprintln!("[配置] 已自动迁移旧配置格式为新格式");
        Ok(new_config)
    } else {
        Err("配置格式无法识别".to_string())
    }
}

/// 同步保存配置（用于迁移）
fn save_config_sync(config: &AppConfig) -> Result<(), String> {
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

fn get_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Commit2Zen")
        .join("config.json")
}
