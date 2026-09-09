use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_start",
    title = "Start process",
    description = "Start a new background process and return its PID immediately.",
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessStart {
    /// The command to execute.
    pub command: String,
    /// Optional working directory.
    pub cwd: Option<String>,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
}

impl ProcessStart {
    pub async fn run_tool(
        params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use std::path::Path;
        let cwd = params.cwd.as_ref().map(Path::new);
        let result = context.start(&params.command, cwd, params.timeout_ms).await?;
        Ok(CallToolResult::text_content(vec![TextContent::from(result.message)]))
    }
}
