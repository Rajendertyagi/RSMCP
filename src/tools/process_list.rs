use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_list",
    title = "List sessions",
    description = "List all active process sessions.",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessList {}

impl ProcessList {
    pub async fn run_tool(
        _params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let sessions = context.list_sessions().await;
        if sessions.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                "No active sessions.",
            )]));
        }
        let mut output = String::from("Active sessions:\n");
        for s in sessions {
            output.push_str(&format!(
                "  PID {}: {} (buffer: {} lines)\n",
                s.pid,
                if s.is_complete { "completed" } else { "running" },
                s.buffer_size
            ));
        }
        Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
    }
}
