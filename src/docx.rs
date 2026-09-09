use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError, TextContent};

pub async fn read_docx(
    path: &Path,
    max_chars: usize,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let bytes = std::fs::read(file_path).map_err(|e| {
        CallToolError::new(format!("Failed to read DOCX file: {}", e))
    })?;

    let text = extract_docx_text(&bytes, max_chars)?;

    Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
}

pub fn extract_docx_text(bytes: &[u8], max_chars: usize) -> Result<String, CallToolError> {
    let mut cursor = Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(&mut cursor) {
        Ok(a) => a,
        Err(e) => {
            return Err(CallToolError::new(format!(
                "Failed to open DOCX as ZIP archive: {}",
                e
            )));
        }
    };

    let mut doc_xml = match archive.by_name("word/document.xml") {
        Ok(f) => f,
        Err(_) => {
            return Err(CallToolError::new(
                "document.xml not found in DOCX".to_string(),
            ));
        }
    };

    let mut xml_content = String::new();
    doc_xml
        .read_to_string(&mut xml_content)
        .map_err(|e| CallToolError::new(format!("Failed to read document.xml: {}", e)))?;

    let mut reader = XmlReader::from_str(&xml_content);
    reader.set_buffer_capacity(4096);

    let mut text_parts = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = reader.decoder().decode(e.name()).unwrap_or_default();
                if name == "w:p" || name == "w:table" {
                    in_paragraph = true;
                    current_text.clear();
                }
            }
            Ok(Event::End(e)) => {
                let name = reader.decoder().decode(e.name()).unwrap_or_default();
                if name == "w:p" || name == "w:table" {
                    if !current_text.is_empty() {
                        text_parts.push(current_text.trim().to_string());
                    }
                    in_paragraph = false;
                }
            }
            Ok(Event::Text(e)) => {
                if in_paragraph {
                    let text = reader.decoder().decode(&e).unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => continue,
            _ => continue,
        }
    }

    let result = text_parts.join("\n");
    let result: String = result.chars().take(max_chars).collect();

    if result.len() >= max_chars {
        Ok(format!("{}\n... [truncated]", result))
    } else {
        Ok(result)
    }
}

pub async fn list_docx_parts(
    path: &Path,
    fs: &crate::fs_service::FileSystemService,
) -> std::result::Result<CallToolResult, CallToolError> {
    let resolved = fs.resolve(path).await.map_err(CallToolError::new)?;

    let file_path = if let Some(unc_root) = &resolved.unc_root {
        unc_root.join(&resolved.rel)
    } else {
        resolved.display.clone()
    };

    let bytes = std::fs::read(file_path).map_err(|e| {
        CallToolError::new(format!("Failed to read DOCX file: {}", e))
    })?;

    let mut cursor = Cursor::new(bytes);
    let mut archive = match zip::ZipArchive::new(&mut cursor) {
        Ok(a) => a,
        Err(e) => {
            return Err(CallToolError::new(format!(
                "Failed to open DOCX as ZIP archive: {}",
                e
            )));
        }
    };

    let mut output = format!("DOCX: {}\nParts: {}\n\n", path.display(), archive.len());

    for i in 0..archive.len() {
        let name = match archive.by_index(i) {
            Ok(f) => f.name().to_string(),
            Err(_) => continue,
        };
        if name.ends_with(".xml") || name.ends_with(".rels") {
            output.push_str(&format!("  {}\n", name));
        } else if name.starts_with("word/media/") {
            output.push_str(&format!("  {} (image)\n", name));
        }
    }

    Ok(CallToolResult::text_content(vec![TextContent::from(output.trim().to_string())]))
}
