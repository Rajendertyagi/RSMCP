use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "read_pdf",
    title = "Read PDF",
    description = "Extract text content from a PDF file. Supports reading specific pages or the entire document.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadPdf {
    /// Path to the PDF file.
    pub path: String,
    /// Optional page number to read (1-based). If None, reads all pages.
    pub page: Option<i64>,
    /// Maximum number of pages to read (default: 10).
    #[serde(default = "default_max_pages")]
    pub max_pages: i64,
}

fn default_max_pages() -> i64 {
    10
}

impl ReadPdf {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::pdf::read_pdf(
            Path::new(&params.path),
            params.page,
            params.max_pages,
            context,
        )
        .await
    }
}
