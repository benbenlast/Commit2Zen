use git2::{Repository, Sort};
use crate::models::{Commit, BranchGroup, BranchSummary, CommitTypes, DateRange, DateFilter};
use std::collections::{HashMap, HashSet};
use regex::Regex;

pub fn collect_commits(repo_path: &str, max_commits: usize, date_filter: Option<DateFilter>) -> Result<Vec<Commit>, String> {
    eprintln!("[collect_commits] 开始收集: path={}, max_commits={:?}, date_filter={:?}", repo_path, max_commits, date_filter);
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("无法打开仓库: {}", e))?;

    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("无法创建revwalk: {}", e))?;

    revwalk.push_head()
        .map_err(|e| format!("无法push HEAD: {}", e))?;

    revwalk.set_sorting(Sort::TIME)
        .map_err(|e| format!("无法设置排序: {}", e))?;

    let mut commits = Vec::new();
    for oid in revwalk.take(max_commits) {
        let oid = oid.map_err(|e| format!("遍历提交失败: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("查找提交失败: {}", e))?;

        let hash = format!("{:.7}", commit.id());
        let author = commit.author()
            .name()
            .unwrap_or("unknown")
            .to_string();

        let timestamp = commit.time().seconds();

        // 日期范围筛选
        if let Some(filter) = &date_filter {
            if let Some(start) = filter.start {
                if timestamp < start {
                    continue;
                }
            }
            if let Some(end) = filter.end {
                if timestamp > end {
                    continue;
                }
            }
        }

        let date = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());

        let message = commit.message()
            .unwrap_or("")
            .trim_end()
            .to_string();

        let branches = get_commit_branches(&repo, &commit);
        let files = get_commit_files(&repo, &commit);

        commits.push(Commit {
            hash,
            author,
            date,
            message,
            branches,
            files,
        });
    }

    Ok(commits)
}

fn get_commit_branches(repo: &Repository, commit: &git2::Commit) -> Vec<String> {
    let mut branches = Vec::new();

    if let Ok(heads) = repo.branches(Some(git2::BranchType::Local)) {
        for head in heads.filter_map(|r| r.ok()) {
            if let (Some(branch_name), Some(ref_head)) = (
                head.0.name().ok().flatten(),
                head.0.get().peel_to_commit().ok()
            ) {
                if ref_head.id() == commit.id() {
                    branches.push(branch_name.to_string());
                }
            }
        }
    }

    if branches.is_empty() {
        // 尝试从 HEAD 判断
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                branches.push(name.to_string());
            }
        }
    }

    branches
}

fn get_commit_files(repo: &Repository, commit: &git2::Commit) -> Vec<String> {
    if let Some(parent) = commit.parent(0).ok() {
        if let (Ok(tree), Ok(parent_tree)) = (commit.tree(), parent.tree()) {
            if let Ok(diff) = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None) {
                let files: Vec<String> = diff.deltas()
                    .filter_map(|delta| delta.new_file().path())
                    .filter_map(|p: &std::path::Path| p.to_str().map(String::from))
                    .collect();
                return files;
            }
        }
    }

    // 初始提交，返回所有文件
    if commit.parent_count() == 0 {
        if let Ok(tree) = commit.tree() {
            let mut files: Vec<String> = Vec::new();
            tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
                if let Some(name) = entry.name() {
                    files.push(name.to_string());
                }
                git2::TreeWalkResult::Ok
            }).ok();
            return files;
        }
    }

    Vec::new()
}

pub fn group_by_branches(commits: Vec<Commit>, branch_pattern: Option<&str>) -> Vec<BranchGroup> {
    eprintln!("[group_by_branches] 收到 {} 条提交, pattern={:?}", commits.len(), branch_pattern);
    let pattern = branch_pattern
        .and_then(|p| Regex::new(p).ok())
        .unwrap_or_else(|| Regex::new(".*").unwrap());

    let mut groups: HashMap<String, Vec<Commit>> = HashMap::new();

    for commit in &commits {
        eprintln!("[group_by_branches] 提交 {} branches={:?}", commit.hash, commit.branches);
    }

    for commit in commits {
        let branch_name = commit.branches
            .iter()
            .find(|b| pattern.is_match(b.as_str()))
            .cloned()
            .unwrap_or_else(|| "未分类".to_string());

        groups.entry(branch_name).or_default().push(commit);
    }

    groups.into_iter().map(|(branch, branch_commits)| {
        let authors: Vec<String> = branch_commits.iter()
            .map(|c| c.author.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let all_files: Vec<String> = branch_commits.iter()
            .flat_map(|c| c.files.clone())
            .collect();

        let total_files: usize = branch_commits.iter().map(|c| c.files.len()).sum();

        let top_files = {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for f in &all_files {
                *counts.entry(f.clone()).or_insert(0) += 1;
            }
            let mut sorted: Vec<_> = counts.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            sorted.into_iter().take(10).map(|(f, _)| f).collect()
        };

        let dates: Vec<&String> = branch_commits.iter().map(|c| &c.date).collect();
        let date_range = if dates.is_empty() {
            DateRange { start: String::new(), end: String::new() }
        } else {
            DateRange {
                start: dates.last().unwrap().to_string(),
                end: dates.first().unwrap().to_string(),
            }
        };

        let commit_types = classify_commits(&branch_commits);

        BranchGroup {
            branch,
            commit_count: branch_commits.len(),
            authors,
            date_range,
            commits: branch_commits,
            summary: BranchSummary {
                total_commits: 0,
                total_authors: 0,
                total_files,
                commit_types,
                top_files,
            },
        }
    }).collect()
}

fn classify_commits(commits: &[Commit]) -> CommitTypes {
    let mut types = CommitTypes {
        feat: 0, fix: 0, docs: 0, refactor: 0,
        test: 0, chore: 0, other: 0,
    };

    for commit in commits {
        let msg = commit.message.to_lowercase();
        if msg.starts_with("feat") { types.feat += 1; }
        else if msg.starts_with("fix") { types.fix += 1; }
        else if msg.starts_with("docs") { types.docs += 1; }
        else if msg.starts_with("refactor") { types.refactor += 1; }
        else if msg.starts_with("test") { types.test += 1; }
        else if msg.starts_with("chore") { types.chore += 1; }
        else { types.other += 1; }
    }

    types
}
