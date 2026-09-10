use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "list_tree",
    title = "List tree",
    description = "List a bounded recursive directory tree with pagination support.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListTree {
    /// Root directory path.
    pub path: String,
    /// Maximum descendant depth (default: 2).
    #[serde(default = "default_depth")]
    pub depth: i64,
    /// Optional inclusion glob patterns.
    #[serde(default)]
    pub include: Vec<String>,
    /// Optional exclusion glob patterns.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Page size (default: 1000).
    pub max_entries: Option<i64>,
    /// Opaque cursor from a previous response for pagination.
    #[serde(default)]
    pub cursor: Option<String>,
}

fn default_depth() -> i64 { 2 }

impl ListTree {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use std::path::PathBuf;
        use crate::tree::{ListTreeRequest, list_tree};

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let abs_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let request = ListTreeRequest {
            path: abs_path,
            depth: params.depth,
            include: params.include,
            exclude: params.exclude,
            max_entries: params.max_entries,
            cursor: params.cursor,
        };

        let output = list_tree(&request).map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e.to_string())))?;

        let mut result = format!(
            "Root: {}\nDepth: {}\nEntries: {}\nHas more: {}\n\n",
            output.root, output.depth, output.entries.len(), output.has_more
        );

        for entry in &output.entries {
            let indent = "  ".repeat(entry.depth);
            let icon = match entry.kind {
                "directory" => "[DIR]",
                "symlink" => "[LINK]",
                _ => "[FILE]",
            };
            let size = entry.size.map_or(String::new(), |s| format!(" ({s} bytes)"));
            result.push_str(&format!("{}{} {}{}\n", indent, icon, entry.name, size));
        }

        if !output.warnings.is_empty() {
            result.push_str("\nWarnings:\n");
            for w in &output.warnings {
                result.push_str(&format!("  {}: {}\n", w.path, w.message));
            }
        }

        if let Some(cursor) = &output.next_cursor {
            result.push_str(&format!("\nNext cursor: {}", cursor));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
