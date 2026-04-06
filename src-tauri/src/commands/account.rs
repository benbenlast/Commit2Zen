use tauri::command;
use std::fs;
use crate::models::{AppConfig, ZentaoAccount};
use crate::services::zentao_service::login;

fn get_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Commit2Zen")
        .join("config.json")
}

fn load_config() -> Result<AppConfig, String> {
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

fn save_config(config: &AppConfig) -> Result<(), String> {
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

/// 获取所有禅道账号
#[command]
pub async fn list_zentao_accounts() -> Result<Vec<ZentaoAccount>, String> {
    let config = load_config()?;
    Ok(config.zentao_accounts)
}

/// 添加禅道账号
#[command]
pub async fn add_zentao_account(account: ZentaoAccount) -> Result<ZentaoAccount, String> {
    let mut config = load_config()?;
    
    // 检查 ID 是否已存在
    if config.zentao_accounts.iter().any(|a| a.id == account.id) {
        return Err(format!("账号 ID 已存在: {}", account.id));
    }
    
    config.zentao_accounts.push(account.clone());
    save_config(&config)?;
    
    Ok(account)
}

/// 更新禅道账号
#[command]
pub async fn update_zentao_account(account: ZentaoAccount) -> Result<ZentaoAccount, String> {
    let mut config = load_config()?;
    
    // 查找并更新
    let found = config.zentao_accounts.iter_mut()
        .find(|a| a.id == account.id);
    
    if let Some(existing) = found {
        *existing = account.clone();
        save_config(&config)?;
        Ok(account)
    } else {
        Err(format!("未找到账号: {}", account.id))
    }
}

/// 删除禅道账号
#[command]
pub async fn delete_zentao_account(id: String) -> Result<(), String> {
    let mut config = load_config()?;
    
    let len_before = config.zentao_accounts.len();
    config.zentao_accounts.retain(|a| a.id != id);
    
    if config.zentao_accounts.len() == len_before {
        return Err(format!("未找到账号: {}", id));
    }
    
    save_config(&config)?;
    Ok(())
}

/// 测试禅道连接
#[command]
pub async fn test_zentao_connection(
    url: String,
    account: String,
    password: String,
) -> Result<String, String> {
    login(&url, &account, &password).await
}
