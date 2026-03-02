/// DOCUMENT EXPORT MODULE
/// ======================
/// Export cover letters and emails to various formats (PDF, DOCX, HTML)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    PDF,
    DOCX,
    HTML,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExportRequest {
    pub document_type: String,  // "cover_letter" or "email"
    pub title: String,
    pub content: String,
    pub author: String,
    pub format: ExportFormat,
}

#[derive(Debug, Clone)]
pub struct DocumentExportResponse {
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

// ============================================
// EXPORT FUNCTIONS
// ============================================

pub fn export_document(request: DocumentExportRequest) -> Result<DocumentExportResponse> {
    match request.format {
        ExportFormat::PDF => export_as_pdf(&request),
        ExportFormat::DOCX => export_as_docx(&request),
        ExportFormat::HTML => export_as_html(&request),
    }
}

fn export_as_pdf(request: &DocumentExportRequest) -> Result<DocumentExportResponse> {
    // For now, return HTML as PDF is complex
    // In production, use a library like wkhtmltopdf or printpdf
    eprintln!("PDF export: converting to simple HTML for now");
    
    let html_content = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; max-width: 800px; margin: 40px; }}
        h2 {{ color: #333; border-bottom: 2px solid #007bff; padding-bottom: 10px; }}
        p {{ margin: 10px 0; text-align: justify; }}
    </style>
</head>
<body>
    <h2>{}</h2>
    <p>By: {}</p>
    <hr>
    <div style="white-space: pre-wrap;">
{}
    </div>
</body>
</html>
"#,
        request.title,
        request.author,
        request.content
    );

    let filename = format!("{}.html", sanitize_filename(&request.title));
    
    Ok(DocumentExportResponse {
        filename,
        mime_type: "text/html".to_string(),
        data: html_content.into_bytes(),
    })
}

fn export_as_docx(request: &DocumentExportRequest) -> Result<DocumentExportResponse> {
    // Using docx-rs crate
    use docx_rs::*;
    
    // Create document
    let mut doc = Document::new();
    
    // Add title
    doc = doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(&request.title)
                .bold()
                .size(24))
    );
    
    // Add author
    doc = doc.add_paragraph(
        Paragraph::new().add_run(Run::new().add_text(format!("By: {}", request.author)))
    );
    
    // Add horizontal line
    doc = doc.add_paragraph(Paragraph::new());
    
    // Add content paragraphs
    for paragraph_text in request.content.split("\n\n") {
        if !paragraph_text.trim().is_empty() {
            doc = doc.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(paragraph_text))
            );
        }
    }
    
    // Convert to bytes
    let bytes = doc.build();
    
    let filename = format!("{}.docx", sanitize_filename(&request.title));
    
    Ok(DocumentExportResponse {
        filename,
        mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        data: bytes,
    })
}

fn export_as_html(request: &DocumentExportRequest) -> Result<DocumentExportResponse> {
    let html_content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background-color: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 3px solid #007bff;
            padding-bottom: 10px;
            margin-bottom: 20px;
        }}
        .author {{
            color: #666;
            font-style: italic;
            margin-bottom: 20px;
        }}
        .content {{
            color: #333;
            text-align: justify;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
        .content p {{
            margin-bottom: 15px;
        }}
        .footer {{
            margin-top: 40px;
            text-align: center;
            color: #999;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
        <div class="author">By: {}</div>
        <div class="content">
{}
        </div>
        <div class="footer">
            <p>Generated by Resume Scanner - Professional Application Tools</p>
            <p>Date: {}</p>
        </div>
    </div>
</body>
</html>
"#,
        request.title,
        request.title,
        request.author,
        request.content
            .split('\n')
            .map(|line| {
                if line.trim().is_empty() {
                    "<br>".to_string()
                } else {
                    format!("{}\n", line)
                }
            })
            .collect::<String>(),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let filename = format!("{}.html", sanitize_filename(&request.title));
    
    Ok(DocumentExportResponse {
        filename,
        mime_type: "text/html; charset=utf-8".to_string(),
        data: html_content.into_bytes(),
    })
}

// ============================================
// HELPER FUNCTIONS
// ============================================

fn sanitize_filename(name: &str) -> String {
    name
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
        .to_lowercase()
        .trim_matches('_')
        .to_string()
}
