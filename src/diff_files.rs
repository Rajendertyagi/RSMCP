//! Unified diff between two sides (file or inline string).
//! Adapted from winfs-mcp (MIT).

use std::path::Path;

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DiffFilesRequest {
    /// First side: file path or inline text prefix "text:".
    pub left: String,
    /// Second side: file path or inline text prefix "text:".
    pub right: String,
    /// "unified" or "minimal" summary format.
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFilesOutput {
    pub diff: String,
    pub added: usize,
    pub removed: usize,
    pub format: String,
}

pub fn diff_files(request: &DiffFilesRequest) -> Result<DiffFilesOutput, String> {
    let left_content = extract_side(&request.left)?;
    let right_content = extract_side(&request.right)?;

    let text_diff = TextDiff::from_lines(&left_content, &right_content);

    let mut diff_output = String::new();
    let mut added = 0usize;
    let mut removed = 0usize;

    // Collect all changes
    let mut all_changes: Vec<(ChangeTag, String)> = Vec::new();
    for change in text_diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                diff_output.push_str(&format!(" {}\n", change.value()));
            }
            ChangeTag::Insert => {
                diff_output.push_str(&format!("+{}\n", change.value()));
                added += 1;
            }
            ChangeTag::Delete => {
                diff_output.push_str(&format!("-{}\n", change.value()));
                removed += 1;
            }
        }
    }

    // Minimal format
    if request.format.to_lowercase() == "minimal" {
        let mut result = String::new();
        result.push_str(&format!("{} insertions(+), {} deletions(-)\n\n", added, removed));

        let mut count = 0usize;
        let mut in_context = false;
        let context_buffer: Vec<String> = Vec::new();
        let mut ctx_count = 0usize;

        for change in text_diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    if !in_context && count < 20 {
                        result.push_str(&format!(" {}\n", change.value()));
                        count += 1;
                        in_context = true;
                        ctx_count = 1;
                    } else if in_context && ctx_count < 3 && count < 20 {
                        result.push_str(&format!(" {}\n", change.value()));
                        ctx_count += 1;
                        count += 1;
                    }
                }
                ChangeTag::Insert | ChangeTag::Delete => {
                    in_context = false;
                    ctx_count = 0;
                    let prefix = if change.tag() == ChangeTag::Insert { "+" } else { "-" };
                    result.push_str(&format!("{}{}\n", prefix, change.value()));
                    count += 1;
                    if count >= 20 {
                        result.push_str("... (truncated)\n");
                        break;
                    }
                }
            }
        }
        return Ok(DiffFilesOutput { diff: result, added, removed, format: "minimal".to_string() });
    }

    Ok(DiffFilesOutput {
        diff: diff_output,
        added,
        removed,
        format: "unified".to_string(),
    })
}

fn extract_side(input: &str) -> Result<String, String> {
    if input.starts_with("text:") {
        Ok(input[5..].to_string())
    } else {
        let path = Path::new(input);
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", input, e))
    }
}
