use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "git_show",
    title = "Git show",
    description = "Show details of a single commit: metadata, body, diff, and changed files.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GitShow {
    /// Path to the git repository.
    pub path: String,
    /// Commit SHA (4-64 hex chars). Resolve via git_log first if using symbolic refs.
    pub sha: String,
}

impl GitShow {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::git_read::{git_show, GitShowRequest};

        let resolved = context.resolve(std::path::Path::new(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let request = GitShowRequest {
            path: abs_path,
            sha: params.sha,
        };

        let output = git_show(&request).map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;

        let mut result = String::new();
        result.push_str(&format!(
            "Commit: {}\nAuthor: {} <>\nDate:   {}\n\n{}\n\n",
            output.sha, output.author, output.date, output.subject
        ));
        if !output.body.is_empty() {
            result.push_str(&format!("Body:\n{}\n\n", output.body));
        }
        if !output.files_changed.is_empty() {
            result.push_str(&format!("Files changed ({}):\n", output.files_changed.len()));
            for f in &output.files_changed {
                result.push_str(&format!("  {}\n", f));
            }
            result.push('\n');
        }
        if !output.diff.is_empty() {
            result.push_str(&format!("Diff:\n{}", output.diff));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
