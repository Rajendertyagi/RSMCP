//! Enhanced grep with context lines support.
//! Uses regex crate directly for pattern matching.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug)]
pub struct GrepMatch {
    pub file_path: String,
    pub line_number: usize,
    pub column: usize,
    pub context_before: Vec<String>,
    pub match_line: String,
    pub context_after: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrepRequest {
    pub path: String,
    pub pattern: String,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default = "default_context_lines")]
    pub context_lines: i64,
    #[serde(default = "default_max_matches")]
    pub max_matches: i64,
}

fn default_context_lines() -> i64 { 0 }
fn default_max_matches() -> i64 { 1000 }

pub async fn grep(request: &GrepRequest, fs: &crate::fs_service::FileSystemService) -> Result<Vec<GrepMatch>, String> {
    let search_path = Path::new(&request.path);
    let resolved = fs.resolve(search_path).await.map_err(|e| e.to_string())?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    if !file_path.exists() {
        return Err(format!("Path not found: {}", request.path));
    }

    let regex = regex::Regex::new(&request.pattern)
        .map_err(|e| format!("Invalid regex: {}", e))?;

    let mut results = Vec::new();
    let max_matches = request.max_matches;

    if file_path.is_file() {
        grep_file(&file_path, &regex, request, &mut results, max_matches);
    } else if file_path.is_dir() {
        grep_directory(&file_path, &regex, request, &mut results, max_matches);
    }

    Ok(results)
}

fn grep_file(
    file_path: &Path,
    regex: &regex::Regex,
    request: &GrepRequest,
    results: &mut Vec<GrepMatch>,
    max_matches: i64,
) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    for (i, line) in lines.iter().enumerate() {
        if results.len() >= max_matches { break; }
        if !regex.is_match(line) { continue; }

        let line_num = i + 1;
        let col = regex.find(line).map_or(0, |m| m.start() + 1);

        let ctx_before: Vec<String> = if request.context_lines > 0 {
            lines.iter()
                .skip(line_num.saturating_sub(request.context_lines as usize))
                .take(request.context_lines as usize)
                .map(|s| (*s).to_string())
                .collect()
        } else { Vec::new() };

        let ctx_after: Vec<String> = if request.context_lines > 0 {
            lines.iter()
                .skip(line_num)
                .take(request.context_lines as usize)
                .map(|s| (*s).to_string())
                .collect()
        } else { Vec::new() };

        results.push(GrepMatch {
            file_path: request.path.clone(),
            line_number: line_num,
            column: col,
            context_before: ctx_before,
            match_line: line.to_string(),
            context_after: ctx_after,
        });
    }
}

fn grep_directory(
    dir_path: &Path,
    regex: &regex::Regex,
    request: &GrepRequest,
    results: &mut Vec<GrepMatch>,
    max_matches: i64,
) {
    let text_extensions = [
        "rs", "ts", "js", "py", "go", "java", "c", "cpp", "h", "hpp",
        "txt", "md", "json", "yaml", "yml", "toml", "cfg", "ini", "sh",
        "bat", "ps1", "html", "css", "sql", "xml", "rb", "swift", "kt",
        "rust", "zig", "lua", "r", "m", "mm", "scala", "clj", "ex", "exs",
    ];

    let walk = ignore::Walk::new(dir_path)
        .filter(|e| {
            let p = match e {
                Ok(e) => e.path(),
                Err(_) => return true,
            };
            p.extension().map_or(false, |ext| {
                text_extensions.contains(&ext.to_string_lossy().as_ref())
            })
        })
        .filter(|e| {
            // Skip common non-source directories
            let p = match e {
                Ok(e) => e.path(),
                Err(_) => return true,
            };
            !p.iter().any(|c| {
                let s = c.to_string_lossy();
                matches!(s.as_ref(), ".git" | "node_modules" | ".venv" | "venv" | "__pycache__" | "target" | "dist" | "build" | ".DS_Store")
            })
        });

    for entry in walk.into_iter().filter_map(|e| e.ok()) {
        if results.len() >= max_matches { break; }
        let filepath = entry.path();
        if !filepath.is_file() { continue; }
        grep_file(filepath, regex, request, results, max_matches);
    }
}
