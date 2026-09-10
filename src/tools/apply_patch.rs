use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use serde::Deserialize;
use std::path::PathBuf;

#[mcp_tool(
    name = "apply_patch",
    title = "Apply patch",
    description = "Atomically apply a single-file unified text diff. Set dryRun=true to validate without writing.",
    destructive_hint = true,
    read_only_hint = false,
)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ApplyPatch {
    /// Target file path.
    pub path: String,
    /// Unified diff patch content.
    pub patch: String,
    /// Expected BLAKE3 hash of the current file (optional).
    #[serde(rename = "expectedBlake3", default)]
    pub expected_blake3: Option<String>,
    /// Validate without writing when true.
    #[serde(default)]
    pub dry_run: bool,
}

impl ApplyPatch {
    pub async fn run_tool(
        params: Self,
        context: &crate::fs_service::FileSystemService,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        use crate::patch::{ApplyPatchRequest, apply_patch};

        let resolved = context.resolve(PathBuf::from(&params.path)).await.map_err(CallToolError::new)?;
        let file_path = if let Some(unc_root) = &resolved.unc_root {
            unc_root.join(&resolved.rel)
        } else {
            resolved.display.clone()
        };

        let request = ApplyPatchRequest {
            path: file_path,
            patch: params.patch,
            expected_blake3: params.expected_blake3,
            dry_run: params.dry_run,
        };

        let output = apply_patch(&request).map_err(|e| CallToolError::new(crate::error::ServiceError::FromString(e)))?;

        let mut result = String::new();
        result.push_str(&format!(
            "Applied: {} | Changed: {} | Dry run: {}\n",
            output.applied, output.changed, output.dry_run
        ));
        result.push_str(&format!(
            "Hunks: {} | Added: {} | Removed: {}\n",
            output.hunks_applied, output.added_lines, output.removed_lines
        ));
        result.push_str(&format!("Old BLAKE3: {}\n", output.old_blake3));
        result.push_str(&format!("New BLAKE3: {}\n", output.new_blake3));
        result.push_str(&format!("Byte length: {}\n", output.byte_length));

        if !output.preview.is_empty() {
            result.push_str(&format!("\nPreview:\n{}\n", output.preview));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(result.trim().to_string())]))
    }
}
