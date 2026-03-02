use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use serde_json::json;

mod models;
mod parser;
mod ai;
mod documents;

use models::AnalysisVerdict;
use ai::contact_extractor::{extract_candidate_contact_info, extract_hr_contact_info};

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/analyze", post(analyze_handler))
        .route("/api/cover-letter", post(cover_letter_handler))
        .route("/api/email-reply", post(email_reply_handler))
        .route("/api/documents/export", post(document_export_handler))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();
    
    println!("🚀 Server running on http://localhost:{}", port);
    println!("📡 API endpoints:");
    println!("  - POST /api/analyze (Resume analysis)");
    println!("  - POST /api/cover-letter (Generate cover letter)");
    println!("  - POST /api/email-reply (Generate email reply)");
    println!("  - POST /api/documents/export (Export documents)");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "AI Resume Scanner API v1.0"
}

async fn analyze_handler(mut multipart: Multipart) -> Json<AnalysisVerdict> {
    let mut cv_text = String::new();
    let mut jd_text = String::new();
    
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();
        
        match name.as_str() {
            "cv" => {
                // Detect file type and parse
                if data.starts_with(b"%PDF") {
                    println!("📄 Parsing PDF file...");
                    cv_text = parser::pdf::extract_pdf_text(&data)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("PDF parse error: {}", e);
                            String::from("Error parsing PDF")
                        });
                } else if data.len() > 4 && &data[0..4] == b"PK\x03\x04" {
                    // DOCX is a ZIP file (starts with PK)
                    println!("📄 Parsing DOCX file...");
                    cv_text = parser::docx::extract_docx_text(&data)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("DOCX parse error: {}", e);
                            String::from("Error parsing DOCX")
                        });
                } else {
                    // Try as plain text
                    cv_text = String::from_utf8_lossy(&data).to_string();
                }
            }
            "jd" => {
                jd_text = String::from_utf8_lossy(&data).to_string();
            }
            _ => {}
        }
    }
    
    println!("🤖 Analyzing resume...");
    println!("CV length: {} chars", cv_text.len());
    println!("JD length: {} chars", jd_text.len());
    
    let verdict = ai::agent::analyze_resume(&cv_text, &jd_text)
        .await
        .unwrap_or_default();
    
    println!("✅ Analysis complete: {}% match", verdict.overall_score);
    
    Json(verdict)
}

async fn cover_letter_handler(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut cv_text = String::new();
    let mut jd_text = String::new();
    let mut candidate_name = String::new();
    let mut candidate_email = String::new();
    let mut company_name = String::new();
    let mut hiring_manager_name = None;
    let mut position_title = String::new();
    let mut language: Option<ai::cover_letter::CoverLetterLanguage> = None;
    
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();
        
        match name.as_str() {
            "cv" => {
                if data.starts_with(b"%PDF") {
                    cv_text = parser::pdf::extract_pdf_text(&data)
                        .await
                        .unwrap_or_default();
                } else if data.len() > 4 && &data[0..4] == b"PK\x03\x04" {
                    cv_text = parser::docx::extract_docx_text(&data)
                        .await
                        .unwrap_or_default();
                } else {
                    cv_text = String::from_utf8_lossy(&data).to_string();
                }
            }
            "jd" => {
                if data.starts_with(b"%PDF") {
                    jd_text = parser::pdf::extract_pdf_text(&data)
                        .await
                        .unwrap_or_default();
                } else if data.len() > 4 && &data[0..4] == b"PK\x03\x04" {
                    jd_text = parser::docx::extract_docx_text(&data)
                        .await
                        .unwrap_or_default();
                } else {
                    jd_text = String::from_utf8_lossy(&data).to_string();
                }
            }
            "candidate_name" => candidate_name = String::from_utf8_lossy(&data).to_string(),
            "candidate_email" => candidate_email = String::from_utf8_lossy(&data).to_string(),
            "company_name" => company_name = String::from_utf8_lossy(&data).to_string(),
            "hiring_manager_name" => {
                let name_str = String::from_utf8_lossy(&data).to_string();
                if !name_str.is_empty() {
                    hiring_manager_name = Some(name_str);
                }
            }
            "position_title" => position_title = String::from_utf8_lossy(&data).to_string(),
            "language" => {
                let lang_str = String::from_utf8_lossy(&data).to_string().to_lowercase();
                language = match lang_str.as_str() {
                    "vietnamese" | "vi" | "tiếng việt" => Some(ai::cover_letter::CoverLetterLanguage::Vietnamese),
                    "english" | "en" => Some(ai::cover_letter::CoverLetterLanguage::English),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    
    if cv_text.is_empty() || jd_text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Missing required fields: cv, jd"
            }))
        ).into_response();
    }

    let candidate_info = extract_candidate_contact_info(&cv_text);
    let hr_info = extract_hr_contact_info(&jd_text);

    if candidate_name.trim().is_empty() {
        candidate_name = candidate_info
            .full_name
            .clone()
            .unwrap_or_else(|| "Ung vien".to_string());
    }
    if candidate_email.trim().is_empty() {
        candidate_email = candidate_info
            .email
            .clone()
            .unwrap_or_else(|| "candidate@example.com".to_string());
    }
    if company_name.trim().is_empty() {
        company_name = hr_info
            .company_name
            .clone()
            .unwrap_or_else(|| "Cong ty tuyen dung".to_string());
    }
    if position_title.trim().is_empty() {
        position_title = hr_info
            .position_title
            .clone()
            .unwrap_or_else(|| "Vi tri ung tuyen".to_string());
    }
    if hiring_manager_name.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        hiring_manager_name = hr_info.full_name.clone();
    }

    let candidate_phone = candidate_info.phone.clone();
    let candidate_personal_site = candidate_info.personal_site.clone();
    let candidate_address = candidate_info.address.clone();
    let company_website = hr_info.company_website.clone();
    let company_address = hr_info.company_address.clone();
    let hiring_manager_phone = hr_info.phone.clone();
    let hiring_manager_email = hr_info.email.clone();

    let request = ai::cover_letter::CoverLetterRequest {
        cv_text,
        jd_text,
        candidate_name,
        candidate_email,
        candidate_phone,
        candidate_personal_site,
        candidate_address,
        company_name,
        company_website,
        company_address,
        hiring_manager_name,
        hiring_manager_phone,
        hiring_manager_email,
        position_title,
        language,
    };
    
    match ai::cover_letter::generate_cover_letter(request).await {
        Ok(response) => {
            println!("✅ Cover letter generated");
            Json(response).into_response()
        }
        Err(e) => {
            eprintln!("❌ Cover letter generation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to generate cover letter: {}", e) }))
            ).into_response()
        }
    }
}

async fn email_reply_handler(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut cv_text = String::new();
    let mut jd_text = String::new();
    let mut candidate_name = String::new();
    let mut candidate_email = String::new();
    let mut company_name = String::new();
    let mut position_title = String::new();
    let mut recipient_email = None;
    let mut recipient_name = None;
    let mut email_type_str = "initial_application".to_string();
    let mut language: Option<ai::email_reply::EmailLanguage> = None;
    
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();
        
        match name.as_str() {
            "cv" => {
                if data.starts_with(b"%PDF") {
                    cv_text = parser::pdf::extract_pdf_text(&data).await.unwrap_or_default();
                } else if data.len() > 4 && &data[0..4] == b"PK\x03\x04" {
                    cv_text = parser::docx::extract_docx_text(&data).await.unwrap_or_default();
                } else {
                    cv_text = String::from_utf8_lossy(&data).to_string();
                }
            }
            "jd" => {
                if data.starts_with(b"%PDF") {
                    jd_text = parser::pdf::extract_pdf_text(&data).await.unwrap_or_default();
                } else if data.len() > 4 && &data[0..4] == b"PK\x03\x04" {
                    jd_text = parser::docx::extract_docx_text(&data).await.unwrap_or_default();
                } else {
                    jd_text = String::from_utf8_lossy(&data).to_string();
                }
            }
            "candidate_name" => candidate_name = String::from_utf8_lossy(&data).to_string(),
            "candidate_email" => candidate_email = String::from_utf8_lossy(&data).to_string(),
            "company_name" => company_name = String::from_utf8_lossy(&data).to_string(),
            "position_title" => position_title = String::from_utf8_lossy(&data).to_string(),
            "recipient_email" => {
                let email_str = String::from_utf8_lossy(&data).to_string();
                if !email_str.is_empty() {
                    recipient_email = Some(email_str);
                }
            }
            "recipient_name" => {
                let name_str = String::from_utf8_lossy(&data).to_string();
                if !name_str.is_empty() {
                    recipient_name = Some(name_str);
                }
            }
            "email_type" => email_type_str = String::from_utf8_lossy(&data).to_string(),
            "language" => {
                let lang_str = String::from_utf8_lossy(&data).to_string().to_lowercase();
                language = match lang_str.as_str() {
                    "vietnamese" | "vi" | "tiếng việt" => Some(ai::email_reply::EmailLanguage::Vietnamese),
                    "english" | "en" => Some(ai::email_reply::EmailLanguage::English),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    
    if cv_text.is_empty() || jd_text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Missing required fields: cv, jd"
            }))
        ).into_response();
    }

    let candidate_info = extract_candidate_contact_info(&cv_text);
    let hr_info = extract_hr_contact_info(&jd_text);

    if candidate_name.trim().is_empty() {
        candidate_name = candidate_info
            .full_name
            .clone()
            .unwrap_or_else(|| "Ung vien".to_string());
    }
    if candidate_email.trim().is_empty() {
        candidate_email = candidate_info
            .email
            .clone()
            .unwrap_or_else(|| "candidate@example.com".to_string());
    }
    if company_name.trim().is_empty() {
        company_name = hr_info
            .company_name
            .clone()
            .unwrap_or_else(|| "Cong ty tuyen dung".to_string());
    }
    if position_title.trim().is_empty() {
        position_title = hr_info
            .position_title
            .clone()
            .unwrap_or_else(|| "Vi tri ung tuyen".to_string());
    }
    if recipient_name.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        recipient_name = hr_info.full_name.clone();
    }
    if recipient_email.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        recipient_email = hr_info.email.clone();
    }

    let candidate_phone = candidate_info.phone.clone();
    let candidate_personal_site = candidate_info.personal_site.clone();
    let candidate_address = candidate_info.address.clone();
    let company_website = hr_info.company_website.clone();
    let company_address = hr_info.company_address.clone();
    let recipient_phone = hr_info.phone.clone();

    let email_type = match email_type_str.as_str() {
        "interview_followup" => ai::email_reply::EmailType::InterviewFollowup,
        "offer_response" => ai::email_reply::EmailType::OfferResponse,
        "negotiation" => ai::email_reply::EmailType::Negotiation,
        "decline" => ai::email_reply::EmailType::Decline,
        _ => ai::email_reply::EmailType::InitialApplication,
    };
    
    let request = ai::email_reply::EmailReplyRequest {
        cv_text,
        jd_text,
        candidate_name,
        candidate_email,
        candidate_phone,
        candidate_personal_site,
        candidate_address,
        company_name,
        company_website,
        company_address,
        position_title,
        recipient_email,
        recipient_name,
        recipient_phone,
        email_type,
        language,
    };
    
    match ai::email_reply::generate_email_reply(request).await {
        Ok(response) => {
            println!("✅ Email reply generated");
            Json(response).into_response()
        }
        Err(e) => {
            eprintln!("❌ Email reply generation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to generate email reply: {}", e) }))
            ).into_response()
        }
    }
}

async fn document_export_handler(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut document_type = String::new();
    let mut title = String::new();
    let mut content = String::new();
    let mut author = String::new();
    let mut format_str = "html".to_string();
    
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();
        let value = String::from_utf8_lossy(&data).to_string();
        
        match name.as_str() {
            "document_type" => document_type = value,
            "title" => title = value,
            "content" => content = value,
            "author" => author = value,
            "format" => format_str = value,
            _ => {}
        }
    }
    
    // Validate required fields
    if document_type.is_empty() || content.is_empty() || author.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields: document_type, content, author" }))
        ).into_response();
    }
    
    let format = match format_str.to_lowercase().as_str() {
        "pdf" => documents::ExportFormat::PDF,
        "docx" => documents::ExportFormat::DOCX,
        _ => documents::ExportFormat::HTML,
    };
    
    let request = documents::DocumentExportRequest {
        document_type,
        title: title.clone(),
        content,
        author,
        format,
    };
    
    match documents::export_document(request) {
        Ok(response) => {
            println!("✅ Document exported: {}", response.filename);
            // Return binary data with appropriate headers
            (
                StatusCode::OK,
                [(
                    axum::http::header::CONTENT_TYPE,
                    response.mime_type,
                ), (
                    axum::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", response.filename),
                )],
                response.data,
            ).into_response()
        }
        Err(e) => {
            eprintln!("❌ Document export failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to export document: {}", e) }))
            ).into_response()
        }
    }
}
