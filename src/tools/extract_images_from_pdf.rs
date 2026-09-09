use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "extract_images_from_pdf",
    title = "Extract PDF images",
    description = "Extract embedded images from a PDF file as base64-encoded data with MIME types.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ExtractImagesFromPdf {
    /// Path to the PDF file.
    pub path: String,
    /// Maximum number of images to extract (default: 20).
    #[serde(default = "default_max_images")]
    pub max_images: i64,
}

fn default_max_images() -> i64 {
    20
}

impl ExtractImagesFromPdf {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::pdf::extract_images_from_pdf(
            Path::new(&params.path),
            params.max_images,
            context,
        )
        .await
    }
}
