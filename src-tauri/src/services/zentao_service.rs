use reqwest::Client;
use crate::models::{TaskPayload, TaskCreateResult, ZentaoProject, Commit};

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

pub fn build_task_description(branch: &str, commits: &[Commit]) -> String {
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
    commits: &[Commit],
    project_id: u32,
    assigned_to: &str,
    task_type: &str,
) -> TaskPayload {
    TaskPayload {
        name: format!("[{}] 工作内容汇总", branch),
        desc: build_task_description(branch, commits),
        project: project_id,
        assigned_to: assigned_to.to_string(),
        task_type: task_type.to_string(),
        est_started: chrono::Local::now().format("%Y-%m-%d").to_string(),
        deadline: chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
}
