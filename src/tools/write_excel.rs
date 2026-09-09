use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "write_excel",
    title = "Write Excel",
    description = "Write a 2D array of string data to an Excel file.",
    destructive_hint = false,
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WriteExcel {
    /// Path to the output Excel file.
    pub path: String,
    /// Name of the sheet to write to.
    pub sheet_name: String,
    /// 2D array of string values to write.
    pub data: Vec<Vec<String>>,
}

impl WriteExcel {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::excel::write_excel(
            Path::new(&params.path),
            &params.sheet_name,
            &params.data,
            context,
        )
        .await
    }
}
