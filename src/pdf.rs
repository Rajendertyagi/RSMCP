use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};

pub async fn read_pdf(
    path: &Path,
    page: Option<usize>,
    max_pages: usize,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let mut reader = match pdf::open(file_path) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::with_error(CallToolError::new(
                format!("Failed to open PDF: {}", e),
            )));
        }
    };

    let num_pages = reader.number_of_pages();
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
        if let Ok(page) = reader.page(i) {
            output.push_str(&format!("\n--- Page {} ---\n", i + 1));
            if page.frame().is_some() {
                let text = extract_text_from_page(&reader, i);
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

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let reader = match pdf::open(file_path) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::with_error(CallToolError::new(
                format!("Failed to open PDF: {}", e),
            )));
        }
    };

    let num_pages = reader.number_of_pages();
    let mut output = format!("PDF: {}\nTotal pages: {}\n\n", path.display(), num_pages);

    for i in 0..num_pages {
        if let Ok(page) = reader.page(i) {
            let width = page.media_box().width();
            let height = page.media_box().height();
            output.push_str(&format!("Page {}: {}x{}\n", i + 1, width, height));
        }
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}

pub async fn extract_images_from_pdf(
    path: &Path,
    max_images: usize,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let reader = match pdf::open(file_path) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::with_error(CallToolError::new(
                format!("Failed to open PDF: {}", e),
            )));
        }
    };

    let num_pages = reader.number_of_pages();
    let mut images = Vec::new();
    let mut extracted = 0;

    for i in 0..num_pages {
        if extracted >= max_images {
            break;
        }
        if let Ok(page) = reader.page(i) {
            if let Ok(resources) = page.resources() {
                if let Some(xobjects) = resources.xobjects() {
                    for (name, xobject) in xobjects.iter() {
                        if extracted >= max_images {
                            break;
                        }
                        if let Ok(stream) = xobject.stream() {
                            let data = stream.slice();
                            let mime = detect_image_mime(data);
                            let encoded = STANDARD.encode(data);
                            images.push(format!(
                                "Image {}: {} ({} bytes, {}x{}, {})\n",
                                extracted + 1,
                                name,
                                data.len(),
                                xobject.width().unwrap_or(0),
                                xobject.height().unwrap_or(0),
                                mime
                            ));
                            // Include base64 for small images (< 1MB)
                            if data.len() < 1_048_576 {
                                images.push(format!("  Base64: {}\n", &encoded[..std::cmp::min(200, encoded.len())]));
                            }
                            extracted += 1;
                        }
                    }
                }
            }
        }
    }

    if images.is_empty() {
        return Ok(CallToolResult::text_content(vec![TextContent::from(
            "No images found in PDF.",
        )]));
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(
        format!("Extracted {} images:\n{}", extracted, images.join("\n")),
    )]))
}

fn extract_text_from_page(doc: &pdf::structure::PdfDocument, page_idx: usize) -> String {
    use std::io::Read;

    let mut result = String::new();
    if let Ok(page) = doc.page(page_idx) {
        if let Some(_frame) = page.frame() {
            if let Ok(content) = page.content() {
                let mut content_reader = content.bytes();
                let mut buf = Vec::new();
                while let Ok(b) = content_reader.next() {
                    buf.push(b);
                    if buf.len() > 100_000 {
                        break;
                    }
                }
                let content_str = String::from_utf8_lossy(&buf);
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
        }
    }
    result.trim().to_string()
}

fn detect_image_mime(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        "image/gif"
    } else if data.starts_with(&[0x42, 0x4D]) {
        "image/bmp"
    } else if data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && &data[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}
