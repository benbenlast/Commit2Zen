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

    eprintln!("[禅道] 项目列表原始响应: {}", json);

    let mut projects = Vec::new();
    // 禅道 API 返回的是 {"projects": [...], "total": N, ...}
    if let Some(data) = json.get("projects").and_then(|d| d.as_array()) {
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
    } else {
        eprintln!("[禅道] 未找到 projects 字段或不是数组");
    }

    eprintln!("[禅道] 解析到 {} 个项目", projects.len());
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

    eprintln!("[禅道] 创建任务 payload: {:?}", task_data);

    let response = client
        .post(&format!("{}/api.php/v1/tasks", url))
        .header("Token", token)
        .json(task_data)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    eprintln!("[禅道] 创建任务响应: {}", json);

    // 禅道不同版本返回的字段名不同：
    // 旧版本: taskID
    // 新版本: id (直接返回任务对象)
    let task_id = json.get("taskID").and_then(|v| v.as_u64())
        .or_else(|| json.get("id").and_then(|v| v.as_u64()));

    if let Some(task_id) = task_id {
        Ok(TaskCreateResult {
            task_id: task_id as u32,
            task_url: format!("{}/task-view-{}.html", url, task_id),
        })
    } else {
        Err(format!("创建失败: {}", json))
    }
}

pub fn build_task_description(branch: &str, commits: &[Commit]) -> String {
    let mut desc = String::new();

    desc.push_str("<h3>提交记录</h3>\n");
    desc.push_str("<table border='1' cellpadding='4' cellspacing='0' style='border-collapse:collapse;width:100%;'>\n");
    desc.push_str("<tr style='background:#f0f0f0;'>\n");
    desc.push_str("<th>作者</th><th>日期</th><th>提交信息</th>\n");
    desc.push_str("</tr>\n");

    for commit in commits {
        let date = commit.date.split(' ').next().unwrap_or(&commit.date);
        desc.push_str("<tr>\n");
        desc.push_str(&format!("<td>{}</td>\n", commit.author));
        desc.push_str(&format!("<td>{}</td>\n", date));
        // 转义 HTML 特殊字符
        let msg = commit.message
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        desc.push_str(&format!("<td>{}</td>\n", msg));
        desc.push_str("</tr>\n");
    }

    desc.push_str("</table>\n");
    desc.push_str(&format!("<p><strong>提交数</strong>: {} | <strong>分支</strong>: {}</p>\n", commits.len(), branch));

    desc
}

pub fn build_task_payload(
    branch: &str,
    commits: &[Commit],
    project_id: u32,
    assigned_to: &str,
    task_type: &str,
) -> TaskPayload {
    // 从第一条提交提取作者，用于任务名称
    let author = commits.first().map(|c| c.author.as_str()).unwrap_or("未知");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    TaskPayload {
        name: format!("[{}] {} - 开发任务 ({})", branch, author, date),
        desc: build_task_description(branch, commits),
        project: project_id,
        assigned_to: assigned_to.to_string(),
        task_type: task_type.to_string(),
        est_started: date.clone(),
        deadline: date,
    }
}
