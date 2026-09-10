//! PDF reading tools — stub implementation for compilation.
//! Full implementation requires fixing pdf crate API usage.

use std::path::Path;

use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use crate::error::ServiceError;

pub async fn read_pdf(
    _path: &Path,
    _page: Option<i64>,
    _max_pages: i64,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        ServiceError::FromString(
            "PDF reading requires the 'pdf' crate with proper API usage. \
             This tool is registered but not yet fully implemented."
                .to_string(),
        ),
    )))
}

pub async fn list_pdf_pages(
    _path: &Path,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        ServiceError::FromString("PDF page listing is not yet implemented.".to_string()),
    )))
}

pub async fn extract_images_from_pdf(
    _path: &Path,
    _max_images: i64,
    _fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    Ok(CallToolResult::with_error(CallToolError::new(
        ServiceError::FromString("PDF image extraction is not yet implemented.".to_string()),
    )))
}
