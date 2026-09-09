//! Safe, bounded application of a unified diff to one existing UTF-8 file.
//! Adapted from fs-mcp-rs (GPL-3.0, used under MIT-compatible logic).

use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ApplyPatchRequest {
    pub path: PathBuf,
    pub patch: String,
    #[serde(rename = "expectedBlake3")]
    pub expected_blake3: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPatchOutput {
    pub applied: bool,
    pub dry_run: bool,
    pub changed: bool,
    pub hunks_applied: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub old_blake3: String,
    pub new_blake3: String,
    pub byte_length: usize,
    pub preview: String,
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("patch exceeds maximum size")]
    TooLarge,
    #[error("invalid unified patch: {0}")]
    Invalid(String),
    #[error("unsupported patch operation: {0}")]
    Unsupported(String),
    #[error("patch header does not match target path")]
    HeaderMismatch,
    #[error("patch context does not match the current file at line {line}")]
    ContextMismatch { line: usize },
    #[error("expected BLAKE3 hash does not match current file")]
    HashConflict,
}

#[derive(Debug)]
struct Hunk {
    old_start: usize,
    old_count: usize,
    new_count: usize,
    lines: Vec<HunkLine>,
}

#[derive(Debug)]
enum HunkLine {
    Context(String),
    Remove(String),
    Add(String),
}

const MAX_PATCH_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_PREVIEW_BYTES: usize = 16 * 1024;

pub fn apply_patch(request: &ApplyPatchRequest) -> Result<ApplyPatchOutput, PatchError> {
    if request.patch.len() > MAX_PATCH_BYTES {
        return Err(PatchError::TooLarge);
    }

    let bytes = std::fs::read(&request.path)?;
    let original = std::str::from_utf8(&bytes).map_err(|_| PatchError::Invalid("file is not valid UTF-8".into()))?;
    let old_blake3 = blake3::hash(&bytes).to_hex();

    if let Some(expected) = &request.expected_blake3 {
        if !expected.eq_ignore_ascii_case(&old_blake3) {
            return Err(PatchError::HashConflict);
        }
    }

    let hunks = parse_patch(&request.patch, &request.path)?;
    let line_ending = if original.contains("\r\n") { "\r\n" } else { "\n" };
    let final_newline = original.ends_with('\n');
    let normalized = original.replace("\r\n", "\n");
    let mut source: Vec<&str> = normalized.split('\n').collect();
    if final_newline {
        source.pop();
    }

    let mut result: Vec<String> = Vec::new();
    let mut source_index = 0usize;
    let mut added = 0usize;
    let mut removed = 0usize;

    for hunk in &hunks {
        let start = hunk.old_start.saturating_sub(1);
        if start < source_index || start > source.len() {
            return Err(PatchError::Invalid("overlapping or out-of-order hunk".into()));
        }
        result.extend(source[source_index..start].iter().map(|s| (*s).to_owned()));
        source_index = start;
        let mut observed_old = 0usize;
        let mut observed_new = 0usize;
        for line in &hunk.lines {
            match line {
                HunkLine::Context(expected) => {
                    if source.get(source_index).copied() != Some(expected.as_str()) {
                        return Err(PatchError::ContextMismatch { line: source_index + 1 });
                    }
                    result.push(expected.clone());
                    source_index += 1;
                    observed_old += 1;
                    observed_new += 1;
                }
                HunkLine::Remove(expected) => {
                    if source.get(source_index).copied() != Some(expected.as_str()) {
                        return Err(PatchError::ContextMismatch { line: source_index + 1 });
                    }
                    source_index += 1;
                    observed_old += 1;
                    removed += 1;
                }
                HunkLine::Add(value) => {
                    result.push(value.clone());
                    observed_new += 1;
                    added += 1;
                }
            }
        }
        if observed_old != hunk.old_count || observed_new != hunk.new_count {
            return Err(PatchError::Invalid("hunk line counts do not match header".into()));
        }
    }
    result.extend(source[source_index..].iter().map(|s| (*s).to_owned()));
    let mut output = result.join(line_ending).into_bytes();
    if final_newline {
        output.extend_from_slice(line_ending.as_bytes());
    }
    let changed = output != bytes;
    let new_blake3 = blake3::hash(&output).to_hex();

    if !request.dry_run && changed {
        std::fs::write(&request.path, &output)?;
    }

    let preview = bounded_preview(&request.patch, MAX_PREVIEW_BYTES);
    Ok(ApplyPatchOutput {
        applied: !request.dry_run && changed,
        dry_run: request.dry_run,
        changed,
        hunks_applied: hunks.len(),
        added_lines: added,
        removed_lines: removed,
        old_blake3: old_blake3.to_string(),
        new_blake3: new_blake3.to_string(),
        byte_length: output.len(),
        preview,
    })
}

fn parse_patch(patch: &str, target: &Path) -> Result<Vec<Hunk>, PatchError> {
    if patch.contains("GIT binary patch") || patch.contains("Binary files ") {
        return Err(PatchError::Unsupported("binary patch".into()));
    }
    if patch.contains("rename from ")
        || patch.contains("rename to ")
        || patch.contains("similarity index ")
    {
        return Err(PatchError::Unsupported("rename or move".into()));
    }
    let normalized = patch.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();
    let old_headers: Vec<usize> = lines.iter().enumerate().filter_map(|(i, l)| l.starts_with("--- ").then_some(i)).collect();
    let new_headers: Vec<usize> = lines.iter().enumerate().filter_map(|(i, l)| l.starts_with("+++ ").then_some(i)).collect();
    if old_headers.len() != 1 || new_headers.len() != 1 || new_headers[0] != old_headers[0] + 1 {
        return Err(PatchError::Unsupported("patch must affect exactly one file".into()));
    }
    let old_name = header_name(lines[old_headers[0]]);
    let new_name = header_name(lines[new_headers[0]]);
    if old_name == "/dev/null" || new_name == "/dev/null" {
        return Err(PatchError::Unsupported("file creation or deletion".into()));
    }
    if !header_matches(old_name, target) || !header_matches(new_name, target) {
        return Err(PatchError::HeaderMismatch);
    }
    let mut hunks = Vec::new();
    let mut i = new_headers[0] + 1;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("diff --git ") || line.starts_with("--- ") || line.starts_with("+++ ") {
            return Err(PatchError::Unsupported("multiple-file patch".into()));
        }
        if !line.starts_with("@@ ") {
            if line.is_empty() { i += 1; continue; }
            return Err(PatchError::Invalid(format!("unexpected line: {line}")));
        }
        let (old_start, old_count, new_count) = parse_hunk_header(line)?;
        i += 1;
        let mut hunk_lines = Vec::new();
        while i < lines.len() && !lines[i].starts_with("@@ ") {
            let value = lines[i];
            if value == "\\ No newline at end of file" { i += 1; continue; }
            let (prefix, rest) = value.split_at(value.char_indices().nth(1).map_or(value.len(), |(n, _)| n));
            let item = match prefix {
                " " => HunkLine::Context(rest.to_owned()),
                "-" => HunkLine::Remove(rest.to_owned()),
                "+" => HunkLine::Add(rest.to_owned()),
                _ => return Err(PatchError::Invalid("malformed hunk line".into())),
            };
            hunk_lines.push(item);
            i += 1;
        }
        hunks.push(Hunk { old_start, old_count, new_count, lines: hunk_lines });
    }
    if hunks.is_empty() { return Err(PatchError::Invalid("patch contains no hunks".into())); }
    Ok(hunks)
}

fn parse_hunk_header(line: &str) -> Result<(usize, usize, usize), PatchError> {
    let end = line[3..].find(" @@").ok_or_else(|| PatchError::Invalid("malformed hunk header".into()))? + 3;
    let mut parts = line[3..end].split_whitespace();
    let old = parts.next().ok_or_else(|| PatchError::Invalid("missing old range".into()))?;
    let new = parts.next().ok_or_else(|| PatchError::Invalid("missing new range".into()))?;
    if parts.next().is_some() || !old.starts_with('-') || !new.starts_with('+') {
        return Err(PatchError::Invalid("malformed hunk range".into()));
    }
    let (old_start, old_count) = parse_range(&old[1..])?;
    let (_, new_count) = parse_range(&new[1..])?;
    if old_start == 0 { return Err(PatchError::Invalid("old hunk start must be positive".into())); }
    Ok((old_start, old_count, new_count))
}

fn parse_range(value: &str) -> Result<(usize, usize), PatchError> {
    let mut parts = value.split(',');
    let start = parts.next().and_then(|v| v.parse().ok()).ok_or_else(|| PatchError::Invalid("invalid hunk range".into()))?;
    let count = parts.next().map_or(Ok(1), |v| v.parse().map_err(|_| PatchError::Invalid("invalid hunk count".into())))?;
    if parts.next().is_some() { return Err(PatchError::Invalid("invalid hunk range".into())); }
    Ok((start, count))
}

fn header_name(line: &str) -> &str {
    line[4..].split('\t').next().unwrap_or("").trim()
}

fn header_matches(header: &str, target: &Path) -> bool {
    let normalized = header.strip_prefix("a/").or_else(|| header.strip_prefix("b/")).unwrap_or(header).replace('\\', "/");
    let target_display = target.to_string_lossy().replace('\\', "/");
    target_display == normalized
        || target_display.ends_with(&format!("/{normalized}"))
        || target.file_name().is_some_and(|n| n.to_string_lossy() == normalized)
}

fn bounded_preview(patch: &str, limit: usize) -> String {
    let normalized = patch.replace("\r\n", "\n");
    if normalized.len() <= limit { return normalized; }
    let mut end = limit.min(normalized.len());
    while !normalized.is_char_boundary(end) { end -= 1; }
    format!("{}\n… preview truncated …", &normalized[..end])
}
