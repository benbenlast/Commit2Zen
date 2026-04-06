#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;
mod models;

use models::{AppConfig, BranchReport};
use services::{collect_commits, group_by_branches, login, create_task, build_task_payload, generate_report, save_report};

#[tauri::command]
async fn execute_full_workflow(
    config: AppConfig,
    project_path: String,
) -> Result<crate::models::ExecutionReport, String> {
    // 1. 收集 Git 提交
    let commits = collect_commits(&project_path, config.git.max_commits)?;

    // 2. 按分支分组
    let branch_groups = group_by_branches(commits, Some(&config.git.branch_pattern));

    // 3. 禅道登录
    let token = login(&config.zentao.url, &config.zentao.account, &config.zentao.password).await?;

    // 4. 为每个分支创建任务
    let mut branch_reports = Vec::new();
    for group in &branch_groups {
        let task_data = build_task_payload(
            &group.branch,
            &group.commits,
            config.zentao.project_id,
            &config.zentao.assigned_to,
            &config.zentao.task_type,
        );

        match create_task(&config.zentao.url, &token, &task_data).await {
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
    let _ = save_report(&report, &config.output.report_dir);

    Ok(report)
}

fn main() {
    tauri::Builder::default()
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
            execute_full_workflow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
