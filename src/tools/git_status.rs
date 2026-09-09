use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "git_status",
    title = "Git status",
    description = "Get repository status: branch, ahead/behind, staged/modified/untracked/conflicted files.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GitStatus {
    /// Path to the git repository.
    pub path: String,
}

impl GitStatus {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::git_read::git_status;

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let output = git_status(&abs_path).map_err(|e| CallToolError::new(e))?;

        let mut result = format!(
            "Branch: {}{}\nAhead: {}\nBehind: {}\n\n",
            output.branch,
            if output.detached { " (detached)" } else { "" },
            output.ahead,
            output.behind
        );

        if !output.staged.is_empty() {
            result.push_str(&format!("Staged ({}):\n", output.staged.len()));
            for f in &output.staged {
                result.push_str(&format!("  A  {}\n", f));
            }
        }
        if !output.modified.is_empty() {
            result.push_str(&format!("Modified ({}):\n", output.modified.len()));
            for f in &output.modified {
                result.push_str(&format!("  M  {}\n", f));
            }
        }
        if !output.untracked.is_empty() {
            result.push_str(&format!("Untracked ({}):\n", output.untracked.len()));
            for f in &output.untracked {
                result.push_str(&format!("  ?  {}\n", f));
            }
        }
        if !output.conflicted.is_empty() {
            result.push_str(&format!("Conflicted ({}):\n", output.conflicted.len()));
            for f in &output.conflicted {
                result.push_str(&format!("  U  {}\n", f));
            }
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
