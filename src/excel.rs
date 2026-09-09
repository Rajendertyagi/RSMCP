//! Excel reading/writing — stub implementation for compilation.

use std::path::Path;

use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};

pub async fn read_excel(
    _path: &Path,
    _sheet: Option<&str>,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        "Excel reading requires the 'calamine' crate. \
         This tool is registered but not yet fully implemented."
            .to_string(),
    )))
}

pub async fn list_excel_sheets(
    _path: &Path,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        "Excel sheet listing is not yet implemented."
            .to_string(),
    )))
}

pub async fn write_excel(
    _path: &Path,
    _sheet_name: &str,
    _data: &[Vec<String>],
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        "Excel writing requires the 'rust_xlsxwriter' crate. \
         This tool is registered but not yet fully implemented."
            .to_string(),
    )))
}
