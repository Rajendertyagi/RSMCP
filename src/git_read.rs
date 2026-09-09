//! Git read-only tools.
//! Adapted from winfs-mcp (MIT). All tools are read-only; mutation flags are denied.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GitStatusRequest {
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusOutput {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
    pub detached: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GitLogRequest {
    pub path: PathBuf,
    #[serde(default = "default_count")]
    pub count: i64,
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default)]
    pub path_filter: Option<String>,
}

fn default_count() -> i64 { 20 }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLogEntry {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub subject: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLogOutput {
    pub entries: Vec<GitLogEntry>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GitShowRequest {
    pub path: PathBuf,
    pub sha: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitShowOutput {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub body: String,
    pub diff: String,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GitDiffRequest {
    pub path: PathBuf,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub path_filter: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GitBlameRequest {
    pub path: PathBuf,
    pub file: PathBuf,
    #[serde(default = "default_blame_lines")]
    pub max_lines: i64,
}

fn default_blame_lines() -> i64 { 1000 }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBlameEntry {
    pub line_number: usize,
    pub sha: String,
    pub author: String,
    pub date: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBlameOutput {
    pub file: String,
    pub total_lines: usize,
    pub entries: Vec<GitBlameEntry>,
}

pub fn git_status(path: &Path) -> Result<GitStatusOutput, String> {
    // Check if path is inside a git repo
    let output = cmd(&["rev-parse", "--show-toplevel"], path)?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

    // Branch
    let branch_out = cmd(&["rev-parse", "--abbrev-ref", "HEAD"], &repo_root)?;
    let branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
    let detached = branch == "HEAD";

    // Ahead/behind
    let merge_base_out = cmd(&["merge-base", "HEAD", "FETCH_HEAD"], &repo_root);
    let (ahead, behind) = match merge_base_out {
        Ok(mb) => {
            let mb_sha = String::from_utf8_lossy(&mb.stdout).trim().to_string();
            let log_ahead = cmd(&["rev-list", "--count", format!("{}..HEAD", mb_sha).as_str()], &repo_root);
            let log_behind = cmd(&["rev-list", "--count", format!("HEAD..{}", mb_sha).as_str()], &repo_root);
            match (log_ahead, log_behind) {
                (Ok(a), Ok(b)) => (
                    String::from_utf8_lossy(&a.stdout).trim().parse().unwrap_or(0),
                    String::from_utf8_lossy(&b.stdout).trim().parse().unwrap_or(0),
                ),
                _ => (0, 0),
            }
        }
        Err(_) => (0, 0),
    };

    // Status in porcelain v2 format
    let status_out = cmd(&["status", "--porcelain=v2", "-z"], &repo_root);
    let (mut staged, mut modified, mut untracked, mut conflicted) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    if let Ok(s) = status_out {
        let status_text = String::from_utf8_lossy(&s.stdout);
        for line in status_text.split('\0') {
            let line = line.trim();
            if line.is_empty() || !line.starts_with("2 ") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let file = parts[2];
                let status_flags = parts[1];
                if status_flags.contains('U') {
                    conflicted.push(file.to_string());
                } else if status_flags.contains('A') || status_flags.contains('C') || status_flags.contains('D') || status_flags.contains('M') {
                    if status_flags.contains('R') || status_flags.contains('C') {
                        // Renamed or copied — in index
                        staged.push(file.to_string());
                    } else {
                        staged.push(file.to_string());
                    }
                } else if status_flags.contains('M') {
                    modified.push(file.to_string());
                } else if status_flags.contains('?') {
                    untracked.push(file.to_string());
                }
            }
        }
    }

    // Fallback to simpler format if v2 produced nothing useful
    if staged.is_empty() && modified.is_empty() && untracked.is_empty() {
        let simple_out = cmd(&["status", "--short"], &repo_root);
        if let Ok(s) = simple_out {
            for line in String::from_utf8_lossy(&s.stdout).lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                let flags = &line[..2];
                let file = line[3..].trim().to_string();
                if flags.contains('?') {
                    untracked.push(file);
                } else if flags.contains('A') || flags.contains('C') {
                    staged.push(file);
                } else {
                    modified.push(file);
                }
            }
        }
    }

    Ok(GitStatusOutput {
        branch,
        ahead,
        behind,
        staged,
        modified,
        untracked,
        conflicted,
        detached,
    })
}

pub fn git_log(request: &GitLogRequest) -> Result<GitLogOutput, String> {
    let output = cmd(&["rev-parse", "--show-toplevel"], &request.path)
        .map_err(|e| format!("Not a git repository: {}", e))?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

    let mut args = vec!["log".to_string()];

    // Range
    if let Some(range) = &request.range {
        validate_git_args(range)?;
        args.push(range.clone());
    } else {
        let count = std::cmp::min(request.count, 1000);
        args.push(format!("-n{}", count));
    }

    // Path filter (must come after --)
    if let Some(filter) = &request.path_filter {
        validate_git_args(filter)?;
        args.push("--".to_string());
        args.push(filter.clone());
    } else {
        args.push("--".to_string());
    }

    args.extend_from_slice(&[
        "--pretty=format:%H%n%an%n%ad%n%s".to_string(),
        "--date=short".to_string(),
    ]);

    let log_out = cmd(&args, &repo_root).map_err(|e| format!("git log failed: {}", e))?;
    let lines: Vec<&str> = String::from_utf8_lossy(&log_out.stdout).lines().collect();

    let mut entries = Vec::new();
    let mut i = 0;
    while i + 3 < lines.len() {
        entries.push(GitLogEntry {
            sha: lines[i].to_string(),
            author: lines[i + 1].to_string(),
            date: lines[i + 2].to_string(),
            subject: lines[i + 3].to_string(),
        });
        i += 4;
    }

    Ok(GitLogOutput {
        entries,
        total: entries.len(),
    })
}

pub fn git_show(request: &GitShowRequest) -> Result<GitShowOutput, String> {
    let output = cmd(&["rev-parse", "--show-toplevel"], &request.path)
        .map_err(|e| format!("Not a git repository: {}", e))?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

    // Validate sha: must be hex, 4-64 chars
    if !request.sha.chars().all(|c| c.is_ascii_hexdigit()) || request.sha.len() < 4 || request.sha.len() > 64 {
        return Err("sha must be 4-64 hex characters".to_string());
    }

    // Get metadata
    let meta_out = cmd(&[
        "show",
        "--pretty=format:SHA:%H%nAuthor:%an%nDate:%ad%nSubject:%s%nBody:%b",
        "--date=short",
        &request.sha,
        "--",
    ], &repo_root).map_err(|e| format!("git show failed: {}", e))?;

    let meta_text = String::from_utf8_lossy(&meta_out.stdout);
    let mut sha = String::new();
    let mut author = String::new();
    let mut date = String::new();
    let mut subject = String::new();
    let mut body = String::new();

    for line in meta_text.lines() {
        if line.starts_with("SHA:") { sha = line[4..].to_string(); }
        else if line.starts_with("Author:") { author = line[7..].to_string(); }
        else if line.starts_with("Date:") { date = line[5..].to_string(); }
        else if line.starts_with("Subject:") { subject = line[8..].to_string(); }
        else if line.starts_with("Body:") { body = line[5..].to_string(); }
    }

    // Get diff
    let diff_out = cmd(&["diff", "--numstat", &request.sha, "--"], &repo_root)
        .unwrap_or_else(|_| Command::new("git").args(&["diff", "--numstat", &request.sha, "--"])
            .current_dir(&repo_root).output().unwrap_or_default());

    let diff_text = String::from_utf8_lossy(&diff_out.stdout);
    let mut files_changed = Vec::new();
    let mut diff_content = String::new();

    for line in diff_text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            files_changed.push(parts[2].to_string());
        }
    }

    // Get full diff for display
    let full_diff = cmd(&["diff", &request.sha, "--"], &repo_root);
    if let Ok(d) = full_diff {
        diff_content = String::from_utf8_lossy(&d.stdout).to_string();
    }

    Ok(GitShowOutput {
        sha,
        author,
        date,
        subject,
        body,
        diff: diff_content,
        files_changed,
    })
}

pub fn git_diff(request: &GitDiffRequest) -> Result<String, String> {
    let output = cmd(&["rev-parse", "--show-toplevel"], &request.path)
        .map_err(|e| format!("Not a git repository: {}", e))?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

    let mut args = vec!["diff".to_string()];

    // Validate both revs
    validate_git_rev(&request.from)?;
    validate_git_rev(&request.to)?;

    args.push(request.from.clone());
    args.push(request.to.clone());

    if let Some(filter) = &request.path_filter {
        validate_git_args(filter)?;
        args.push("--".to_string());
        args.push(filter.clone());
    }

    let out = cmd(&args, &repo_root).map_err(|e| format!("git diff failed: {}", e))?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn git_blame(request: &GitBlameRequest) -> Result<GitBlameOutput, String> {
    let output = cmd(&["rev-parse", "--show-toplevel"], &request.path)
        .map_err(|e| format!("Not a git repository: {}", e))?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

    // file must be absolute
    let file_abs = if request.file.is_absolute() {
        request.file.clone()
    } else {
        repo_root.join(&request.file)
    };

    if !file_abs.exists() {
        return Err(format!("File not found: {}", file_abs.display()));
    }

    let max_lines = std::cmp::min(request.max_lines, 10000);

    // Run git blame with porcelain format
    let blame_out = cmd(&[
        "blame",
        "--line-porcelain",
        "-n",
        &max_lines.to_string(),
        "--",
        file_abs.to_string_lossy().as_ref(),
    ], &repo_root).map_err(|e| format!("git blame failed: {}", e))?;

    let blame_text = String::from_utf8_lossy(&blame_out.stdout);
    let mut entries = Vec::new();
    let mut current_sha = String::new();
    let mut current_line_num = 0usize;
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_content = String::new();

    for line in blame_text.lines() {
        if line.is_empty() {
            // Empty line separates entries
            if !current_sha.is_empty() {
                entries.push(GitBlameEntry {
                    line_number: current_line_num,
                    sha: current_sha.clone(),
                    author: current_author.clone(),
                    date: current_date.clone(),
                    content: current_content.clone(),
                });
                current_sha.clear();
                current_content.clear();
            }
            continue;
        }
        if line.starts_with(' ') {
            // Content line
            current_content = line[1..].to_string();
        } else if line.starts_with("SHA ") {
            current_sha = line[4..].trim().to_string();
        } else if line.starts_with("author ") {
            current_author = line[7..].to_string();
        } else if line.starts_with("author-mail ") {
            // skip
        } else if line.starts_with("original-file ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(n) = parts[2].parse::<usize>() {
                    current_line_num = n;
                }
            }
        } else if line.starts_with("author-time ") {
            // skip
        } else if line.starts_with("author-tz ") {
            // skip
        }
    }

    // Don't forget the last entry
    if !current_sha.is_empty() {
        entries.push(GitBlameEntry {
            line_number: current_line_num,
            sha: current_sha,
            author: current_author,
            date: current_date,
            content: current_content,
        });
    }

    Ok(GitBlameOutput {
        file: file_abs.to_string_lossy().to_string(),
        total_lines: entries.len(),
        entries,
    })
}

fn validate_git_rev(rev: &str) -> Result<(), String> {
    if rev.is_empty() || rev.len() > 64 {
        return Err("Invalid revision".to_string());
    }
    // Allow hex, or common symbolic refs
    if rev.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(());
    }
    // Check if it's a known ref
    match cmd(&["rev-parse", "--verify", rev], &std::env::current_dir().unwrap()) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Invalid revision: {}", rev)),
    }
}

fn validate_git_args(arg: &str) -> Result<(), String> {
    // Reject dangerous characters
    if arg.contains('\0') || arg.contains('\n') || arg.contains('\r') {
        return Err("Invalid characters in argument".to_string());
    }
    if arg.starts_with('-') {
        return Err("Arguments starting with - are not allowed".to_string());
    }
    Ok(())
}

fn cmd(args: &[&str], cwd: &Path) -> Result<std::process::Output, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git command failed: {}", stderr.trim()));
    }

    Ok(output)
}
