use std::path::Path;

use regex::Regex;
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use strsim::{levenshtein, jaro_winkler};
use crate::error::ServiceError;

const FUZZY_THRESHOLD: f64 = 0.7;

pub async fn edit_block(
    path: &Path,
    search: &str,
    replace: &str,
    expected: i64,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    if search.is_empty() {
        return Ok(CallToolResult::with_error(CallToolError::new(
            ServiceError::FromString("Search string cannot be empty.".to_string()),
        )));
    }

    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        CallToolError::new(ServiceError::FromString(format!("Failed to read file: {}", e)))
    })?;

    let count = content.matches(search).count();
    let expected_usize = expected as usize;

    if count > 0 && count == expected_usize {
        let new_content = if expected == 1 {
            content.replacen(search, replace, 1)
        } else {
            content.replace(search, replace)
        };

        std::fs::write(&file_path, &new_content).map_err(|e| {
            CallToolError::new(ServiceError::FromString(format!("Failed to write file: {}", e)))
        })?;

        return Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Successfully replaced {} occurrence(s) in {}.", count, path.display()),
        )]));
    }

    if count > 0 && count != expected_usize {
        return Ok(CallToolResult::with_error(CallToolError::new(
            ServiceError::FromString(
                format!(
                    "Expected {} occurrences but found {} in {}. Please adjust expected count or search string.",
                    expected, count, path.display()
                ),
            ),
        )));
    }

    let best_match = find_best_match(&content, search);
    let similarity = calculate_similarity(search, &best_match);

    if similarity >= FUZZY_THRESHOLD {
        let diff = highlight_diff(search, &best_match);
        return Ok(CallToolResult::with_error(CallToolError::new(
            ServiceError::FromString(
                format!(
                    "Exact match not found. Found similar text with {}% similarity:\n\n{}\n\nTo replace, use the exact text shown above.",
                    (similarity * 100.0) as usize, diff
                ),
            ),
        )));
    }

    Ok(CallToolResult::with_error(CallToolError::new(
        ServiceError::FromString(
            format!(
                "Search content not found in {}. The closest match was '{}' with only {}% similarity, which is below the {}% threshold.",
                path.display(),
                best_match,
                (similarity * 100.0) as usize,
                (FUZZY_THRESHOLD * 100.0) as usize
            ),
        ),
    )))
}

pub async fn search_and_replace(
    path: &Path,
    pattern: &str,
    replacement: &str,
    max_replacements: i64,
    is_regex: bool,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        CallToolError::new(ServiceError::FromString(format!("Failed to read file: {}", e)))
    })?;

    if is_regex {
        let re = Regex::new(pattern).map_err(|e| {
            CallToolError::new(ServiceError::FromString(format!("Invalid regex pattern: {}", e)))
        })?;
        let matches = re.find_iter(&content).count();
        let max = if max_replacements > 0 {
            max_replacements as usize
        } else {
            usize::MAX
        };
        let mut count = 0;
        let _result = re.replace_all(&content, |cap: &regex::Captures| {
            if count < max {
                count += 1;
                replacement.to_string()
            } else {
                cap[0].to_string()
            }
        });
        return Ok(CallToolResult::text_content(vec![TextContent::from(
            format!(
                "Replaced {} occurrence(s) of '{}' in {}.",
                count, pattern, path.display()
            ),
        )]));
    }

    let count = content.matches(pattern).count();
    let new_content = if max_replacements > 0 && count > max_replacements as usize {
        content.replacen(pattern, replacement, max_replacements as usize)
    } else {
        content.replace(pattern, replacement)
    };

    std::fs::write(&file_path, &new_content).map_err(|e| {
        CallToolError::new(ServiceError::FromString(format!("Failed to write file: {}", e)))
    })?;

    Ok(CallToolResult::text_content(vec![TextContent::from(
        format!("Successfully replaced in {}.", path.display()),
    )]))
}

fn find_best_match(content: &str, search: &str) -> String {
    let mut best_match = String::new();
    let mut best_score = 0.0;

    for i in 0..content.len() {
        for j in (i + 1)..=std::cmp::min(i + search.len() + 10, content.len()) {
            let candidate = &content[i..j];
            let score = calculate_similarity(search, candidate);
            if score > best_score {
                best_score = score;
                best_match = candidate.to_string();
            }
        }
    }

    if best_match.is_empty() {
        content.chars().take(search.len()).collect()
    } else {
        best_match
    }
}

fn calculate_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let jw = jaro_winkler(a, b);
    let lv = 1.0 - (levenshtein(a, b) as f64 / std::cmp::max(a.len(), b.len()) as f64);

    0.6 * jw + 0.4 * lv
}

fn highlight_diff(expected: &str, actual: &str) -> String {
    let mut result = String::new();
    let exp_chars: Vec<char> = expected.chars().collect();
    let act_chars: Vec<char> = actual.chars().collect();
    let min_len = std::cmp::min(exp_chars.len(), act_chars.len());

    for i in 0..min_len {
        if exp_chars[i] == act_chars[i] {
            result.push(exp_chars[i]);
        } else {
            result.push_str(&format!("{{-{}-}}", exp_chars[i]));
            result.push_str(&format!("{{+{}+}}", act_chars[i]));
        }
    }

    if exp_chars.len() > act_chars.len() {
        for c in &exp_chars[min_len..] {
            result.push_str(&format!("{{-{}-}}", c));
        }
    } else if act_chars.len() > exp_chars.len() {
        for c in &act_chars[min_len..] {
            result.push_str(&format!("{{+{}+}}", c));
        }
    }

    result
}
