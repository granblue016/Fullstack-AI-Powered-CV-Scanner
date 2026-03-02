use serde::{Deserialize, Serialize};
use crate::ai::skill_matcher::SkillMatchResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub technical_match: String,
    pub experience_level: String,
    pub culture_fit: String,
    pub growth_potential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationVietnamese {
    pub summary: String,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub score_breakdown: ScoreBreakdown,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisVerdict {
    pub overall_score: u8,          // 0-100
    pub verdict: String,             // "Hire", "Maybe", "Pass"
    pub matching_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub ghost_skills: Vec<String>,   // Implied but not stated
    pub red_flags: Vec<String>,      // Gaps, job hopping
    pub confidence_level: f32,       // AI confidence 0.0-1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation_vietnamese: Option<ExplanationVietnamese>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_match_details: Option<SkillMatchResult>,
}

impl Default for AnalysisVerdict {
    fn default() -> Self {
        Self {
            overall_score: 0,
            verdict: String::from("Pass"),
            matching_skills: Vec::new(),
            missing_skills: Vec::new(),
            ghost_skills: Vec::new(),
            red_flags: Vec::new(),
            confidence_level: 0.0,
            explanation_vietnamese: None,
            skill_match_details: None,
        }
    }
}
