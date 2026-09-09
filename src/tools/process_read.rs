use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_read",
    title = "Read process output",
    description = "Read output from a running process by PID with pagination support.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessRead {
    /// The PID of the process.
    pub pid: u32,
    /// Offset to start reading from. Use negative values for tail reads.
    #[serde(default)]
    pub offset: i64,
    /// Number of lines to read.
    #[serde(default = "default_length")]
    pub length: i64,
}

fn default_length() -> i64 {
    100
}

impl ProcessRead {
    pub async fn run_tool(
        params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let result = context.read_output(params.pid, params.offset, params.length).await?;
        let msg = if result.output.is_empty() {
            "(No output in requested range)".to_string()
        } else {
            result.output
        };
        Ok(CallToolResult::text_content(vec![TextContent::from(msg)]))
    }
}
