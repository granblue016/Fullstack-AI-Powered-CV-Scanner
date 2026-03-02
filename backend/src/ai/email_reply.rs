/// EMAIL REPLY GENERATION MODULE
/// =============================
/// Generates professional email replies to job offers and follow-ups

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

// ============================================
// DATA STRUCTURES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailReplyRequest {
    pub cv_text: String,
    pub jd_text: String,
    pub candidate_name: String,
    pub candidate_email: String,
    pub candidate_phone: Option<String>,
    pub candidate_personal_site: Option<String>,
    pub candidate_address: Option<String>,
    pub company_name: String,
    pub company_website: Option<String>,
    pub company_address: Option<String>,
    pub position_title: String,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub recipient_phone: Option<String>,
    pub email_type: EmailType,  // Initial application, Interview follow-up, Offer response, etc.
    pub language: Option<EmailLanguage>,  // Vietnamese or English
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailType {
    #[serde(rename = "initial_application")]
    InitialApplication,
    #[serde(rename = "interview_followup")]
    InterviewFollowup,
    #[serde(rename = "offer_response")]
    OfferResponse,
    #[serde(rename = "negotiation")]
    Negotiation,
    #[serde(rename = "decline")]
    Decline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailLanguage {
    #[serde(rename = "vietnamese")]
    Vietnamese,
    #[serde(rename = "english")]
    English,
}

impl Default for EmailLanguage {
    fn default() -> Self {
        EmailLanguage::English
    }
}

impl EmailLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailLanguage::Vietnamese => "Tiếng Việt",
            EmailLanguage::English => "English",
        }
    }
}

impl EmailType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailType::InitialApplication => "Initial Application",
            EmailType::InterviewFollowup => "Interview Follow-up",
            EmailType::OfferResponse => "Offer Response",
            EmailType::Negotiation => "Salary Negotiation",
            EmailType::Decline => "Job Decline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailReplyResponse {
    pub id: String,
    pub candidate_name: String,
    pub company_name: String,
    pub position_title: String,
    pub recipient_email: Option<String>,
    pub subject: String,
    pub body: String,  // Plain text version
    pub html_body: String,  // HTML version for display
    pub tone: String,  // professional, enthusiastic, etc.
    pub email_type: String,
    pub draft: bool,  // Is this a draft or ready to send
    pub created_at: String,
}

// ============================================
// MAIN INTERFACE
// ============================================

/// Generate an email reply using Gemini API
pub async fn generate_email_reply(
    request: EmailReplyRequest,
) -> Result<EmailReplyResponse> {
    let api_key = resolve_gemini_api_key()?;

    // Prepare the prompt for Gemini
    let prompt = build_email_prompt(
        &request,
    );

    // Call Gemini API
    let email_content = match call_gemini_api(&api_key, &prompt).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("⚠️ Gemini email reply failed, using local fallback: {}", e);
            generate_email_fallback(&request)
        }
    };

    // Parse the email content (expects format: SUBJECT: xxx\n\nBODY: xxx)
    let (subject, body) = parse_email_content(&email_content)?;

    // Convert to HTML
    let html_body = convert_email_to_html(&body);

    Ok(EmailReplyResponse {
        id: Uuid::new_v4().to_string(),
        candidate_name: request.candidate_name,
        company_name: request.company_name,
        position_title: request.position_title,
        recipient_email: request.recipient_email,
        subject,
        body,
        html_body,
        tone: "Professional".to_string(),
        email_type: request.email_type.as_str().to_string(),
        draft: true,  // Always start as draft for user review
        created_at: Utc::now().to_rfc3339(),
    })
}

// ============================================
// HELPER FUNCTIONS
// ============================================

/// Build the prompt for Gemini API
fn build_email_prompt(request: &EmailReplyRequest) -> String {
    let language = request.language.as_ref().unwrap_or(&EmailLanguage::English);
    
    match language {
        EmailLanguage::Vietnamese => build_vietnamese_email_prompt(request),
        EmailLanguage::English => build_english_email_prompt(request),
    }
}

/// Build Vietnamese email prompt based on user templates
fn build_vietnamese_email_prompt(request: &EmailReplyRequest) -> String {
    let greeting = match &request.recipient_name {
        Some(name) => format!("Kính gửi Anh/Chị {},", name),
        None => "Kính gửi Ban Tuyển dụng,".to_string(),
    };

    let template_guidance = r#"Sử dụng các mẫu email sau đây làm tham khảo:

MẪU 1 - VỊ TRÍ CYBERSECURITY:
Tiêu đề: [Tên ứng viên] - Ứng tuyển vị trí [Tên vị trí]
Nội dung:
- Bày tỏ sự quan tâm sâu sắc đến vị trí
- Nêu nền tảng kiến thức và kinh nghiệm thực chiến
- Liệt kê 3 thế mạnh chính
- Đính kèm CV và chứng chỉ liên quan
- Mong có cơ hội trao đổi trực tiếp

MẪU 2 - VỊ TRÍ AI/MACHINE LEARNING:
Tiêu đề: [Tên ứng viên] - Ứng tuyển vị trí [Tên vị trí]
Nội dung:
- Giới thiệu bản thân và lĩnh vực theo đuổi
- Ấn tượng với dự án AI của công ty
- Liệt kê giá trị có thể mang lại (3 điểm)
- Hy vọng sớm nhận được phản hồi
- Cảm ơn thời gian xem xét"#;

    format!(
        r#"Tạo một email ứng tuyển chuyên nghiệp bằng TIẾNG VIỆT với thông tin sau:

THÔNG TIN ỨNG VIÊN:
Họ tên: {}
Email: {}
Số điện thoại: {}
Website: {}
Địa chỉ: {}

THÔNG TIN CÔNG TY/HR:
Tên công ty: {}
Website công ty: {}
Địa chỉ công ty: {}
Người nhận: {}
Email người nhận: {}
Vị trí ứng tuyển: {}

MÔ TẢ CÔNG VIỆC:
{}

CV CỦA ỨNG VIÊN:
{}

{}

YÊU CẦU VỀ EMAIL:
1. Lời mở đầu: {}
2. Độ dài: 200-300 từ
3. Giọng điệu: Chuyên nghiệp, lịch sự và chân thành
4. Nội dung:
   - Bày tỏ sự quan tâm đến vị trí
   - Trích dẫn 2-3 kỹ năng/thành tích chính từ CV phù hợp với JD
   - Liệt kê 3 thế mạnh/giá trị mang lại cho công ty
   - Đề cập đính kèm CV và chứng chỉ (nếu có)
   - Mong muốn có cơ hội phỏng vấn/trao đổi
5. Kết thúc: "Trân trọng,\n{}\n{}"

Định dạng email:
SUBJECT: [tiêu đề email bằng tiếng Việt]

BODY: [nội dung email bằng tiếng Việt]

LƯU Ý QUAN TRỌNG:
- Tránh các cụm từ sáo rỗng
- Cụ thể hóa bằng ví dụ và con số từ CV
- Thể hiện sự hiểu biết về ngành/vị trí
- Chia đoạn rõ ràng
- Định dạng văn bản thuần túy (không có HTML tags)

Hãy tạo email hoàn chỉnh theo định dạng trên:
"#,
        request.candidate_name,
        request.candidate_email,
        request.candidate_phone.clone().unwrap_or_else(|| "Chưa cung cấp".to_string()),
        request.candidate_personal_site.clone().unwrap_or_else(|| "Chưa có".to_string()),
        request.candidate_address.clone().unwrap_or_else(|| "Chưa cung cấp".to_string()),
        request.company_name,
        request.company_website.clone().unwrap_or_else(|| "Chưa có".to_string()),
        request.company_address.clone().unwrap_or_else(|| "Chưa có".to_string()),
        request.recipient_name.clone().unwrap_or_else(|| "Ban Tuyển dụng".to_string()),
        request.recipient_email.clone().unwrap_or_else(|| "Chưa có".to_string()),
        request.position_title,
        request.jd_text,
        request.cv_text,
        template_guidance,
        greeting,
        request.candidate_name,
        request.candidate_email
    )
}

/// Build English email prompt (original)
fn build_english_email_prompt(request: &EmailReplyRequest) -> String {
    let greeting = match &request.recipient_name {
        Some(name) => format!("Dear {},", name),
        None => "Dear Hiring Manager,".to_string(),
    };

    let type_description = match &request.email_type {
        EmailType::InitialApplication => {
            "Write an enthusiastic and professional initial application email expressing interest in the position."
        }
        EmailType::InterviewFollowup => {
            "Write a professional follow-up email after an interview, reiterating interest and highlighting key conversation points."
        }
        EmailType::OfferResponse => {
            "Write a professional email accepting the job offer with enthusiasm."
        }
        EmailType::Negotiation => {
            "Write a professional email proposing salary negotiation, highlighting value and market research."
        }
        EmailType::Decline => {
            "Write a respectful and professional email declining the job offer, expressing gratitude."
        }
    };

    format!(
        r#"Generate a professional email for the following scenario:

TYPE OF EMAIL: {}

CANDIDATE INFORMATION:
Name: {}
Email: {}
Phone: {}
Personal site: {}
Address: {}

HR / COMPANY INFORMATION:
Company: {}
Company website: {}
Company address: {}
Recipient: {}
Recipient email: {}
Recipient phone: {}
Position: {}

JOB DESCRIPTION:
{}

CANDIDATE'S CV:
{}

Write a professional email with these requirements:
1. Start with: {}
2. Email type requirements: {}
3. Length: 150-250 words
4. Tone: Professional, warm, and genuine
5. Include specific details from CV and JD where relevant
6. End with professional closing: "Best regards, {} ({})"

The email should be formatted as:
SUBJECT: [email subject line]

BODY: [email body text]

IMPORTANT: 
- Avoid generic phrases
- Be specific with achievements and examples
- Show enthusiasm and genuine interest
- Use clear paragraphs
- Plain text format (no HTML tags)
- Professional but personable tone

Please generate the complete email in the format specified above:
"#,
        request.email_type.as_str(),
        request.candidate_name,
        request.candidate_email,
        request.candidate_phone.clone().unwrap_or_else(|| "N/A".to_string()),
        request.candidate_personal_site.clone().unwrap_or_else(|| "N/A".to_string()),
        request.candidate_address.clone().unwrap_or_else(|| "N/A".to_string()),
        request.company_name,
        request.company_website.clone().unwrap_or_else(|| "N/A".to_string()),
        request.company_address.clone().unwrap_or_else(|| "N/A".to_string()),
        request.recipient_name.clone().unwrap_or_else(|| "Hiring Manager".to_string()),
        request.recipient_email.clone().unwrap_or_else(|| "N/A".to_string()),
        request.recipient_phone.clone().unwrap_or_else(|| "N/A".to_string()),
        request.position_title,
        request.jd_text,
        request.cv_text,
        greeting,
        type_description,
        request.candidate_name,
        request.candidate_email
    )
}

/// Call Gemini API
async fn call_gemini_api(api_key: &str, prompt: &str) -> Result<String> {
    #[derive(Serialize)]
    struct GeminiRequest {
        contents: Vec<GeminiContent>,
    }

    #[derive(Serialize)]
    struct GeminiContent {
        parts: Vec<GeminiPart>,
    }

    #[derive(Serialize)]
    struct GeminiPart {
        text: String,
    }

    #[derive(Deserialize)]
    struct GeminiResponse {
        candidates: Vec<GeminiCandidate>,
    }

    #[derive(Deserialize)]
    struct GeminiCandidate {
        content: GeminiResponseContent,
    }

    #[derive(Deserialize)]
    struct GeminiResponseContent {
        parts: Vec<GeminiResponsePart>,
    }

    #[derive(Deserialize)]
    struct GeminiResponsePart {
        text: String,
    }

    let client = reqwest::Client::new();
    let models = resolve_gemini_models();
    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: prompt.to_string(),
            }],
        }],
    };

    let mut last_error = String::new();

    for model in models {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        let response = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.post(&url).json(&request).send(),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                last_error = format!("Gemini request error on model {}: {}", model, e);
                continue;
            }
            Err(_) => {
                last_error = format!("Gemini timeout on model {}", model);
                continue;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            last_error = format!(
                "Gemini API error on model {} ({}): {}",
                model,
                status,
                truncate_error_text(&error_body)
            );
            continue;
        }

        let data: GeminiResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                last_error = format!("Failed to parse Gemini response on model {}: {}", model, e);
                continue;
            }
        };

        if let Some(text) = data
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
        {
            return Ok(text);
        }

        last_error = format!("Empty Gemini response on model {}", model);
    }

    Err(anyhow!("{}", last_error))
}

fn resolve_gemini_api_key() -> Result<String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| anyhow!("Missing Gemini key. Set GEMINI_API_KEY or GOOGLE_API_KEY in backend/.env"))?;

    if api_key.trim().is_empty() {
        return Err(anyhow!("Gemini key is empty. Set GEMINI_API_KEY or GOOGLE_API_KEY in backend/.env"));
    }

    Ok(api_key)
}

fn resolve_gemini_models() -> Vec<String> {
    if let Ok(models) = std::env::var("GEMINI_MODEL") {
        let parsed: Vec<String> = models
            .split(',')
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .collect();

        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![
        "gemini-1.5-flash".to_string(),
        "gemini-1.5-pro".to_string(),
        "gemini-2.0-flash".to_string(),
    ]
}

fn truncate_error_text(text: &str) -> String {
    let max_len = 220;
    if text.chars().count() <= max_len {
        return text.to_string();
    }

    let trimmed: String = text.chars().take(max_len).collect();
    format!("{}...", trimmed)
}

fn generate_email_fallback(request: &EmailReplyRequest) -> String {
    let language = request.language.as_ref().unwrap_or(&EmailLanguage::English);
    
    match language {
        EmailLanguage::Vietnamese => generate_vietnamese_email_fallback(request),
        EmailLanguage::English => generate_english_email_fallback(request),
    }
}

fn generate_vietnamese_email_fallback(request: &EmailReplyRequest) -> String {
    let (subject, purpose) = match request.email_type {
        EmailType::InitialApplication => (
            format!("[{}] - Ứng tuyển vị trí {}", request.candidate_name, request.position_title),
            "Tôi viết email này để bày tỏ sự quan tâm sâu sắc đến vị trí mà Quý công ty đang đăng tuyển.",
        ),
        EmailType::InterviewFollowup => (
            format!("Cảm ơn - Phỏng vấn vị trí {}", request.position_title),
            "Cảm ơn Anh/Chị đã dành thời gian phỏng vấn. Tôi rất hứng thú với vị trí và đội ngũ của Quý công ty.",
        ),
        EmailType::OfferResponse => (
            format!("Phản hồi đề nghị - {}", request.position_title),
            "Cảm ơn Anh/Chị về lời đề nghị. Tôi rất hân hạnh khi được làm việc cùng đội ngũ của Quý công ty.",
        ),
        EmailType::Negotiation => (
            format!("Về chi tiết đề nghị - {}", request.position_title),
            "Cảm ơn Anh/Chị về lời đề nghị. Tôi muốn trao đổi thêm một số chi tiết để đảm bảo sự phù hợp.",
        ),
        EmailType::Decline => (
            format!("Cảm ơn về cơ hội - {}", request.position_title),
            "Cảm ơn Anh/Chị về lời đề nghị và thời gian dành cho tôi trong suốt quá trình.",
        ),
    };

    let greeting = request
        .recipient_name
        .as_ref()
        .map(|name| format!("Kính gửi Anh/Chị {},", name))
        .unwrap_or_else(|| "Kính gửi Ban Tuyển dụng,".to_string());

    let body = format!(
        "{}\n\n{}\n\nVới nền tảng kiến thức và kinh nghiệm của mình, tôi tự tin có thể đóng góp hiệu quả vào vị trí {} tại {}. Những thế mạnh chính của tôi bao gồm các kỹ năng chuyên môn và kinh nghiệm thực tế đã được nêu trong CV.\n\nTôi đã đính kèm CV và các chứng chỉ liên quan để Quý công ty tiện tham khảo. Rất mong có cơ hội được trao đổi trực tiếp về cách tôi có thể hỗ trợ đội ngũ của công ty.\n\nTrân trọng,\n{}\n{}",
        greeting,
        purpose,
        request.position_title,
        request.company_name,
        request.candidate_name,
        request.candidate_email
    );

    format!("SUBJECT: {}\n\nBODY: {}", subject, body)
}

fn generate_english_email_fallback(request: &EmailReplyRequest) -> String {
    let (subject, purpose) = match request.email_type {
        EmailType::InitialApplication => (
            format!("Application for {} - {}", request.position_title, request.candidate_name),
            "I am excited to apply for this opportunity and share how my background aligns with your requirements.",
        ),
        EmailType::InterviewFollowup => (
            format!("Thank You - Interview for {}", request.position_title),
            "Thank you for the interview opportunity. I remain very interested in the role and your team.",
        ),
        EmailType::OfferResponse => (
            format!("Response to Offer - {}", request.position_title),
            "Thank you for the offer. I appreciate your trust and am pleased to respond professionally.",
        ),
        EmailType::Negotiation => (
            format!("Regarding Offer Details - {}", request.position_title),
            "Thank you for the offer. I would like to discuss a few details to ensure mutual fit.",
        ),
        EmailType::Decline => (
            format!("Thank You for the Opportunity - {}", request.position_title),
            "Thank you for the offer and your time throughout the process.",
        ),
    };

    let greeting = request
        .recipient_name
        .as_ref()
        .map(|name| format!("Dear {},", name))
        .unwrap_or_else(|| "Dear Hiring Manager,".to_string());

    let body = format!(
        "{}\n\n{}\n\nI am confident that my skills and experience are a strong match for the {} position at {}. I would be glad to provide any additional information if needed.\n\nBest regards,\n{} ({})",
        greeting,
        purpose,
        request.position_title,
        request.company_name,
        request.candidate_name,
        request.candidate_email
    );

    format!("SUBJECT: {}\n\nBODY: {}", subject, body)
}

/// Parse email content in format "SUBJECT: xxx\n\nBODY: xxx"
fn parse_email_content(content: &str) -> Result<(String, String)> {
    // Find SUBJECT: and BODY: markers
    let subject_start = content.find("SUBJECT:").ok_or_else(|| anyhow!("No SUBJECT found"))?;
    let body_start = content.find("BODY:").ok_or_else(|| anyhow!("No BODY found"))?;
    
    // Extract subject (between SUBJECT: and next newline or BODY:)
    let subject_text = &content[subject_start + 8..body_start];
    let subject = subject_text
        .trim()
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    
    // Extract body (everything after BODY:)
    let body = content[body_start + 5..].trim().to_string();
    
    Ok((subject, body))
}

/// Convert plain text email to HTML
fn convert_email_to_html(text: &str) -> String {
    let mut html = String::from(
        r#"<div style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px;">
"#
    );
    
    // Split into paragraphs
    for paragraph in text.split("\n\n") {
        if !paragraph.trim().is_empty() {
            html.push_str(&format!(
                "<p style=\"margin: 15px 0;\">{}</p>\n",
                paragraph.trim()
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;")
            ));
        }
    }
    
    html.push_str("</div>");
    html
}
