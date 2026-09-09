use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "list_pdf_pages",
    title = "List PDF pages",
    description = "Get metadata about pages in a PDF file, including page count and dimensions.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListPdfPages {
    /// Path to the PDF file.
    pub path: String,
}

impl ListPdfPages {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::pdf::list_pdf_pages(Path::new(&params.path), context).await
    }
}
