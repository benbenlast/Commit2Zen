use tauri::command;
use crate::models::{ExecutionReport, BranchReport, ReportSummary, ReportMeta};
use crate::services::{generate_report as gen_report, save_report as save_rep, get_history};

#[command]
pub async fn generate_report(branches: Vec<BranchReport>, _summary: Option<ReportSummary>) -> Result<ExecutionReport, String> {
    Ok(gen_report(branches))
}

#[command]
pub async fn save_report_cmd(report: ExecutionReport, report_dir: String) -> Result<String, String> {
    save_rep(&report, &report_dir)
}

#[command]
pub async fn get_report_history(report_dir: String) -> Result<Vec<ReportMeta>, String> {
    get_history(&report_dir)
}
