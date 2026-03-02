/// COVER LETTER GENERATION MODULE
/// ==============================
/// Generates professional cover letters based on CV and JD analysis

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

// ============================================
// DATA STRUCTURES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverLetterLanguage {
    #[serde(rename = "vietnamese")]
    Vietnamese,
    #[serde(rename = "english")]
    English,
}

impl Default for CoverLetterLanguage {
    fn default() -> Self {
        CoverLetterLanguage::English
    }
}

impl CoverLetterLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoverLetterLanguage::Vietnamese => "Tiếng Việt",
            CoverLetterLanguage::English => "English",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverLetterRequest {
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
    pub hiring_manager_name: Option<String>,
    pub hiring_manager_phone: Option<String>,
    pub hiring_manager_email: Option<String>,
    pub position_title: String,
    pub language: Option<CoverLetterLanguage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverLetterResponse {
    pub id: String,
    pub candidate_name: String,
    pub company_name: String,
    pub position_title: String,
    pub content: String,  // Plain text version
    pub html_content: String,  // HTML version for display
    pub key_points: Vec<String>,  // Key selling points
    pub estimated_match_score: u8,  // 0-100
    pub created_at: String,
}

// ============================================
// MAIN INTERFACE
// ============================================

/// Generate a cover letter using Gemini API
pub async fn generate_cover_letter(
    request: CoverLetterRequest,
) -> Result<CoverLetterResponse> {
    let api_key = resolve_gemini_api_key()?;

    // Prepare the prompt for Gemini
    let prompt = build_cover_letter_prompt(
        &request.cv_text,
        &request.jd_text,
        &request.candidate_name,
        &request.candidate_email,
        &request.candidate_phone,
        &request.candidate_personal_site,
        &request.candidate_address,
        &request.company_name,
        &request.company_website,
        &request.company_address,
        &request.hiring_manager_name,
        &request.hiring_manager_phone,
        &request.hiring_manager_email,
        &request.position_title,
        &request.language,
    );

    // Call Gemini API
    let cover_letter_text = match call_gemini_api(&api_key, &prompt).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("⚠️ Gemini cover letter failed, using local fallback: {}", e);
            generate_cover_letter_fallback(&request)
        }
    };

    // Extract key points from the cover letter
    let key_points = extract_key_points(&cover_letter_text);

    // Estimate match score based on content
    let match_score = estimate_match_score_from_letter(&cover_letter_text, &request.jd_text);

    // Convert to HTML
    let html_content = convert_to_html(&cover_letter_text);

    Ok(CoverLetterResponse {
        id: Uuid::new_v4().to_string(),
        candidate_name: request.candidate_name,
        company_name: request.company_name,
        position_title: request.position_title,
        content: cover_letter_text,
        html_content,
        key_points,
        estimated_match_score: match_score,
        created_at: Utc::now().to_rfc3339(),
    })
}

// ============================================
// HELPER FUNCTIONS
// ============================================

/// Build the prompt for Gemini API
fn build_cover_letter_prompt(
    cv_text: &str,
    jd_text: &str,
    candidate_name: &str,
    candidate_email: &str,
    candidate_phone: &Option<String>,
    candidate_personal_site: &Option<String>,
    candidate_address: &Option<String>,
    company_name: &str,
    company_website: &Option<String>,
    company_address: &Option<String>,
    hiring_manager_name: &Option<String>,
    hiring_manager_phone: &Option<String>,
    hiring_manager_email: &Option<String>,
    position_title: &str,
    language: &Option<CoverLetterLanguage>,
) -> String {
    let lang = language.as_ref().unwrap_or(&CoverLetterLanguage::English);
    
    match lang {
        CoverLetterLanguage::Vietnamese => build_vietnamese_cover_letter_prompt(
            cv_text, jd_text, candidate_name, candidate_email, candidate_phone,
            candidate_personal_site, candidate_address, company_name, company_website,
            company_address, hiring_manager_name, hiring_manager_phone,
            hiring_manager_email, position_title,
        ),
        CoverLetterLanguage::English => build_english_cover_letter_prompt(
            cv_text, jd_text, candidate_name, candidate_email, candidate_phone,
            candidate_personal_site, candidate_address, company_name, company_website,
            company_address, hiring_manager_name, hiring_manager_phone,
            hiring_manager_email, position_title,
        ),
    }
}

/// Build Vietnamese cover letter prompt
fn build_vietnamese_cover_letter_prompt(
    cv_text: &str,
    jd_text: &str,
    candidate_name: &str,
    candidate_email: &str,
    candidate_phone: &Option<String>,
    candidate_personal_site: &Option<String>,
    candidate_address: &Option<String>,
    company_name: &str,
    company_website: &Option<String>,
    company_address: &Option<String>,
    hiring_manager_name: &Option<String>,
    hiring_manager_phone: &Option<String>,
    hiring_manager_email: &Option<String>,
    position_title: &str,
) -> String {
    let hiring_greeting = match hiring_manager_name {
        Some(name) => format!("Kính gửi Anh/Chị {},", name),
        None => "Kính gửi Ban Tuyển dụng,".to_string(),
    };

    let template_structure = r#"MẪU CẤU TRÚC COVER LETTER (TIẾNG VIỆT):

Tiêu đề: [ID hoặc Ref nếu có] Cover Letter - Ứng tuyển vị trí [Tên vị trí] - [Họ tên]

Phần 1 - Lời mở đầu (2-3 câu):
- Bày tỏ sự quan tâm sâu sắc đến vị trí
- Nêu nền tảng vững chắc về [kỹ năng chính]
- Khẳng định sự phù hợp cho vị trí

Phần 2 - Thể hiện kỹ năng phù hợp (3 đoạn nhỏ):
A. Về [Kỹ năng 1 từ JD]:
   - Nêu kinh nghiệm thực tế liên quan
   - Trích dẫn thành tích cụ thể từ CV (có số liệu)
   
B. Về [Kỹ năng 2 từ JD]:
   - Khác biệt so với ứng viên thông thường
   - Năng lực đặc biệt (ví dụ: lập trình, bảo mật)
   
C. Về [Kỹ năng 3/Chứng chỉ]:
   - Chứng chỉ liên quan (nếu có trong CV)
   - Tư duy chuyên môn (security, optimization)

Phần 3 - Lợi thế địa lý/cam kết (1-2 câu):
- Nêu vị trí địa lý thuận lợi (nếu có)
- Cam kết về thời gian làm việc

Phần 4 - Kết thúc (2 câu):
- Đính kèm CV chi tiết
- Mong có cơ hội trao đổi trực tiếp

Chữ ký:
Trân trọng cảm ơn,
[Họ tên]
[Số điện thoại]
[Link LinkedIn/Portfolio - nếu có]"#;

    format!(
        r#"Tạo một Cover Letter chuyên nghiệp bằng TIẾNG VIỆT dựa trên thông tin sau:

THÔNG TIN ỨNG VIÊN:
Họ tên: {}
Email: {}
Số điện thoại: {}
Website cá nhân: {}
Địa chỉ: {}

THÔNG TIN CÔNG TY/NGƯỜI TUYỂN DỤNG:
Tên công ty: {}
Website công ty: {}
Địa chỉ công ty: {}
Người quản lý tuyển dụng: {}
SĐT người tuyển dụng: {}
Email người tuyển dụng: {}
Vị trí ứng tuyển: {}

MÔ TẢ CÔNG VIỆC (JD):
{}

CV CỦA ỨNG VIÊN:
{}

{}

YÊU CẦU KHI VIẾT COVER LETTER:
1. Lời chào: {}
2. Phân tích JD để tìm ra 3 yêu cầu kỹ năng chính
3. Đối chiếu CV để tìm kinh nghiệm/kỹ năng phù hợp với từng yêu cầu
4. Extract chứng chỉ từ CV (nếu có) để làm nổi bật
5. Sử dụng số liệu và thành tích cụ thể từ CV
6. Độ dài: 300-400 từ
7. Giọng điệu: Chuyên nghiệp, tự tin nhưng khiêm tốn
8. Format: Văn bản thuần túy, chia đoạn rõ ràng (không dùng HTML tags)
9. Kết thúc bằng chữ ký: "Trân trọng cảm ơn,\n{}\n{}\n{}"

LƯU Ý QUAN TRỌNG:
- Tránh các cụm từ chung chung, sáo rỗng
- Phải cụ thể hóa bằng ví dụ từ CV
- Thể hiện sự hiểu biết về vị trí và ngành
- Nêu rõ giá trị mang lại cho công ty
- Chứng minh sự khác biệt so với ứng viên khác

Hãy tạo Cover Letter hoàn chỉnh theo cấu trúc trên:
"#,
        candidate_name,
        candidate_email,
        candidate_phone.clone().unwrap_or_else(|| "Chưa cung cấp".to_string()),
        candidate_personal_site.clone().unwrap_or_else(|| "Chưa có".to_string()),
        candidate_address.clone().unwrap_or_else(|| "Chưa cung cấp".to_string()),
        company_name,
        company_website.clone().unwrap_or_else(|| "Chưa có".to_string()),
        company_address.clone().unwrap_or_else(|| "Chưa có".to_string()),
        hiring_manager_name.clone().unwrap_or_else(|| "Ban Tuyển dụng".to_string()),
        hiring_manager_phone.clone().unwrap_or_else(|| "Chưa có".to_string()),
        hiring_manager_email.clone().unwrap_or_else(|| "Chưa có".to_string()),
        position_title,
        jd_text,
        cv_text,
        template_structure,
        hiring_greeting,
        candidate_name,
        candidate_phone.clone().unwrap_or_else(|| "[SĐT]".to_string()),
        candidate_personal_site.clone().unwrap_or_else(|| "[Link Portfolio]".to_string())
    )
}

/// Build English cover letter prompt (original)
fn build_english_cover_letter_prompt(
    cv_text: &str,
    jd_text: &str,
    candidate_name: &str,
    candidate_email: &str,
    candidate_phone: &Option<String>,
    candidate_personal_site: &Option<String>,
    candidate_address: &Option<String>,
    company_name: &str,
    company_website: &Option<String>,
    company_address: &Option<String>,
    hiring_manager_name: &Option<String>,
    hiring_manager_phone: &Option<String>,
    hiring_manager_email: &Option<String>,
    position_title: &str,
) -> String {
    let hiring_greeting = match hiring_manager_name {
        Some(name) => format!("Dear {},", name),
        None => format!("Dear Hiring Manager,"),
    };

    format!(
        r#"Generate a professional cover letter in English based on the following information:

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
Hiring manager: {}
Hiring manager phone: {}
Hiring manager email: {}
Position: {}

JOB DESCRIPTION:
{}

CANDIDATE'S CV:
{}

Write a compelling and professional cover letter with the following characteristics:
1. Start with: {}
2. Opening paragraph: Express genuine interest in the position at {} and mention 2-3 key qualifications from the CV that match the JD
3. Middle paragraphs (2-3): 
   - Highlight specific achievements and skills from CV that directly address requirements in JD
   - Show understanding of company culture and role responsibilities
   - Explain how candidate's experience solves their problems
4. Closing paragraph: Strong call to action and professional sign-off
5. Signature: {} ({})

IMPORTANT REQUIREMENTS:
- Length: 250-350 words
- Tone: Professional but warm and genuine
- Avoid generic phrases, be specific with examples
- Use numbers and metrics where possible from the CV
- Address pain points mentioned in the JD
- Format with clear paragraphs (no HTML tags, plain text)
- Include a [Your Address] [Date] section at the top

Please generate the complete cover letter ready to send:
"#,
        candidate_name,
        candidate_email,
        candidate_phone.clone().unwrap_or_else(|| "N/A".to_string()),
        candidate_personal_site.clone().unwrap_or_else(|| "N/A".to_string()),
        candidate_address.clone().unwrap_or_else(|| "N/A".to_string()),
        company_name,
        company_website.clone().unwrap_or_else(|| "N/A".to_string()),
        company_address.clone().unwrap_or_else(|| "N/A".to_string()),
        hiring_manager_name.clone().unwrap_or_else(|| "Hiring Manager".to_string()),
        hiring_manager_phone.clone().unwrap_or_else(|| "N/A".to_string()),
        hiring_manager_email.clone().unwrap_or_else(|| "N/A".to_string()),
        position_title,
        jd_text,
        cv_text,
        hiring_greeting,
        company_name,
        candidate_name,
        candidate_email
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

fn generate_cover_letter_fallback(request: &CoverLetterRequest) -> String {
    let language = request.language.as_ref().unwrap_or(&CoverLetterLanguage::English);
    
    match language {
        CoverLetterLanguage::Vietnamese => generate_vietnamese_cover_letter_fallback(request),
        CoverLetterLanguage::English => generate_english_cover_letter_fallback(request),
    }
}

fn generate_vietnamese_cover_letter_fallback(request: &CoverLetterRequest) -> String {
    let greeting = request
        .hiring_manager_name
        .as_ref()
        .map(|name| format!("Kính gửi Anh/Chị {},", name))
        .unwrap_or_else(|| "Kính gửi Ban Tuyển dụng,".to_string());

    format!(
        "Cover Letter - Ứng tuyển vị trí {} - {}\n\n{}\n\nTôi viết thư này để bày tỏ sự quan tâm sâu sắc đến vị trí {} tại {}. Với nền tảng vững chắc về các kỹ năng chuyên môn và kinh nghiệm thực tế được nêu trong CV, tôi tin rằng mình là ứng viên phù hợp để đóng góp vào sự phát triển của công ty.\n\nQua tìm hiểu về mô tả công việc, tôi nhận thấy kỹ năng và kinh nghiệm của mình đáp ứng tốt các yêu cầu then chốt. Tôi có kinh nghiệm thực tế trong việc phát triển hệ thống, xây dựng giải pháp kỹ thuật và làm việc nhóm hiệu quả để giải quyết các vấn đề quan trọng.\n\nNhững thế mạnh chính của tôi bao gồm:\n• Kỹ năng kỹ thuật vững chắc và khả năng học hỏi nhanh\n• Kinh nghiệm làm việc với các công nghệ hiện đại\n• Tư duy giải quyết vấn đề và tinh thần trách nhiệm cao\n\nTôi đã đính kèm CV chi tiết các dự án và kỹ năng kỹ thuật để Quý công ty tiện xem xét. Rất mong có cơ hội được trao đổi trực tiếp về cách tôi có thể hỗ trợ đội ngũ của công ty.\n\nTrân trọng cảm ơn Anh/Chị đã dành thời gian xem xét hồ sơ.\n\n{}\n{}\n{}",
        request.position_title,
        request.candidate_name,
        greeting,
        request.position_title,
        request.company_name,
        request.candidate_name,
        request.candidate_phone.clone().unwrap_or_else(|| "[Số điện thoại]".to_string()),
        request.candidate_email
    )
}

fn generate_english_cover_letter_fallback(request: &CoverLetterRequest) -> String {
    let greeting = request
        .hiring_manager_name
        .as_ref()
        .map(|name| format!("Dear {},", name))
        .unwrap_or_else(|| "Dear Hiring Manager,".to_string());

    format!(
        "[Your Address]\n[Date]\n\n{}\n\nI am writing to express my strong interest in the {} position at {}. With hands-on experience highlighted in my CV and a strong alignment with your job description, I am confident I can contribute effectively from day one.\n\nMy background includes building backend systems, delivering reliable APIs, and collaborating across teams to solve business-critical problems. I am especially excited about this opportunity because it matches my strengths in execution, ownership, and continuous learning.\n\nI believe my technical foundation and communication skills would allow me to quickly add value to your engineering team. I would welcome the opportunity to discuss how my experience can support your goals for this role.\n\nThank you for your time and consideration. I look forward to hearing from you.\n\nSincerely,\n{}\n{}",
        greeting,
        request.position_title,
        request.company_name,
        request.candidate_name,
        request.candidate_email
    )
}

/// Extract key points from the cover letter
fn extract_key_points(content: &str) -> Vec<String> {
    let mut points = Vec::new();
    
    // Look for sentences mentioning achievements, skills, or experiences
    let sentences: Vec<&str> = content.split('.').collect();
    
    for sentence in sentences.iter().take(5) {
        let trimmed = sentence.trim();
        if trimmed.len() > 20 
            && (trimmed.contains("achieved") 
                || trimmed.contains("improved") 
                || trimmed.contains("developed")
                || trimmed.contains("led")
                || trimmed.contains("managed")) {
            points.push(format!("{}.", trimmed));
        }
    }
    
    // If we didn't find enough points, just take first 3 sentences
    if points.len() < 3 {
        points.clear();
        for sentence in sentences.iter().take(3) {
            let trimmed = sentence.trim();
            if trimmed.len() > 20 {
                points.push(format!("{}.", trimmed));
            }
        }
    }
    
    points
}

/// Estimate match score based on cover letter content
fn estimate_match_score_from_letter(letter: &str, jd: &str) -> u8 {
    let letter_lower = letter.to_lowercase();
    let jd_lower = jd.to_lowercase();
    
    // Extract keywords from JD
    let jd_words: Vec<&str> = jd_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .collect();
    
    // Count how many JD keywords appear in letter
    let mut matches = 0;
    for word in &jd_words {
        if letter_lower.contains(word) {
            matches += 1;
        }
    }
    
    // Calculate percentage
    let match_percentage = if jd_words.is_empty() {
        50
    } else {
        ((matches as f32 / jd_words.len() as f32) * 100.0).min(100.0) as u8
    };
    
    // Add bonus if letter is detailed (long)
    let bonus = if letter.len() > 300 { 5 } else { 0 };
    
    (match_percentage + bonus).min(100)
}

/// Convert plain text cover letter to HTML
fn convert_to_html(text: &str) -> String {
    let mut html = String::from(
        r#"<div style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
"#
    );
    
    // Split into paragraphs
    for paragraph in text.split("\n\n") {
        if !paragraph.trim().is_empty() {
            // Check if it looks like a heading (ends with colon or is very short)
            if paragraph.trim().ends_with(':') || (paragraph.len() < 100 && !paragraph.contains(' ')) {
                html.push_str(&format!(
                    "<p style=\"margin-top: 20px; font-weight: bold;\">{}</p>\n",
                    paragraph.trim()
                ));
            } else {
                html.push_str(&format!(
                    "<p style=\"margin: 10px 0;\">{}</p>\n",
                    paragraph.trim()
                ));
            }
        }
    }
    
    html.push_str("</div>");
    html
}
