use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "diff_files",
    title = "Diff files",
    description = "Compute a unified diff between two sides. Each side can be a file path or inline text (prefix with 'text:').",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DiffFiles {
    /// First side: file path or "text:your content here".
    pub left: String,
    /// Second side: file path or "text:your content here".
    pub right: String,
    /// Output format: "unified" (default) or "minimal" (summary + first 20 changes).
    #[serde(default)]
    pub format: String,
}

impl DiffFiles {
    pub async fn run_tool(
        params: Self,
        _context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::diff_files::{diff_files, DiffFilesRequest};

        let request = DiffFilesRequest {
            left: params.left,
            right: params.right,
            format: params.format,
        };

        let output = diff_files(&request).map_err(|e| CallToolError::new(e))?;

        let mut result = String::new();
        if output.format == "minimal" {
            result.push_str(&output.diff);
        } else {
            result.push_str(&format!(
                "--- left\n+++ right\n{}\n",
                output.diff
            ));
            result.push_str(&format!(
                "Summary: {} insertions(+), {} deletions(-)\n",
                output.added, output.removed
            ));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
