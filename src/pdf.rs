//! PDF reading tools using the `pdf` crate.

use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use pdf::file::FileHandle;
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};

pub async fn read_pdf(
    path: &Path,
    page: Option<usize>,
    max_pages: usize,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;
    let file_path = resolved.display.clone();

    let file = std::fs::File::open(&file_path).map_err(|e| {
        CallToolError::new(format!("Failed to open PDF: {}", e))
    })?;

    let mut reader = pdf::parser::PdfParser::new(FileHandle::from(file))
        .map_err(|e| CallToolError::new(format!("Failed to parse PDF: {}", e)))?;

    let num_pages = reader.get_pages().len();
    let page_to_read = page.unwrap_or(0);

    if page_to_read > 0 && page_to_read > num_pages {
        return Ok(CallToolResult::with_error(CallToolError::new(
            format!("Page {} does not exist. Document has {} pages.", page_to_read, num_pages),
        )));
    }

    let max_pages = std::cmp::min(max_pages, num_pages);
    let start_page = if page_to_read > 0 { page_to_read - 1 } else { 0 };
    let end_page = std::cmp::min(start_page + max_pages, num_pages);

    let mut output = String::new();
    output.push_str(&format!("PDF: {}\nPages: {}\n\n", path.display(), num_pages));

    for i in start_page..end_page {
        output.push_str(&format!("\n--- Page {} ---\n", i + 1));
        if let Some(page) = reader.get_pages().get(i) {
            if let Ok(text) = extract_page_text(&reader, page) {
                output.push_str(&text);
            }
        }
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}

pub async fn list_pdf_pages(
    path: &Path,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;
    let file_path = resolved.display.clone();

    let file = std::fs::File::open(&file_path).map_err(|e| {
        CallToolError::new(format!("Failed to open PDF: {}", e))
    })?;

    let mut reader = pdf::parser::PdfParser::new(FileHandle::from(file))
        .map_err(|e| CallToolError::new(format!("Failed to parse PDF: {}", e)))?;

    let num_pages = reader.get_pages().len();
    let mut output = format!("PDF: {}\nTotal pages: {}\n\n", path.display(), num_pages);

    for (i, _page) in reader.get_pages().iter().enumerate() {
        output.push_str(&format!("Page {} (index {})\n", i + 1, i));
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}

pub async fn extract_images_from_pdf(
    path: &Path,
    max_images: usize,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;
    let file_path = resolved.display.clone();

    let file = std::fs::File::open(&file_path).map_err(|e| {
        CallToolError::new(format!("Failed to open PDF: {}", e))
    })?;

    let mut reader = pdf::parser::PdfParser::new(FileHandle::from(file))
        .map_err(|e| CallToolError::new(format!("Failed to parse PDF: {}", e)))?;

    let num_pages = reader.get_pages().len();
    let mut images = Vec::new();
    let mut extracted = 0;

    for i in 0..num_pages {
        if extracted >= max_images { break; }
        if let Some(page) = reader.get_pages().get(i) {
            if let Ok(resources) = page.get_resources() {
                if let Some(xobjects) = resources.get("XObject") {
                    // Iterate through XObjects to find images
                    // This is a simplified approach - full implementation would need
                    // to traverse the PDF object tree
                    let _ = xobjects;
                }
            }
        }
    }

    if images.is_empty() {
        return Ok(CallToolResult::text_content(vec![TextContent::from(
            "Image extraction is not yet implemented for this PDF version.\nUse a dedicated PDF tool for image extraction.",
        )]));
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(
        format!("Extracted {} images.", extracted),
    )]))
}

fn extract_page_text(reader: &pdf::parser::PdfParser, page: &pdf::object::Page) -> Result<String, String> {
    use std::io::Read;

    let mut result = String::new();

    // Get the page content stream
    if let Ok(content) = page.get_content() {
        let mut content_reader = content.bytes();
        let mut buf = Vec::new();
        while let Ok(b) = content_reader.next() {
            buf.push(b);
            if buf.len() > 100_000 { break; }
        }

        let content_str = String::from_utf8_lossy(&buf);
        // Extract text between parentheses (simplified approach)
        for part in content_str.split('(') {
            if let Some(end) = part.find(')') {
                let text = &part[..end];
                let clean: String = text.chars().filter(|c| c.is_printable() || *c == '\n').collect();
                if !clean.is_empty() && clean.len() > 1 {
                    result.push_str(&clean);
                    result.push(' ');
                }
            }
        }
    }

    Ok(result.trim().to_string())
}
