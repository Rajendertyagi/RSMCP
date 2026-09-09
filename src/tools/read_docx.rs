use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "read_docx",
    title = "Read DOCX",
    description = "Extract plain text content from a DOCX (Word) document.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadDocx {
    /// Path to the DOCX file.
    pub path: String,
    /// Maximum number of characters to extract (default: 50000).
    #[serde(default = "default_max_chars")]
    pub max_chars: i64,
}

fn default_max_chars() -> i64 {
    50000
}

impl ReadDocx {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::docx::read_docx(
            Path::new(&params.path),
            params.max_chars,
            context,
        )
        .await
    }
}
