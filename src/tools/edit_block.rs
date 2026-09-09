use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "edit_block",
    title = "Edit block",
    description = "Apply surgical text replacement with fuzzy matching fallback. Search for exact or similar text and replace it.",
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EditBlock {
    /// Path to the text file.
    pub path: String,
    /// The text to search for.
    pub search: String,
    /// The replacement text.
    pub replace: String,
    /// Expected number of replacements (default: 1).
    #[serde(default = "default_expected")]
    pub expected: usize,
}

fn default_expected() -> usize {
    1
}

impl EditBlock {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::fuzzy_edit::edit_block(
            Path::new(&params.path),
            &params.search,
            &params.replace,
            params.expected,
            context,
        )
        .await
    }
}
