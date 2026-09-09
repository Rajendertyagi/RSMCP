use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "list_docx_parts",
    title = "List DOCX parts",
    description = "List all XML parts contained in a DOCX file, including paragraphs, tables, and images.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListDocxParts {
    /// Path to the DOCX file.
    pub path: String,
}

impl ListDocxParts {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::docx::list_docx_parts(Path::new(&params.path), context).await
    }
}
