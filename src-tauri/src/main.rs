#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;
mod models;
mod errors;

use errors::AppError;

use models::{BranchReport, ZentaoAccount, GitConfig, OutputConfig, DateFilter};
use services::{collect_commits, group_by_branches, login, create_task, build_task_payload, generate_report, save_report};
use commands::git::ScanCancelMap;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[tauri::command]
async fn execute_full_workflow(
    account: ZentaoAccount,
    project_id: u32,
    project_path: String,
    git_config: GitConfig,
    output_config: OutputConfig,
    date_filter: Option<DateFilter>,
    ai_summaries: Option<std::collections::HashMap<String, String>>,
) -> Result<crate::models::ExecutionReport, String> {
    // 1. 收集 Git 提交
    let commits = collect_commits(&project_path, git_config.max_commits, date_filter)?;

    // 2. 按分支分组
    let branch_groups = group_by_branches(commits, Some(&git_config.branch_pattern));

    // 3. 禅道登录
    let token = login(&account.url, &account.account, &account.password).await?;

    // 4. 为每个分支创建任务
    let mut branch_reports = Vec::new();
    for group in &branch_groups {
        let ai_summary = ai_summaries.as_ref().and_then(|map| map.get(&group.branch).map(|s| s.as_str()));
        let task_data = build_task_payload(
            &group.branch,
            &group.commits,
            project_id,
            &account.assigned_to,
            &account.task_type,
            ai_summary,
        );

        match create_task(&account.url, &token, &task_data).await {
            Ok(result) => {
                branch_reports.push(BranchReport {
                    branch: group.branch.clone(),
                    commit_count: group.commit_count,
                    task_created: true,
                    task_id: Some(result.task_id),
                    task_url: Some(result.task_url),
                    error: None,
                });
            }
            Err(e) => {
                branch_reports.push(BranchReport {
                    branch: group.branch.clone(),
                    commit_count: group.commit_count,
                    task_created: false,
                    task_id: None,
                    task_url: None,
                    error: Some(e),
                });
            }
        }
    }

    // 5. 生成报告
    let report = generate_report(branch_reports);

    // 6. 保存报告
    let _ = save_report(&report, &output_config.report_dir);

    Ok(report)
}

fn main() {
    let cancel_map: ScanCancelMap = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(cancel_map)
        .invoke_handler(tauri::generate_handler![
            commands::scan_git_repositories,
            commands::collect_git_log,
            commands::group_commits_by_branch,
            commands::zentao_login,
            commands::zentao_get_projects,
            commands::zentao_create_task,
            commands::load_config,
            commands::save_config,
            commands::validate_config,
            commands::generate_report,
            commands::save_report_cmd,
            commands::get_report_history,
            commands::read_report,
            commands::start_folder_scan,
            commands::cancel_scan,
            commands::list_zentao_accounts,
            commands::add_zentao_account,
            commands::update_zentao_account,
            commands::delete_zentao_account,
            commands::test_zentao_connection,
            commands::llm_call_openai,
            commands::llm_call_claude,
            commands::llm_call_gemini,
            commands::llm_call_ollama,
            commands::llm_load_config,
            commands::llm_save_config,
            commands::llm_test_connection,
            execute_full_workflow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
