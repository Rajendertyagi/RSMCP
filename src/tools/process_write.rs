use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;

#[mcp_tool(
    name = "process_write",
    title = "Write to process stdin",
    description = "Send input to a running process's stdin.",
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProcessWrite {
    /// The PID of the process.
    pub pid: u32,
    /// The data to write.
    pub data: String,
}

impl ProcessWrite {
    pub async fn run_tool(
        params: Self,
        context: &crate::process::ProcessManager,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        context.write_input(params.pid, &params.data).await.map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            "Input sent successfully.",
        )]))
    }
}
