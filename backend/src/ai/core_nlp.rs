/// CORE NLP MODULE - Resume Scoring with Gemini AI + Regex Fallback
/// ================================================================
/// 
/// Architecture:
/// 1. PRIMARY: Call Gemini API cho scoring chính xác
/// 2. FALLBACK: Use Regex patterns nếu API fails
/// 3. VALIDATION: Edge case handling cho CV/JD
/// 4. SCORING: 4-criteria breakdown (0-10 each)

use regex::Regex;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use crate::ai::nlp_processing;

// ============================================
// DATA STRUCTURES
// ============================================

/// Configuration cho NLP Module
#[derive(Debug, Clone)]
pub struct NLPConfig {
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub timeout_seconds: u64,
    pub min_cv_length: usize,
    pub min_jd_length: usize,
}

impl Default for NLPConfig {
    fn default() -> Self {
        let gemini_api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .unwrap_or_else(|_| String::new());

        let gemini_model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        Self {
            gemini_api_key,
            gemini_model,
            timeout_seconds: 30,
            min_cv_length: 50,
            min_jd_length: 10, // Reduced to allow shorter JD text
        }
    }
}

/// Kết quả điểm số cho từng tiêu chí
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub technical_score: u8,           // 0-10
    pub experience_score: u8,          // 0-10
    pub culture_score: u8,             // 0-10
    pub growth_score: u8,              // 0-10
    pub overall_score: u8,             // 0-100
    pub confidence: f32,               // 0.0-1.0
    pub source: ScoringSource,         // Gemini hoặc Regex
}

impl Default for ScoreResult {
    fn default() -> Self {
        Self {
            technical_score: 0,
            experience_score: 0,
            culture_score: 0,
            growth_score: 0,
            overall_score: 0,
            confidence: 0.0,
            source: ScoringSource::Regex,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringSource {
    Gemini,
    Regex,
}

/// Dữ liệu hỗ trợ cho scoring
#[derive(Debug, Clone)]
pub struct ScoringContext {
    pub matching_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub ghost_skills: Vec<String>,
    pub red_flags: Vec<String>,
    pub has_learning: bool,
    pub skill_diversity: usize,
    pub semantic_alignment: f32,
}

/// Chi tiết giải thích từng điểm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreExplanation {
    pub technical: String,          // "7/10 - Giải thích"
    pub experience: String,         // "6/10 - Giải thích"
    pub culture: String,            // "8/10 - Giải thích"
    pub growth: String,             // "7/10 - Giải thích"
    pub summary: String,            // Tóm tắt chung
}

// ============================================
// GEMINI API STRUCTURES
// ============================================

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeminiScore {
    technical_score: u8,
    experience_score: u8,
    culture_score: u8,
    growth_score: u8,
    confidence: f32,
    explanation: ScoreExplanation,
}

// ============================================
// MAIN SCORING INTERFACE
// ============================================

/// Chấm điểm CV với Gemini API (ưu tiên) + Regex fallback
pub async fn score_with_gemini_fallback(
    cv_text: &str,
    jd_text: &str,
    config: &NLPConfig,
) -> Result<(ScoreResult, Option<ScoringContext>)> {
    // Bước 1: Validate inputs
    validate_inputs(cv_text, jd_text, config)?;

    // Bước 2: Try Gemini API
    match try_gemini_scoring(cv_text, jd_text, config).await {
        Ok((score, context)) => {
            println!("✅ Gemini API scoring successful");
            Ok((score, Some(context)))
        }
        Err(e) => {
            eprintln!("⚠️ Gemini API failed: {}, falling back to Regex", e);
            
            // Bước 3: Fallback to Regex
            let context = extract_context_regex(cv_text, jd_text)?;
            let score = score_with_regex(&context)?;
            
            println!("✅ Regex fallback scoring successful");
            Ok((score, Some(context)))
        }
    }
}

/// Chấm điểm chỉ sử dụng Regex (không gọi API)
#[allow(dead_code)]
pub fn score_with_regex_only(
    cv_text: &str,
    jd_text: &str,
) -> Result<(ScoreResult, ScoringContext)> {
    validate_inputs(cv_text, jd_text, &NLPConfig::default())?;
    
    let context = extract_context_regex(cv_text, jd_text)?;
    let score = score_with_regex(&context)?;
    
    Ok((score, context))
}

// ============================================
// VALIDATION & INPUT HANDLING
// ============================================

/// Validate CV và JD inputs
fn validate_inputs(cv_text: &str, jd_text: &str, config: &NLPConfig) -> Result<()> {
    // Edge case 1: Empty inputs
    if cv_text.trim().is_empty() {
        return Err(anyhow!("CV text is empty"));
    }
    if jd_text.trim().is_empty() {
        return Err(anyhow!("JD text is empty"));
    }

    // Edge case 2: Too short inputs
    if cv_text.trim().len() < config.min_cv_length {
        return Err(anyhow!(
            "CV too short: {} chars (min: {})",
            cv_text.len(),
            config.min_cv_length
        ));
    }
    if jd_text.trim().len() < config.min_jd_length {
        return Err(anyhow!(
            "JD too short: {} chars (min: {})",
            jd_text.len(),
            config.min_jd_length
        ));
    }

    // Edge case 3: Check for special characters / encoding issues
    if !cv_text.is_ascii() {
        eprintln!("⚠️ CV contains non-ASCII characters (will be handled)");
    }
    if !jd_text.is_ascii() {
        eprintln!("⚠️ JD contains non-ASCII characters (will be handled)");
    }

    // Edge case 4: Very large files
    if cv_text.len() > 1_000_000 {
        eprintln!("⚠️ CV is very large: {} MB", cv_text.len() / 1_000_000);
    }

    Ok(())
}

/// Sanitize text: remove special chars, normalize (preserve emoji and unicode)
fn sanitize_text(text: &str) -> String {
    text
        .to_lowercase()
        .chars()
        .filter(|c| {
            c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '/' || 
            !c.is_ascii() // Keep unicode/emoji characters
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ============================================
// GEMINI API INTEGRATION
// ============================================

/// Call Gemini API để chấm điểm
async fn try_gemini_scoring(
    cv_text: &str,
    jd_text: &str,
    config: &NLPConfig,
) -> Result<(ScoreResult, ScoringContext)> {
    // Edge case: Missing API key
    if config.gemini_api_key.is_empty() {
        return Err(anyhow!("Gemini API key is empty"));
    }

    let client = reqwest::Client::new();

    let prompt = format!(
        r#"Bạn là expert HR analyst. Chấm điểm CV so với JD theo 4 tiêu chí (0-10 mỗi tiêu chí):

1. TECHNICAL_MATCH (0-10): % kỹ năng kỹ thuật khớp
   - 9-10: Đáp ứng >= 90% kỹ năng yêu cầu
   - 7-8: Đáp ứng 70-89%
   - 5-6: Đáp ứng 50-69%
   - 3-4: Đáp ứng 30-49%
   - 0-2: Đáp ứng < 30%

2. EXPERIENCE_LEVEL (0-10): Leadership, Project Management, Problem Solving, etc.
   - 9-10: 4+ ghost skills (dày dạn, senior)
   - 7-8: 3-4 ghost skills (mid-senior)
   - 5-6: 2-3 ghost skills (mid-level)
   - 3-4: 1-2 ghost skills (junior)
   - 0-2: 0 ghost skills (fresher)

3. CULTURE_FIT (0-10): Job hopping, gaps, inconsistency
   - 8-10: Không có red flags
   - 6-7: 1 red flag
   - 4-5: 2 red flags
   - 2-3: 3 red flags
   - 0-1: >= 4 red flags

4. GROWTH_POTENTIAL (0-10): Learning ability, skill diversity
   - 9-10: Learning indicators + 5+ skills + ghost skills
   - 7-8: Có 2/3 yếu tố
   - 5-6: Có 1/3 yếu tố
   - 3-4: Chưa rõ
   - 0-2: Không có dấu hiệu

Trả lại JSON:
{{
  "technical_score": <0-10>,
  "experience_score": <0-10>,
  "culture_score": <0-10>,
  "growth_score": <0-10>,
  "confidence": <0.0-1.0>,
  "explanation": {{
    "technical": "<score>/10 - Giải thích chi tiết",
    "experience": "<score>/10 - Giải thích chi tiết",
    "culture": "<score>/10 - Giải thích chi tiết",
    "growth": "<score>/10 - Giải thích chi tiết",
    "summary": "Tóm tắt chung (1-2 câu)"
  }}
}}

CV (first 20000 chars):
{}

JD (first 15000 chars):
{}

CHỈ TRẢ JSON, KHÔNG MARKDOWN."#,
        truncate_to_chars(&cv_text, 20000),
        truncate_to_chars(&jd_text, 15000)
    );

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt }],
        }],
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.gemini_model, config.gemini_api_key
    );

    // Edge case: Network timeout
    let response = match tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_seconds),
        client.post(&url).json(&request).send(),
    )
    .await
    {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => return Err(anyhow!("Network error: {}", e)),
        Err(_) => return Err(anyhow!("Request timeout after {} seconds", config.timeout_seconds)),
    };

    // Edge case: HTTP error
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Gemini API error ({}): {}",
            status,
            error_text.chars().take(200).collect::<String>()
        ));
    }

    // Edge case: Invalid JSON response
    let gemini_response: GeminiResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse Gemini response: {}", e))?;

    let json_text = gemini_response
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.as_str())
        .ok_or_else(|| anyhow!("No response from Gemini API"))?;

    // Clean JSON from markdown blocks
    let clean_json = clean_json_response(json_text);

    // Edge case: JSON parse error - try to extract scores
    let score_data: GeminiScore = serde_json::from_str(clean_json)
        .map_err(|e| anyhow!("JSON parse error: {}", e))?;

    // Validate scores are in 0-10 range
    if score_data.technical_score > 10
        || score_data.experience_score > 10
        || score_data.culture_score > 10
        || score_data.growth_score > 10
    {
        return Err(anyhow!(
            "Scores out of range: {}+{}+{}+{}",
            score_data.technical_score,
            score_data.experience_score,
            score_data.culture_score,
            score_data.growth_score
        ));
    }

    let overall_score = (
        (score_data.technical_score as f32
            + score_data.experience_score as f32
            + score_data.culture_score as f32
            + score_data.growth_score as f32)
            / 4.0
            * 10.0
    ).round() as u8;

    let score = ScoreResult {
        technical_score: score_data.technical_score,
        experience_score: score_data.experience_score,
        culture_score: score_data.culture_score,
        growth_score: score_data.growth_score,
        overall_score,
        confidence: score_data.confidence.clamp(0.0, 1.0),
        source: ScoringSource::Gemini,
    };

    // Extract context from response untuk reference
    let context = extract_context_regex(cv_text, jd_text)?;

    Ok((score, context))
}

fn clean_json_response(text: &str) -> &str {
    if text.contains("```json") {
        text.split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(text)
            .trim()
    } else if text.contains("```") {
        text.split("```")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(text)
            .trim()
    } else {
        text.trim()
    }
}

// ============================================
// REGEX FALLBACK SCORING
// ============================================

/// Extract context từ CV/JD sử dụng Regex
fn extract_context_regex(cv_text: &str, jd_text: &str) -> Result<ScoringContext> {
    let cv_lower = sanitize_text(cv_text);
    let jd_lower = sanitize_text(jd_text);

    // Technical skills database (40+ skills)
    let technical_skills = vec![
        "python", "javascript", "typescript", "react", "vue", "angular",
        "nodejs", "express", "django", "flask", "spring", "java",
        "cpp", "csharp", "rust", "go", "php", "ruby",
        "sql", "postgresql", "mysql", "mongodb", "redis",
        "docker", "kubernetes", "aws", "azure", "gcp",
        "git", "cicd", "jenkins", "github", "gitlab",
        "html", "css", "sass", "tailwind", "bootstrap",
        "restapi", "graphql", "microservices", "agile", "scrum",
    ];

    // RUSH-BERT + Regex hybrid keyword matching
    let keyword_result = nlp_processing::rush_bert_regex_match(&cv_lower, &jd_lower, &technical_skills);

    // Ghost skills (soft skills)
    let ghost = detect_ghost_skills(&cv_lower);

    // Red flags
    let red_flags = detect_red_flags(&cv_lower);

    // Learning indicators
    let learning_keywords = vec![
        "self-taught", "learned", "studied", "certified", "course",
        "training", "bootcamp", "udemy", "coursera",
    ];
    let has_learning = learning_keywords.iter().any(|kw| cv_lower.contains(kw));

    Ok(ScoringContext {
        matching_skills: keyword_result.matching_skills,
        missing_skills: keyword_result.missing_skills,
        ghost_skills: ghost,
        red_flags,
        has_learning,
        skill_diversity: keyword_result.skill_diversity,
        semantic_alignment: keyword_result.semantic_alignment,
    })
}


fn detect_ghost_skills(cv: &str) -> Vec<String> {
    let ghost_keywords = vec![
        ("Team Leadership", vec!["led team", "team lead", "managed team", "mentored"]),
        ("Project Management", vec!["managed project", "project lead", "delivered"]),
        ("Problem Solving", vec!["solved", "optimized", "improved"]),
        ("Communication", vec!["presented", "documented", "collaborated"]),
        ("Agile/Scrum", vec!["sprint", "standup", "scrum"]),
    ];

    let mut detected = Vec::new();
    for (skill_name, keywords) in ghost_keywords {
        if keywords.iter().any(|kw| cv.contains(kw)) {
            detected.push(skill_name.to_string());
        }
    }

    detected
}

fn detect_red_flags(cv: &str) -> Vec<String> {
    let mut flags = Vec::new();

    // Job hopping
    if let Ok(re) = Regex::new(r"(\d{1,2})\s*(month|tháng)") {
        let short_jobs = re.find_iter(cv).count();
        if short_jobs >= 3 {
            flags.push(format!("Job hopping: {} short positions", short_jobs));
        }
    }

    // Employment gap
    if cv.contains("gap") || cv.contains("break") || cv.contains("unemployed") {
        flags.push("Employment gap detected".to_string());
    }

    // Inconsistency
    let freelance_count = cv.matches("freelance").count();
    if freelance_count >= 3 && cv.contains("fulltime") {
        flags.push("Freelance/FT inconsistency".to_string());
    }

    flags
}

/// Score sử dụng Regex
pub fn score_with_regex(context: &ScoringContext) -> Result<ScoreResult> {
    // 1. Technical Score (0-10)
    let total_required = context.matching_skills.len() + context.missing_skills.len();
    let mut tech_score = if total_required > 0 {
        ((context.matching_skills.len() as f32 / total_required as f32) * 10.0).round() as u8
    } else {
        5
    };

    if context.semantic_alignment > 0.65 {
        tech_score = std::cmp::min(10, tech_score + 1);
    }

    // 2. Experience Score (0-10)
    let exp_score = std::cmp::min(10, (context.ghost_skills.len() * 2) as u8);

    // 3. Culture Score (0-10)
    let culture_score = std::cmp::max(0, 8 - (context.red_flags.len() * 2) as i8) as u8;

    // 4. Growth Score (0-10)
    let mut growth_score = 5u8;
    if context.has_learning { growth_score += 2; }
    if context.matching_skills.len() >= 5 { growth_score += 2; }
    if !context.ghost_skills.is_empty() { growth_score += 1; }
    if context.skill_diversity >= 6 { growth_score += 1; }
    let growth_score = std::cmp::min(10, growth_score);

    // Overall Score (0-100)
    let overall_score = (
        (tech_score as f32 + exp_score as f32 + culture_score as f32 + growth_score as f32)
            / 4.0
            * 10.0
    ).round() as u8;

    let confidence = if overall_score >= 70 {
        0.9
    } else if overall_score >= 50 {
        0.75
    } else {
        0.6
    };

    Ok(ScoreResult {
        technical_score: tech_score,
        experience_score: exp_score,
        culture_score,
        growth_score,
        overall_score,
        confidence,
        source: ScoringSource::Regex,
    })
}

// ============================================
// EXPLANATION GENERATION
// ============================================

/// Generate detailed explanations cho mỗi score
pub fn generate_explanations(score: &ScoreResult, context: &ScoringContext) -> ScoreExplanation {
    let technical = format!(
        "{}/10 - {}",
        score.technical_score,
        if score.technical_score >= 7 {
            format!("Excellent: {} matched skills", context.matching_skills.len())
        } else if score.technical_score >= 5 {
            format!("Good: {} matched, {} missing", context.matching_skills.len(), context.missing_skills.len())
        } else {
            format!("Poor: {} matched, {} missing", context.matching_skills.len(), context.missing_skills.len())
        }
    );

    let experience = format!(
        "{}/10 - {}",
        score.experience_score,
        if score.experience_score >= 7 {
            format!("Experienced: {} ghost skills", context.ghost_skills.len())
        } else if score.experience_score >= 4 {
            format!("Some experience: {} ghost skills", context.ghost_skills.len())
        } else {
            "Limited experience".to_string()
        }
    );

    let culture = format!(
        "{}/10 - {}",
        score.culture_score,
        if context.red_flags.is_empty() {
            "Stable career, no red flags".to_string()
        } else {
            format!("{} red flags", context.red_flags.len())
        }
    );

    let growth = format!(
        "{}/10 - {}",
        score.growth_score,
        if score.growth_score >= 7 {
            "High growth potential".to_string()
        } else if score.growth_score >= 5 {
            "Moderate growth potential".to_string()
        } else {
            "Limited growth indicators".to_string()
        }
    );

    let summary = format!(
        "Candidate scored {}%. {} skills matched, {} red flags, semantic fit {:.0}%. Confidence: {:.0}%",
        score.overall_score,
        context.matching_skills.len(),
        context.red_flags.len(),
        context.semantic_alignment * 100.0,
        score.confidence * 100.0
    );

    ScoreExplanation {
        technical,
        experience,
        culture,
        growth,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_text() {
        let input = "Python, React, Node.js!!!";
        let output = sanitize_text(input);
        assert_eq!(output, "python react nodejs");
    }

    #[test]
    fn test_validate_inputs() {
        let config = NLPConfig::default();
        
        // Test empty inputs
        assert!(validate_inputs("", "test", &config).is_err());
        assert!(validate_inputs("test", "", &config).is_err());
        
        // Test valid inputs
        assert!(validate_inputs("python react node.js", "looking for python developer", &config).is_ok());
    }

    #[test]
    fn test_regex_scoring() {
        let context = ScoringContext {
            matching_skills: vec!["PYTHON".to_string(), "REACT".to_string()],
            missing_skills: vec!["RUST".to_string()],
            ghost_skills: vec!["Team Leadership".to_string(), "Project Management".to_string()],
            red_flags: vec![],
            has_learning: true,
            skill_diversity: 3,
            semantic_alignment: 0.7,
        };

        let result = score_with_regex(&context).unwrap();
        assert_eq!(result.technical_score, 6); // 2/3 = 6.67 => 7
        assert_eq!(result.experience_score, 4); // 2 * 2 = 4
        assert_eq!(result.culture_score, 8); // 8 - 0 = 8
        assert!(result.growth_score >= 8); // 5 + 2 (learning) + 1 (ghost) = 8+
    }

    #[test]
    fn test_red_flag_detection() {
        let cv = "worked 3 months, then 2 months, then 4 months";
        let flags = detect_red_flags(cv);
        assert!(!flags.is_empty(), "Should detect job hopping");
    }
}

// ============================================
// HELPER FUNCTIONS
// ============================================

/// Safely truncate string to N characters (handles UTF-8 properly)
fn truncate_to_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
