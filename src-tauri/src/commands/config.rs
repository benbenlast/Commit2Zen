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
