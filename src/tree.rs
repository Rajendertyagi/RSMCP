//! Paginated, bounded directory tree traversal.
//! Adapted from fs-mcp-rs (GPL-3.0, adapted under MIT-compatible logic).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ListTreeRequest {
    pub path: PathBuf,
    #[serde(default = "default_depth")]
    pub depth: i64,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub max_entries: Option<i64>,
    pub cursor: Option<String>,
}

fn default_depth() -> i64 { 2 }

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    pub relative_path: String,
    pub display_path: String,
    pub name: String,
    pub kind: &'static str,
    pub size: Option<u64>,
    pub depth: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeWarning {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTreeOutput {
    pub root: String,
    pub depth: usize,
    pub entries: Vec<TreeEntry>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub warnings: Vec<TreeWarning>,
}

#[derive(Debug, Error)]
pub enum TreeError {
    #[error("path is not a directory")]
    NotDirectory,
    #[error("invalid cursor format")]
    InvalidCursor,
    #[error("cursor was created for different arguments")]
    IncompatibleCursor,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn list_tree(request: &ListTreeRequest) -> Result<ListTreeOutput, TreeError> {
    let root = if request.path.is_absolute() {
        request.path.clone()
    } else {
        std::env::current_dir()?.join(&request.path)
    };

    if !root.is_dir() {
        return Err(TreeError::NotDirectory);
    }

    let page_size = request.max_entries.unwrap_or(1000);
    let max_warnings = 32;

    // Build include/exclude globs (simple suffix matching for Windows paths)
    let inc_patterns: Vec<&str> = request.include.iter().map(|s| s.as_str()).collect();
    let exc_patterns: Vec<&str> = request.exclude.iter().map(|s| s.as_str()).collect();

    let mut entries = Vec::new();
    let mut warnings = Vec::new();

    walk_dir(&root, &root, 0, request.depth, &inc_patterns, &exc_patterns, &mut entries, &mut warnings, false);

    // Sort by relative path
    entries.sort_unstable_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let has_more = entries.len() > page_size;
    let mut final_entries = if has_more {
        entries.truncate(page_size);
        entries.clone()
    } else {
        entries
    };

    let next_cursor = if has_more {
        let last = final_entries.last().expect("non-empty").relative_path.clone();
        Some(encode_cursor(&root, &last))
    } else {
        None
    };

    Ok(ListTreeOutput {
        root: display_path(&root),
        depth: request.depth,
        entries: final_entries,
        next_cursor,
        has_more,
        warnings,
    })
}

fn walk_dir(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    include: &[&str],
    exclude: &[&str],
    entries: &mut Vec<TreeEntry>,
    warnings: &mut Vec<TreeWarning>,
    is_link: bool,
) {
    if depth > max_depth { return; }
    if is_link { return; } // skip symlinked directories

    let Ok(mut items) = std::fs::read_dir(current) else {
        if warnings.len() < 32 {
            warnings.push(TreeWarning {
                path: display_path(current),
                message: "permission denied".to_string(),
            });
        }
        return;
    };

    while let Some(Ok(entry)) = items.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if warnings.len() < max_warnings {
                    warnings.push(TreeWarning {
                        path: String::new(),
                        message: e.to_string(),
                    });
                }
                continue;
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let is_symlink = metadata.file_type().is_symlink();

        if is_symlink && is_dir {
            continue; // skip symlinked directories
        }

        let relative = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let rel_str = relative.to_string_lossy().replace('\\', "/");

        // Check exclude patterns
        let excluded = exclude.iter().any(|pat| glob_match(pat, &rel_str));
        if excluded { continue; }

        // Check include patterns (if any specified, must match)
        if !include.is_empty() && !include.iter().any(|pat| glob_match(pat, &rel_str)) {
            continue;
        }

        let kind = if is_dir { "directory" } else if is_symlink { "symlink" } else { "file" };
        entries.push(TreeEntry {
            relative_path: rel_str.clone(),
            display_path: display_path(&path),
            name: entry.file_name().to_string_lossy().into_owned(),
            kind,
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
            depth,
        });

        if is_dir && depth < max_depth {
            walk_dir(root, &path, depth + 1, max_depth, include, exclude, entries, warnings, is_symlink);
        }
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple glob matching: ** means any path, * means any filename segment
    let pat = pattern.to_lowercase();
    let txt = text.to_lowercase();
    if pat == "**" { return true; }
    if pat.ends_with("**") {
        return txt.starts_with(&pat[..pat.len()-2]);
    }
    if pat.contains("**") {
        let parts: Vec<&str> = pat.split("**").collect();
        return txt.contains(parts[0]) && (parts.len() == 1 || txt.contains(parts[1]));
    }
    if pat.contains('*') {
        let pat_bytes = pat.as_bytes();
        let txt_bytes = txt.as_bytes();
        return simple_glob_match(pat_bytes, txt_bytes);
    }
    pat == txt
}

fn simple_glob_match(pat: &[u8], text: &[u8]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi = None;
    let mut star_ti = 0;
    while ti < text.len() {
        if pi < pat.len() && (pat[pi] == b'?' || pat[pi] == text[ti]) {
            pi += 1; ti += 1;
        } else if pi < pat.len() && pat[pi] == b'*' {
            star_pi = Some(pi); star_ti = ti; pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1; star_ti += 1; ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < pat.len() && pat[pi] == b'*' { pi += 1; }
    pi == pat.len()
}

fn encode_cursor(root: &Path, relative: &str) -> String {
    let root_hash = blake3::hash(root.to_string_lossy().as_bytes());
    let rel_hash = blake3::hash(relative.as_bytes());
    format!("{}:{}", &root_hash.to_hex()[..8], &rel_hash.to_hex()[..16])
}

fn display_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
            return format!("\\\\{rest}");
        }
        if let Some(rest) = value.strip_prefix("\\\\?\\") {
            return rest.to_owned();
        }
    }
    value.into_owned()
}
