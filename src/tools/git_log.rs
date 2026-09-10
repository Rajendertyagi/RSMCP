use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "git_log",
    title = "Git log",
    description = "Get commit history with optional range and path filter.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GitLog {
    /// Path to the git repository.
    pub path: String,
    /// Number of commits to return (default: 20, max: 1000).
    #[serde(default = "default_count")]
    pub count: i64,
    /// Optional commit range (e.g., "main..HEAD").
    #[serde(default)]
    pub range: Option<String>,
    /// Optional path filter to limit commits touching this path.
    #[serde(default)]
    pub path_filter: Option<String>,
}

fn default_count() -> i64 { 20 }

impl GitLog {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::git_read::{git_log, GitLogRequest};

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let request = GitLogRequest {
            path: abs_path,
            count: params.count,
            range: params.range,
            path_filter: params.path_filter,
        };

        let output = git_log(&request).map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;

        let mut result = format!("Commits (showing {} of {}):\n\n", output.entries.len(), output.total);
        for entry in &output.entries {
            result.push_str(&format!(
                "{} {} {} {}\n",
                &entry.sha[..8], entry.date, entry.author, entry.subject
            ));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
