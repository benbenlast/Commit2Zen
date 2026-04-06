use tauri::command;
use crate::models::{ZentaoProject, TaskCreateResult, Commit};
use crate::services::{login, get_projects, create_task, build_task_payload};

#[command]
pub async fn zentao_login(url: String, account: String, password: String) -> Result<String, String> {
    login(&url, &account, &password).await
}

#[command]
pub async fn zentao_get_projects(url: String, token: String) -> Result<Vec<ZentaoProject>, String> {
    get_projects(&url, &token).await
}

#[command]
pub async fn zentao_create_task(
    url: String,
    token: String,
    branch: String,
    commits: Vec<Commit>,
    project_id: u32,
    assigned_to: String,
    task_type: String,
) -> Result<TaskCreateResult, String> {
    let task_data = build_task_payload(&branch, &commits, project_id, &assigned_to, &task_type);
    create_task(&url, &token, &task_data).await
}
