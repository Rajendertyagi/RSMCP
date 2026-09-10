use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "git_blame",
    title = "Git blame",
    description = "Show per-line blame information for a file.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GitBlame {
    /// Path to the git repository.
    pub path: String,
    /// Absolute path to the file to blame.
    pub file: String,
    /// Maximum number of lines to blame (default: 1000, max: 10000).
    #[serde(default = "default_max_lines")]
    pub max_lines: i64,
}

fn default_max_lines() -> i64 { 1000 }

impl GitBlame {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::git_read::{git_blame, GitBlameRequest};

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let file_path = PathBuf::from(&params.file);
        let request = GitBlameRequest {
            path: abs_path,
            file: file_path,
            max_lines: params.max_lines,
        };

        let output = git_blame(&request).map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;

        let mut result = format!("Blame: {}\nTotal lines: {}\n\n", output.file, output.total_lines);

        // Show up to 50 entries by default to avoid overwhelming output
        let display_entries = std::cmp::min(output.entries.len(), 50);
        for entry in &output.entries[..display_entries] {
            let short_sha = &entry.sha[..8];
            result.push_str(&format!(
                "{:>6} {:>8} {}\n",
                entry.line_number, short_sha, entry.content
            ));
        }
        if output.entries.len() > 50 {
            result.push_str(&format!("\n... and {} more lines\n", output.entries.len() - 50));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
