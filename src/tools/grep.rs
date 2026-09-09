use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "grep",
    title = "Grep enhanced",
    description = "Regex search across files with context lines and match limits. Supports single files and directory recursion.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Grep {
    /// Path to search (file or directory).
    pub path: String,
    /// Regex pattern to search for.
    pub pattern: String,
    /// Case-sensitive search (default: false).
    #[serde(default)]
    pub case_sensitive: bool,
    /// Number of context lines before and after each match (default: 0).
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
    /// Maximum number of matches to return (default: 1000).
    #[serde(default = "default_max_matches")]
    pub max_matches: usize,
}

fn default_context_lines() -> usize { 0 }
fn default_max_matches() -> usize { 1000 }

impl Grep {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::grep_enhanced::{GrepRequest, grep};

        let request = GrepRequest {
            path: params.path,
            pattern: params.pattern,
            case_sensitive: params.case_sensitive,
            context_lines: params.context_lines,
            max_matches: params.max_matches,
        };

        let matches = grep(&request, context).await.map_err(|e| CallToolError::new(e))?;

        if matches.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                "No matches found.",
            )]));
        }

        let mut result = format!("Found {} match(es):\n\n", matches.len());
        for m in &matches {
            if !m.context_before.is_empty() {
                for line in &m.context_before {
                    result.push_str(&format!("  {} |\n", line));
                }
            }
            result.push_str(&format!(
                "  {}:{} | {}\n",
                m.file_path, m.line_number, m.match_line
            ));
            if !m.context_after.is_empty() {
                for line in &m.context_after {
                    result.push_str(&format!("  {} |\n", line));
                }
            }
            result.push('\n');
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
