use std::path::Path;

use calamine::{Reader, Xlsx, open_workbook};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};
use rust_xlsxwriter::Workbook;

pub async fn read_excel(
    path: &Path,
    sheet: Option<&str>,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e| {
        CallToolError::new(format!("Failed to open Excel file: {}", e))
    })?;

    let sheet_name = sheet
        .or_else(|| workbook.worksheets().first().copied());

    let sheet_name = match sheet_name {
        Some(s) => s,
        None => {
            return Ok(CallToolResult::with_error(CallToolError::new(
                "No sheets found in Excel file.".to_string(),
            )));
        }
    };

    let range = workbook
        .range(&sheet_name)
        .map_err(|e| CallToolError::new(format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

    let mut output = format!("Sheet: {}\nRows: {}\nColumns: {}\n\n", sheet_name, range.rows(), range.cols());

    for (row_idx, row) in range.rows().enumerate() {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| match cell {
                calamine::CellDataType::String(s) => s.clone(),
                calamine::CellDataType::Float(f) => format!("{}", f),
                calamine::CellDataType::Int(i) => format!("{}", i),
                calamine::CellDataType::Empty => String::new(),
                _ => String::new(),
            })
            .collect();
        output.push_str(&format!("{}\n", cells.join("\t")));
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}

pub async fn list_excel_sheets(
    path: &Path,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e| {
        CallToolError::new(format!("Failed to open Excel file: {}", e))
    })?;

    let sheets = workbook.worksheets();
    let mut output = format!("Excel file: {}\nSheets: {}\n\n", path.display(), sheets.len());

    for sheet_name in &sheets {
        if let Ok(range) = workbook.range(sheet_name) {
            output.push_str(&format!(
                "  {} ({} rows x {} columns)\n",
                sheet_name,
                range.rows(),
                range.cols()
            ));
        } else {
            output.push_str(&format!("  {} (unable to read dimensions)\n", sheet_name));
        }
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}

pub async fn write_excel(
    path: &Path,
    sheet_name: &str,
    data: &[Vec<String>],
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet(Some(sheet_name));

    for (row_idx, row) in data.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            worksheet.write(row_idx as u16, col_idx as u16, cell).map_err(|e| {
                CallToolError::new(format!("Failed to write cell: {}", e))
            })?;
        }
    }

    workbook.save(file_path).map_err(|e| {
        CallToolError::new(format!("Failed to save Excel file: {}", e))
    })?;

    Ok(CallToolResult::text_content(vec![TextContent::from(
        format!(
            "Successfully wrote {} rows x {} columns to '{}'",
            data.len(),
            data.first().map_or(0, |r| r.len()),
            path.display()
        ),
    )]))
}
