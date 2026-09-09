use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "read_excel",
    title = "Read Excel",
    description = "Read data from an Excel file and return it as structured text or JSON.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadExcel {
    /// Path to the Excel file.
    pub path: String,
    /// Optional sheet name to read. If None, reads the first sheet.
    pub sheet: Option<String>,
}

impl ReadExcel {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::excel::read_excel(
            Path::new(&params.path),
            params.sheet.as_deref(),
            context,
        )
        .await
    }
}
