use crate::models::{AnalysisVerdict, ExplanationVietnamese, ScoreBreakdown};
use crate::ai::core_nlp::{self, NLPConfig, ScoringSource};
use crate::ai::skill_matcher;
use anyhow::Result;
use serde::{Deserialize, Serialize};

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

/// Main analyze function with Gemini + Regex fallback
/// 
/// Priority:
/// 1. Try Gemini API for accurate scoring
/// 2. Fallback to Regex if Gemini fails
/// 3. Return detailed Vietnamese explanations
pub async fn analyze_resume(cv_text: &str, jd_text: &str) -> Result<AnalysisVerdict> {
    // Analyze skill matching (new feature)
    let skill_match = skill_matcher::analyze_skill_match(cv_text, jd_text);
    
    // Create NLP config
    let config = NLPConfig::default();
    
    // Create empty default context
    let default_context = core_nlp::ScoringContext {
        matching_skills: vec![],
        missing_skills: vec![],
        ghost_skills: vec![],
        red_flags: vec![],
        has_learning: false,
        skill_diversity: 0,
        semantic_alignment: 0.0,
    };

    // Try scoring with Gemini (fallback to Regex)
    let (score_result, context) = match core_nlp::score_with_gemini_fallback(cv_text, jd_text, &config).await {
        Ok((score, Some(context))) => (score, context),
        Ok((score, None)) => (score, default_context.clone()),
        Err(e) => {
            eprintln!("⚠️ Gemini + Regex failed: {}, using default context", e);
            (core_nlp::ScoreResult::default(), default_context)
        }
    };

    // Generate explanations
    let explanations = core_nlp::generate_explanations(&score_result, &context);

    let required_skill_count = skill_match.matched_skills.len() + skill_match.missing_skills.len();
    let technical_score_10 = if required_skill_count == 0 {
        0u8
    } else {
        ((skill_match.matched_skills.len() as f32 / required_skill_count as f32) * 10.0)
            .round()
            .clamp(0.0, 10.0) as u8
    };

    let technical_breakdown = format!(
        "{}/10 - Matched {} technical skills, missing {}",
        technical_score_10,
        skill_match.matched_skills.len(),
        skill_match.missing_skills.len()
    );

    let experience_breakdown = match (
        skill_match.total_experience_years,
        skill_match.required_experience_years,
        skill_match.candidate_experience_level.as_ref(),
        skill_match.required_experience_level.as_ref(),
    ) {
        (Some(cv_years), Some(jd_years), Some(cv_level), Some(jd_level)) => {
            let exp_score = if skill_match.experience_match { 10 } else { 4 };
            format!(
                "{}/10 - CV {:.1} years ({}) vs JD {:.1} years ({})",
                exp_score, cv_years, cv_level, jd_years, jd_level
            )
        }
        (Some(cv_years), Some(jd_years), _, _) => {
            let exp_score = if skill_match.experience_match { 9 } else { 4 };
            format!(
                "{}/10 - CV {:.1} years vs JD {:.1} years",
                exp_score, cv_years, jd_years
            )
        }
        (Some(cv_years), None, Some(cv_level), _) => {
            format!("7/10 - CV thể hiện {:.1} years, level {}", cv_years, cv_level)
        }
        (Some(cv_years), None, None, _) => {
            format!("6/10 - CV thể hiện {:.1} years", cv_years)
        }
        (None, _, Some(cv_level), Some(jd_level)) => {
            format!("4/10 - CV level {} so với yêu cầu {}", cv_level, jd_level)
        }
        _ => "0/10 - Chưa trích xuất rõ thông tin kinh nghiệm từ CV/JD".to_string(),
    };

    // Determine verdict
    let verdict = if score_result.overall_score >= 75 {
        "Hire"
    } else if score_result.overall_score >= 50 {
        "Maybe"
    } else {
        "Pass"
    };

    // Build Vietnamese explanation
    let summary = match score_result.overall_score {
        90..=100 => format!(
            "⭐⭐ Ứng viên XẤU SẮC ({}%) - Phù hợp RẤT CAO với vị trí. ",
            score_result.overall_score
        ),
        75..=89 => format!(
            "⭐ Ứng viên TỐT ({}%) - Phù hợp CAO với vị trí. ",
            score_result.overall_score
        ),
        50..=74 => format!(
            "⚡ Ứng viên CÓ TIỀM NĂNG ({}%) - Phù hợp TRUNG BÌNH. ",
            score_result.overall_score
        ),
        _ => format!(
            "⚠️ Ứng viên CHƯA PHÙ HỢP ({}%) - Điểm không đủ. ",
            score_result.overall_score
        ),
    };

    let mut strengths = Vec::new();

    if !skill_match.matched_skills.is_empty() {
        let top_skills = skill_match
            .matched_skills
            .iter()
            .take(6)
            .map(|item| match item.years_experience {
                Some(years) => format!("{} ({:.1} năm)", item.skill_name, years),
                None => item.skill_name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        strengths.push(format!(
            "✅ Kỹ Năng Kỹ Thuật Rõ Ràng: Khớp {} kỹ năng quan trọng ({})",
            skill_match.matched_skills.len(),
            top_skills
        ));
    }

    if !skill_match.matched_certificates.is_empty() {
        strengths.push(format!(
            "🏅 Chứng Chỉ Phù Hợp: {}",
            skill_match.matched_certificates.join(", ")
        ));
    }

    if let Some(years) = skill_match.total_experience_years {
        if let Some(level) = &skill_match.candidate_experience_level {
            strengths.push(format!(
                "📈 Mức Độ Kinh Nghiệm: {} ({:.1} năm)",
                level,
                years
            ));
        } else {
            strengths.push(format!(
                "📈 Tổng Kinh Nghiệm: {:.1} năm",
                years
            ));
        }
    }

    if !context.ghost_skills.is_empty() {
        strengths.push(format!(
            "💡 Kỹ Năng Mềm: Có {} kỹ năng ngầm ({})",
            context.ghost_skills.len(),
            context.ghost_skills.join(", ")
        ));
    }

    if context.has_learning {
        strengths.push(
            "📚 Khả Năng Tự Học: CV thể hiện sự chủ động học hỏi và phát triển".to_string()
        );
    }

    if score_result.technical_score >= 7 {
        strengths.push(
            "🎯 Nền Tảng Kỹ Thuật: Đáp ứng tốt yêu cầu công việc".to_string()
        );
    }

    if strengths.is_empty() {
        strengths.push("Chưa có điểm mạnh nổi bật rõ ràng".to_string());
    }

    let mut weaknesses = Vec::new();

    if !skill_match.missing_skills.is_empty() {
        let top_missing = skill_match
            .missing_skills
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        weaknesses.push(format!(
            "❌ Thiếu Kỹ Năng Cốt Lõi: {} kỹ năng chưa đáp ứng ({})",
            skill_match.missing_skills.len(),
            top_missing
        ));
    }

    if !skill_match.missing_certificates.is_empty() {
        weaknesses.push(format!(
            "📜 Thiếu Chứng Chỉ Yêu Cầu: {}",
            skill_match.missing_certificates.join(", ")
        ));
    }

    if !skill_match.experience_match {
        let required_years = skill_match
            .required_experience_years
            .map(|value| format!("{:.1} năm", value))
            .unwrap_or_else(|| "không xác định".to_string());
        let candidate_years = skill_match
            .total_experience_years
            .map(|value| format!("{:.1} năm", value))
            .unwrap_or_else(|| "chưa thể hiện".to_string());
        let required_level = skill_match
            .required_experience_level
            .clone()
            .unwrap_or_else(|| "không yêu cầu level cụ thể".to_string());
        let candidate_level = skill_match
            .candidate_experience_level
            .clone()
            .unwrap_or_else(|| "chưa xác định".to_string());

        weaknesses.push(format!(
            "⏳ Kinh Nghiệm Chưa Đạt: JD cần {} ({}) nhưng CV hiện có {} ({})",
            required_years,
            required_level,
            candidate_years,
            candidate_level
        ));
    }

    if !context.red_flags.is_empty() {
        for flag in &context.red_flags {
            weaknesses.push(format!("⚠️ {}", flag));
        }
    }

    if weaknesses.is_empty() {
        weaknesses.push("Không có điểm yếu đáng kể".to_string());
    }

    let recommendation = match verdict {
        "Hire" => format!(
            "💚 ĐỀ XUẤT PHỎNG VẤN - Ứng viên đạt {}% ({:.0}% confidence). Top candidate, mời phỏng vấn ngay.",
            score_result.overall_score,
            score_result.confidence * 100.0
        ),
        "Maybe" => format!(
            "💛 CÂN NHẮC KỸ - Ứng viên đạt {}% ({:.0}% confidence). Có tiềm năng, nên phỏng vấn để làm rõ kỹ năng còn thiếu.",
            score_result.overall_score,
            score_result.confidence * 100.0
        ),
        _ => format!(
            "❤️ KHÔNG PHÙ HỢP - Ứng viên chỉ đạt {}% ({:.0}% confidence). Chưa đáp ứng yêu cầu cơ bản.",
            score_result.overall_score,
            score_result.confidence * 100.0
        ),
    };

    let source_note = match score_result.source {
        ScoringSource::Gemini => "(Chấm điểm bằng Gemini AI)",
        ScoringSource::Regex => "(Chấm điểm fallback bằng Regex)",
    };

    let explanation = ExplanationVietnamese {
        summary: format!("{}{}", summary, source_note),
        strengths,
        weaknesses,
        score_breakdown: ScoreBreakdown {
            technical_match: technical_breakdown,
            experience_level: experience_breakdown,
            culture_fit: explanations.culture,
            growth_potential: explanations.growth,
        },
        recommendation,
    };

    let clean_matching_skills = skill_match
        .matched_skills
        .iter()
        .map(|item| item.skill_name.to_uppercase())
        .collect::<Vec<_>>();
    let clean_missing_skills = skill_match
        .missing_skills
        .iter()
        .map(|item| item.to_uppercase())
        .collect::<Vec<_>>();

    Ok(AnalysisVerdict {
        overall_score: score_result.overall_score,
        verdict: verdict.to_string(),
        matching_skills: clean_matching_skills,
        missing_skills: clean_missing_skills,
        ghost_skills: context.ghost_skills,
        red_flags: context.red_flags,
        confidence_level: score_result.confidence,
        explanation_vietnamese: Some(explanation),
        skill_match_details: Some(skill_match),
    })
}

/// LOGIC CHẤM ĐIỂM CHI TIẾT
/// ===========================
/// 
/// Hệ thống chấm điểm gồm 4 tiêu chí, mỗi tiêu chí từ 0-10 điểm:
/// 
/// 1. KỸ NĂNG KỸ THUẬT (Technical Match) - 0/10
///    - Đánh giá: % kỹ năng kỹ thuật trong CV khớp với JD
///    - Công thức: (số kỹ năng khớp / tổng kỹ năng yêu cầu) * 10
///    - Ví dụ: 5 khớp/10 yêu cầu = 5/10 điểm
/// 
/// 2. MỨC ĐỘ KINH NGHIỆM (Experience Level) - 0/10  
///    - Đánh giá: Kinh nghiệm thực tế qua ghost skills & keywords
///    - Ghost skills: Leadership, Project Management, Problem Solving...
///    - Công thức: (số ghost skills * 2) điểm, tối đa 10
///    - Ví dụ: 3 ghost skills = 6/10 điểm
/// 
/// 3. PHÙ HỢP VĂN HÓA (Culture Fit) - 0/10
///    - Đánh giá: Sự nghiệp ổn định, không có red flags
///    - Red flags: Nhảy việc nhiều, gap employment, inconsistency
///    - Điểm: 8/10 nếu không có red flag, -2 điểm mỗi red flag
///    - Ví dụ: 1 red flag = 6/10 điểm
/// 
/// 4. TIỀM NĂNG PHÁT TRIỂN (Growth Potential) - 0/10
///    - Đánh giá: Khả năng học hỏi và phát triển
///    - Dựa vào: Diversity của skills, learning keywords
///    - Công thức: Base 5 + bonus từ learning indicators
///    - Ví dụ: Có "learned", "self-taught" = +2 điểm
/// 
/// OVERALL SCORE (0-100):
/// - Công thức: Trung bình 4 tiêu chí * 10
/// - Ví dụ: (5+6+8+7)/4 * 10 = 65/100
/// 
/// VERDICT (Kết luận tuyển dụng):
/// - HIRE (>= 75%): Phù hợp cao, đề xuất phỏng vấn
/// - MAYBE (50-74%): Cân nhắc, cần đánh giá thêm  
/// - PASS (< 50%): Không phù hợp, từ chối
///
fn create_mock_analysis(cv_text: &str, jd_text: &str) -> AnalysisVerdict {
    let cv_lower = cv_text.to_lowercase();
    let jd_lower = jd_text.to_lowercase();
    
    // Analyze skill matching
    let skill_match = skill_matcher::analyze_skill_match(cv_text, jd_text);
    
    // ============================================
    // BƯỚC 1: PHÂN TÍCH KỸ NĂNG KỸ THUẬT
    // ============================================
    let technical_skills = vec![
        "python", "javascript", "typescript", "react", "vue", "angular",
        "node.js", "express", "django", "flask", "spring", "java",
        "c++", "c#", "rust", "go", "php", "ruby",
        "sql", "postgresql", "mysql", "mongodb", "redis",
        "docker", "kubernetes", "aws", "azure", "gcp",
        "git", "ci/cd", "jenkins", "github actions",
        "html", "css", "sass", "tailwind", "bootstrap",
        "rest api", "graphql", "microservices", "agile", "scrum"
    ];
    
    let mut matching_skills = Vec::new();
    let mut missing_skills = Vec::new();
    
    for skill in &technical_skills {
        let in_cv = cv_lower.contains(skill);
        let in_jd = jd_lower.contains(skill);
        
        if in_jd && in_cv {
            matching_skills.push(skill.to_string());
        } else if in_jd && !in_cv {
            missing_skills.push(skill.to_string());
        }
    }
    
    let total_required = matching_skills.len() + missing_skills.len();
    
    // ============================================
    // BƯỚC 2: PHÁT HIỆN GHOST SKILLS (Kỹ năng ngầm)
    // ============================================
    let ghost_keywords = vec![
        ("Team Leadership", vec!["led team", "team lead", "managed team", "mentored", "coached"]),
        ("Project Management", vec!["managed project", "project lead", "delivered", "stakeholder", "roadmap"]),
        ("Problem Solving", vec!["solved", "optimized", "improved", "reduced cost", "increased efficiency"]),
        ("Communication", vec!["presented", "documented", "wrote", "collaborated", "coordinated"]),
        ("Agile/Scrum", vec!["sprint", "standup", "retrospective", "scrum master", "product owner"]),
    ];
    
    let mut ghost_skills = Vec::new();
    for (skill_name, keywords) in &ghost_keywords {
        if keywords.iter().any(|kw| cv_lower.contains(kw)) {
            ghost_skills.push(skill_name.to_string());
        }
    }
    
    // ============================================
    // BƯỚC 3: PHÁT HIỆN RED FLAGS (Cảnh báo)
    // ============================================
    let mut red_flags = Vec::new();
    
    // Kiểm tra job hopping (nhảy việc nhiều)
    let short_job_pattern = regex::Regex::new(r"(\d{1,2})\s*tháng|(\d{1,2})\s*months?").ok();
    if let Some(re) = short_job_pattern {
        let short_jobs = re.find_iter(&cv_text).count();
        if short_jobs >= 3 {
            red_flags.push(format!("Có {} vị trí làm việc ngắn hạn (< 1 năm)", short_jobs));
        }
    }
    
    // Kiểm tra employment gap
    if cv_lower.contains("gap") || cv_lower.contains("break") || cv_lower.contains("unemployed") {
        red_flags.push("Có khoảng trống trong sự nghiệp".to_string());
    }
    
    // Kiểm tra inconsistency
    if cv_lower.contains("freelance") && cv_lower.contains("full-time") {
        let freelance_count = cv_lower.matches("freelance").count();
        if freelance_count >= 3 {
            red_flags.push("Nhiều công việc freelance xen kẽ".to_string());
        }
    }
    
    // ============================================
    // BƯỚC 4: ĐÁNH GIÁ TIỀM NĂNG HỌC HỎI
    // ============================================
    let learning_indicators = vec![
        "self-taught", "learned", "studied", "certified", "certification",
        "course", "training", "bootcamp", "online course", "udemy", "coursera"
    ];
    
    let has_learning = learning_indicators.iter().any(|ind| cv_lower.contains(ind));
    
    // ============================================
    // BƯỚC 5: TÍNH ĐIỂM CHO TỪNG TIÊU CHÍ (0-10)
    // ============================================
    
    // 1. Kỹ Năng Kỹ Thuật (0-10)
    let tech_score = if total_required > 0 {
        ((matching_skills.len() as f32 / total_required as f32) * 10.0).round() as u8
    } else {
        5 // Mặc định nếu không có yêu cầu cụ thể
    };
    
    let tech_explanation = if tech_score >= 7 {
        format!("{}/10 - Xuất sắc: Đáp ứng {}/{} kỹ năng kỹ thuật yêu cầu", 
                tech_score, matching_skills.len(), total_required)
    } else if tech_score >= 5 {
        format!("{}/10 - Trung bình: Có {}/{} kỹ năng, cần bổ sung {} kỹ năng còn thiếu", 
                tech_score, matching_skills.len(), total_required, missing_skills.len())
    } else {
        format!("{}/10 - Yếu: Chỉ có {}/{} kỹ năng yêu cầu, thiếu nhiều kỹ năng cốt lõi", 
                tech_score, matching_skills.len(), total_required)
    };
    
    // 2. Mức Độ Kinh Nghiệm (0-10)
    let exp_score = std::cmp::min(10, (ghost_skills.len() * 2) as u8);
    
    let exp_explanation = if exp_score >= 7 {
        format!("{}/10 - Dày dạn: Có {} kỹ năng mềm quan trọng ({})", 
                exp_score, ghost_skills.len(), ghost_skills.join(", "))
    } else if exp_score >= 4 {
        format!("{}/10 - Chưa nhiều kinh nghiệm: {} kỹ năng mềm, cần thêm kinh nghiệm thực chiến", 
                exp_score, ghost_skills.len())
    } else {
        format!("{}/10 - Thiếu kinh nghiệm: CV chưa thể hiện rõ leadership, project management", exp_score)
    };
    
    // 3. Phù Hợp Văn Hóa (0-10)
    let culture_score = std::cmp::max(0, 8 - (red_flags.len() * 2) as i8) as u8;
    
    let culture_explanation = if red_flags.is_empty() {
        format!("{}/10 - Tốt: Sự nghiệp ổn định, không có cảnh báo đáng lo ngại", culture_score)
    } else {
        format!("{}/10 - Cần làm rõ: Có {} điểm cần thảo luận trong phỏng vấn", 
                culture_score, red_flags.len())
    };
    
    // 4. Tiềm Năng Phát Triển (0-10)
    let mut growth_score = 5u8; // Base score
    
    // Bonus từ learning ability
    if has_learning {
        growth_score += 2;
    }
    
    // Bonus từ diversity của skills
    if matching_skills.len() >= 5 {
        growth_score += 2;
    }
    
    // Bonus nếu có ghost skills
    if !ghost_skills.is_empty() {
        growth_score += 1;
    }
    
    growth_score = std::cmp::min(10, growth_score);
    
    let growth_explanation = if growth_score >= 7 {
        format!("{}/10 - Cao: Thể hiện khả năng học hỏi và đa dạng kỹ năng, tiềm năng phát triển tốt", growth_score)
    } else if growth_score >= 5 {
        format!("{}/10 - Trung bình: Có nền tảng, cần đào tạo thêm để phát triển", growth_score)
    } else {
        format!("{}/10 - Hạn chế: Chưa thấy rõ khả năng học hỏi và phát triển", growth_score)
    };
    
    // ============================================
    // BƯỚC 6: TÍNH OVERALL SCORE (0-100)
    // ============================================
    let average_score = (tech_score as f32 + exp_score as f32 + culture_score as f32 + growth_score as f32) / 4.0;
    let overall_score = (average_score * 10.0).round() as u8;
    
    // ============================================
    // BƯỚC 7: XÁC ĐỊNH VERDICT
    // ============================================
    let verdict = if overall_score >= 75 {
        "Hire"
    } else if overall_score >= 50 {
        "Maybe"
    } else {
        "Pass"
    };
    
    // ============================================
    // BƯỚC 8: TẠO GIẢI THÍCH CHI TIẾT
    // ============================================
    let summary = if overall_score >= 75 {
        format!("⭐ Ứng viên đạt {}% - Phù hợp cao với vị trí. Có {}/{} kỹ năng yêu cầu và {} năng lực mềm quan trọng.",
                overall_score, matching_skills.len(), total_required, ghost_skills.len())
    } else if overall_score >= 50 {
        format!("⚡ Ứng viên đạt {}% - Phù hợp trung bình. Có tiềm năng nhưng cần bổ sung {} kỹ năng còn thiếu.",
                overall_score, missing_skills.len())
    } else {
        format!("⚠️ Ứng viên đạt {}% - Chưa phù hợp. Thiếu {}/{} kỹ năng cốt lõi, cần nhiều đào tạo.",
                overall_score, missing_skills.len(), total_required)
    };
    
    let mut strengths = Vec::new();
    
    if !matching_skills.is_empty() {
        let top_skills = matching_skills.iter().take(5).map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        strengths.push(format!("✅ Kỹ năng kỹ thuật: Có {} kỹ năng phù hợp ({})", matching_skills.len(), top_skills));
    }
    
    if !ghost_skills.is_empty() {
        strengths.push(format!("💡 Kỹ năng mềm: {}", ghost_skills.join(", ")));
    }
    
    if has_learning {
        strengths.push("📚 Khả năng tự học: CV thể hiện sự chủ động học hỏi và phát triển".to_string());
    }
    
    if tech_score >= 7 {
        strengths.push("🎯 Nền tảng kỹ thuật vững: Đáp ứng tốt yêu cầu công việc".to_string());
    }
    
    if strengths.is_empty() {
        strengths.push("Chưa có điểm mạnh nổi bật, cần đánh giá kỹ hơn trong phỏng vấn".to_string());
    }
    
    let mut weaknesses = Vec::new();
    
    if !missing_skills.is_empty() {
        let top_missing = missing_skills.iter().take(5).map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        weaknesses.push(format!("❌ Thiếu {} kỹ năng quan trọng: {}", missing_skills.len(), top_missing));
    }
    
    if !red_flags.is_empty() {
        for flag in &red_flags {
            weaknesses.push(format!("⚠️ {}", flag));
        }
    }
    
    if exp_score < 5 {
        weaknesses.push("📊 Kinh nghiệm hạn chế: CV chưa thể hiện rõ leadership và quản lý dự án".to_string());
    }
    
    if tech_score < 5 {
        weaknesses.push("🔧 Kỹ năng kỹ thuật chưa đủ: Cần đào tạo nhiều để đáp ứng yêu cầu".to_string());
    }
    
    if weaknesses.is_empty() {
        weaknesses.push("Không có điểm yếu đáng kể".to_string());
    }
    
    let recommendation = match verdict {
        "Hire" => format!("💚 ĐỀ XUẤT PHỎNG VẤN - Ứng viên đạt {}%, phù hợp cao với vị trí. Nên mời phỏng vấn để tìm hiểu sâu hơn về kinh nghiệm và văn hóa làm việc.", overall_score),
        "Maybe" => format!("💛 CÂN NHẮC KỸ - Ứng viên đạt {}%, có tiềm năng nhưng cần đánh giá thêm. Nên phỏng vấn để làm rõ các kỹ năng còn thiếu và khả năng học hỏi.", overall_score),
        _ => format!("❤️ KHÔNG PHÙ HỢP - Ứng viên chỉ đạt {}%, chưa đáp ứng yêu cầu cơ bản. Nên tìm ứng viên khác phù hợp hơn hoặc xem xét vị trí junior.", overall_score),
    };
    
    let explanation = ExplanationVietnamese {
        summary,
        strengths,
        weaknesses,
        score_breakdown: ScoreBreakdown {
            technical_match: tech_explanation,
            experience_level: exp_explanation,
            culture_fit: culture_explanation,
            growth_potential: growth_explanation,
        },
        recommendation,
    };
    
    // ============================================
    // BƯỚC 9: TRẢ VỀ KẾT QUẢ PHÂN TÍCH
    // ============================================
    AnalysisVerdict {
        overall_score,
        verdict: verdict.to_string(),
        matching_skills: matching_skills.iter().map(|s| s.to_uppercase()).collect(),
        missing_skills: missing_skills.iter().take(10).map(|s| s.to_uppercase()).collect(),
        ghost_skills,
        red_flags,
        confidence_level: if overall_score >= 70 { 0.9 } else if overall_score >= 50 { 0.75 } else { 0.6 },
        explanation_vietnamese: Some(explanation),
        skill_match_details: Some(skill_match),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_analysis() {
        let cv = "I have experience with Python, React, and Docker";
        let jd = "Looking for someone with Python, React, Kubernetes experience";
        
        let result = analyze_resume(cv, jd).await;
        assert!(result.is_ok());
        
        let verdict = result.unwrap();
        assert!(verdict.overall_score > 0);
        assert!(!verdict.matching_skills.is_empty());
    }
}
