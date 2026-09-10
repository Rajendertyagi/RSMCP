use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_kill",
    title = "Kill process",
    description = "Terminate a running process by PID.",
    destructive_hint = true,
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessKill {
    /// The PID of the process to kill.
    pub pid: u32,
}

impl ProcessKill {
    pub async fn run_tool(
        params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let result = context.kill(params.pid).await.map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            if result.killed {
                format!("Process {} terminated.", params.pid)
            } else {
                format!("No active session found for PID {}", params.pid)
            },
        )]))
    }
}
