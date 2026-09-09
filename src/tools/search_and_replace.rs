use std::path::Path;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "search_and_replace",
    title = "Search and replace",
    description = "Find all occurrences of a pattern and replace them. Supports regex patterns.",
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchAndReplace {
    /// Path to the file.
    pub path: String,
    /// The search pattern (literal or regex).
    pub pattern: String,
    /// The replacement string.
    pub replacement: String,
    /// Maximum number of replacements (0 = unlimited).
    #[serde(default)]
    pub max_replacements: usize,
    /// Whether to treat pattern as regex (default: false).
    #[serde(default)]
    pub is_regex: bool,
}

impl SearchAndReplace {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        crate::fuzzy_edit::search_and_replace(
            Path::new(&params.path),
            &params.pattern,
            &params.replacement,
            params.max_replacements,
            params.is_regex,
            context,
        )
        .await
    }
}
