use anyhow::{Result, anyhow};
use std::io::{Cursor, Read};
use zip::ZipArchive;
use xml::reader::{EventReader, XmlEvent};

pub async fn extract_docx_text(bytes: &[u8]) -> Result<String> {
    // DOCX is a ZIP file containing XML documents
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;
    
    // Extract text from word/document.xml
    let mut document_xml = archive.by_name("word/document.xml")
        .map_err(|e| anyhow!("Failed to find document.xml: {}", e))?;
    
    let mut xml_content = String::new();
    document_xml.read_to_string(&mut xml_content)?;
    
    // Parse XML and extract text from <w:t> elements
    let parser = EventReader::from_str(&xml_content);
    let mut text_content = String::new();
    let mut in_text_element = false;
    
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "t" {
                    in_text_element = true;
                }
            }
            Ok(XmlEvent::Characters(text)) => {
                if in_text_element {
                    text_content.push_str(&text);
                    text_content.push(' ');
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "t" {
                    in_text_element = false;
                }
            }
            Err(e) => {
                eprintln!("XML parsing error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(text_content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docx_extraction() {
        // Test would require valid DOCX bytes
        let result = extract_docx_text(b"invalid").await;
        assert!(result.is_err()); // Should fail with invalid data
    }
}
