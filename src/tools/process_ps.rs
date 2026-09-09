use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_ps",
    title = "List system processes",
    description = "List all running processes on the system (like ps/Task Manager).",
    read_only_hint = true,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessPs {}

impl ProcessPs {
    pub async fn run_tool(
        _params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let processes = context.list_processes().await?;
        if processes.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                "No processes found.",
            )]));
        }
        let mut output = String::from("Running processes:\n");
        for p in processes.iter().take(100) {
            output.push_str(&format!("  {} (PID: {})\n", p.name, p.pid));
        }
        if processes.len() > 100 {
            output.push_str(&format!("  ... and {} more processes\n", processes.len() - 100));
        }
        Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
    }
}
