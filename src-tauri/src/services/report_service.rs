use std::fs;
use crate::models::{ExecutionReport, BranchReport, ReportSummary, ReportMeta};

pub fn generate_report(branches: Vec<BranchReport>) -> ExecutionReport {
    let total_commits: usize = branches.iter().map(|b| b.commit_count).sum();
    let tasks_created = branches.iter().filter(|b| b.task_created).count();
    let tasks_failed = branches.iter().filter(|b| !b.task_created).count();

    ExecutionReport {
        timestamp: chrono::Local::now().to_rfc3339(),
        project: "Commit2Zen".to_string(),
        branches,
        summary: ReportSummary {
            total_branches: branches.len(),
            total_commits,
            tasks_created,
            tasks_failed,
        },
    }
}

pub fn save_report(report: &ExecutionReport, report_dir: &str) -> Result<String, String> {
    let date = chrono::Local::now().format("%Y-%m-%d");
    let filename = format!("{}-report.json", date);
    let path = format!("{}/{}", report_dir, filename);

    fs::create_dir_all(report_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(path)
}

pub fn get_history(report_dir: &str) -> Result<Vec<ReportMeta>, String> {
    let entries = fs::read_dir(report_dir)
        .map_err(|e| format!("读取目录失败: {}", e))?;

    let mut reports = Vec::new();
    for entry in entries.flatten() {
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if filename_str.ends_with("-report.json") {
            reports.push(ReportMeta {
                path: entry.path().to_string_lossy().to_string(),
                date: filename_str.replace("-report.json", ""),
                filename: filename_str.to_string(),
            });
        }
    }

    reports.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(reports)
}
