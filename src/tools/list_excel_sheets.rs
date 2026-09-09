use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "list_excel_sheets",
    title = "List Excel sheets",
    description = "List all sheet names and their dimensions in an Excel file.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListExcelSheets {
    /// Path to the Excel file.
    pub path: String,
}

impl ListExcelSheets {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::excel::list_excel_sheets(Path::new(&params.path), context).await
    }
}
