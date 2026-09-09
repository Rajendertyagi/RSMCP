use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "git_diff",
    title = "Git diff",
    description = "Get unified diff between two revisions, or between a revision and the working tree.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GitDiff {
    /// Path to the git repository.
    pub path: String,
    /// First revision (e.g., "HEAD", "main", "abc1234").
    pub from: String,
    /// Second revision (e.g., "HEAD", "origin/main", "def5678").
    pub to: String,
    /// Optional path filter to limit the diff to this path.
    #[serde(default)]
    pub path_filter: Option<String>,
}

impl GitDiff {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::git_read::{git_diff, GitDiffRequest};

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let request = GitDiffRequest {
            path: abs_path,
            from: params.from,
            to: params.to,
            path_filter: params.path_filter,
        };

        let diff = git_diff(&request).map_err(|e| CallToolError::new(e))?;

        Ok(CallToolResult::text_content(vec![TextContent::from(diff)]))
    }
}
