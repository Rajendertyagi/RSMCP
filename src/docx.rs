//! DOCX reading — stub implementation for compilation.

use std::path::Path;

use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};

pub async fn read_docx(
    _path: &Path,
    _max_chars: usize,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        "DOCX reading requires the 'quick-xml' crate. \
         This tool is registered but not yet fully implemented."
            .to_string(),
    )))
}

pub async fn list_docx_parts(
    _path: &Path,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        "DOCX part listing is not yet implemented."
            .to_string(),
    )))
}
