/// SKILL & QUALIFICATION MATCHER MODULE
/// ======================================
/// Extracts and matches skills, experience, and certifications between CV and JD

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================
// DATA STRUCTURES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatchResult {
    pub overall_match_percentage: u8,
    pub matched_skills: Vec<SkillMatch>,
    pub missing_skills: Vec<String>,
    pub extra_skills: Vec<SkillMatch>,
    pub matched_certificates: Vec<String>,
    pub missing_certificates: Vec<String>,
    pub total_experience_years: Option<f32>,
    pub required_experience_years: Option<f32>,
    pub candidate_experience_level: Option<String>,
    pub required_experience_level: Option<String>,
    pub experience_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill_name: String,
    pub years_experience: Option<f32>,
    pub proficiency_level: Option<String>,
    pub context: String, // Where it was found in CV
}

// ============================================
// MAIN INTERFACE
// ============================================

/// Analyze how well CV skills match JD requirements
pub fn analyze_skill_match(cv_text: &str, jd_text: &str) -> SkillMatchResult {
    // Extract from CV
    let cv_skills = extract_skills_from_cv(cv_text);
    let cv_certificates = extract_certificates(cv_text);
    let cv_total_experience = extract_total_experience(cv_text);
    let cv_experience_level = extract_experience_level(cv_text);

    // Extract from JD
    let jd_required_skills = extract_required_skills_from_jd(jd_text);
    let jd_required_certificates = extract_required_certificates_from_jd(jd_text);
    let jd_required_experience = extract_required_experience_from_jd(jd_text);
    let jd_required_level = extract_experience_level(jd_text);

    // Match skills
    let (matched, missing, extra) = match_skills(&cv_skills, &jd_required_skills);

    // Match certificates
    let (matched_certs, missing_certs) = match_certificates(&cv_certificates, &jd_required_certificates);

    // Calculate overall match percentage
    let match_percentage = calculate_match_percentage(
        matched.len(),
        jd_required_skills.len(),
        matched_certs.len(),
        jd_required_certificates.len(),
        &cv_total_experience,
        &jd_required_experience,
    );

    // Check if experience requirement is met
    let experience_match = is_experience_match(
        cv_total_experience,
        jd_required_experience,
        cv_experience_level.as_deref(),
        jd_required_level.as_deref(),
    );

    SkillMatchResult {
        overall_match_percentage: match_percentage,
        matched_skills: matched,
        missing_skills: missing,
        extra_skills: extra,
        matched_certificates: matched_certs,
        missing_certificates: missing_certs,
        total_experience_years: cv_total_experience,
        required_experience_years: jd_required_experience,
        candidate_experience_level: cv_experience_level,
        required_experience_level: jd_required_level,
        experience_match,
    }
}

// ============================================
// SKILL EXTRACTION FROM CV
// ============================================

fn extract_skills_from_cv(text: &str) -> Vec<SkillMatch> {
    let mut skills = Vec::new();
    let text_lower = text.to_lowercase();
    let known_skills = get_common_technologies();

    // Pattern 1: "Python (3 years)", "Java: 5 years"
    let pattern1 = Regex::new(r"(?i)([a-z][a-z0-9+#.\-]{0,30})\s*[:\(]\s*(\d+\.?\d*)\s*\+?\s*years?\)?").unwrap();
    for cap in pattern1.captures_iter(text) {
        if let (Some(skill), Some(years)) = (cap.get(1), cap.get(2)) {
            if let Some(skill_name) = map_to_known_skill(skill.as_str(), &known_skills) {
                skills.push(SkillMatch {
                    skill_name: skill_name.clone(),
                    years_experience: years.as_str().parse().ok(),
                    proficiency_level: None,
                    context: cap.get(0).unwrap().as_str().to_string(),
                });
            }
        }
    }

    // Pattern 2: "3+ years of Python", "5 years experience with React"
    let pattern2 = Regex::new(r"(?i)(\d+\.?\d*)\s*\+?\s*years?\s+(?:of\s+)?(?:experience\s+)?(?:with\s+|in\s+|using\s+)?([a-z][a-z0-9+#.\-]{0,30})").unwrap();
    for cap in pattern2.captures_iter(text) {
        if let (Some(years), Some(skill)) = (cap.get(1), cap.get(2)) {
            if let Some(skill_name) = map_to_known_skill(skill.as_str(), &known_skills) {
                skills.push(SkillMatch {
                    skill_name: skill_name.clone(),
                    years_experience: years.as_str().parse().ok(),
                    proficiency_level: None,
                    context: cap.get(0).unwrap().as_str().to_string(),
                });
            }
        }
    }

    // Pattern 3: "Senior Python Developer", "Junior C++ Engineer", "Intern Backend"
    let pattern3 = Regex::new(r"(?i)(intern|fresher|junior|mid|senior|lead|principal|expert|advanced|proficient|skilled)\s+(?:in\s+)?([a-z][a-z0-9+#.\-]{0,30})").unwrap();
    for cap in pattern3.captures_iter(text) {
        if let (Some(level), Some(skill)) = (cap.get(1), cap.get(2)) {
            if let Some(skill_name) = map_to_known_skill(skill.as_str(), &known_skills) {
                skills.push(SkillMatch {
                    skill_name: skill_name.clone(),
                    years_experience: None,
                    proficiency_level: Some(normalize_experience_level(level.as_str()).to_string()),
                    context: cap.get(0).unwrap().as_str().to_string(),
                });
            }
        }
    }

    // Pattern 4: Technology list - "Skills: Python, Java, React, Docker"
    let pattern4 = Regex::new(r"(?i)(?:skills?|technologies|tech stack|tools):\s*([a-z0-9+#.,\s\-/]+)").unwrap();
    for cap in pattern4.captures_iter(&text_lower) {
        if let Some(skill_list) = cap.get(1) {
            let list_text = skill_list.as_str();
            for skill in list_text.split(&[',', '/', '\n'][..]) {
                if let Some(skill_name) = map_to_known_skill(skill.trim(), &known_skills) {
                    skills.push(SkillMatch {
                        skill_name: skill_name.clone(),
                        years_experience: None,
                        proficiency_level: None,
                        context: format!("Listed in skills: {}", skill_name),
                    });
                }
            }
        }
    }

    // Pattern 5: Date ranges - "Python Developer (2020-2023)"
    let pattern5 = Regex::new(r"(?i)([a-z][a-z0-9+#.\-]{0,30})\s+(?:developer|engineer|specialist)?\s*\(?\s*(\d{4})\s*-\s*(\d{4}|present|current)\)?").unwrap();
    for cap in pattern5.captures_iter(text) {
        if let (Some(skill), Some(start), Some(end)) = (cap.get(1), cap.get(2), cap.get(3)) {
            if let Some(skill_name) = map_to_known_skill(skill.as_str(), &known_skills) {
                let start_year: i32 = start.as_str().parse().unwrap_or(0);
                let end_year: i32 = if end.as_str().to_lowercase().contains("present") || end.as_str().to_lowercase().contains("current") {
                    2026 // Current year
                } else {
                    end.as_str().parse().unwrap_or(2026)
                };
                let years = (end_year - start_year).max(0) as f32;
                
                skills.push(SkillMatch {
                    skill_name: skill_name.clone(),
                    years_experience: Some(years),
                    proficiency_level: None,
                    context: cap.get(0).unwrap().as_str().to_string(),
                });
            }
        }
    }

    // Pattern 6: Add direct known tech mentions (C, C++, Python, ...)
    for skill in extract_known_skills_from_text(text, &known_skills) {
        skills.push(SkillMatch {
            skill_name: skill.clone(),
            years_experience: None,
            proficiency_level: None,
            context: format!("Detected in CV text: {}", skill),
        });
    }

    // Deduplicate skills (keep the one with most info)
    deduplicate_skills(skills)
}

// ============================================
// CERTIFICATE EXTRACTION
// ============================================

fn extract_certificates(text: &str) -> Vec<String> {
    let mut certificates = Vec::new();

    // Common certificate patterns
    let cert_patterns = vec![
        // AWS
        Regex::new(r"(?i)aws\s+certified\s+([a-z\s\-]+?)(?:\s|,|\.|$)").unwrap(),
        // Google Cloud
        Regex::new(r"(?i)google\s+cloud\s+(?:certified\s+)?([a-z\s\-]+?)(?:\s|,|\.|$)").unwrap(),
        // Microsoft
        Regex::new(r"(?i)microsoft\s+certified\s+([a-z\s\-:]+?)(?:\s|,|\.|$)").unwrap(),
        // Kubernetes
        Regex::new(r"(?i)(?:certified\s+)?kubernetes\s+(?:administrator|developer|security)\s*\(?(CKA|CKD|CKS)?\)?").unwrap(),
        // PMP, CISSP, etc
        Regex::new(r"(?i)\b(PMP|CISSP|CEH|CISA|CISM|CompTIA\s+\w+|ITIL|Scrum Master|CSM|PSM)\b").unwrap(),
        // General "Certified X" pattern
        Regex::new(r"(?i)certified\s+([a-z][a-z\s\-]{3,40})(?:certificate|certification)?").unwrap(),
    ];

    for pattern in &cert_patterns {
        for cap in pattern.captures_iter(text) {
            if let Some(cert) = cap.get(0) {
                let cert_text = cert.as_str().trim();
                if cert_text.len() > 3 {
                    certificates.push(cert_text.to_string());
                }
            }
        }
    }

    // Deduplicate
    let unique: HashSet<String> = certificates.into_iter().collect();
    unique.into_iter().collect()
}

// ============================================
// EXPERIENCE EXTRACTION
// ============================================

fn extract_total_experience(text: &str) -> Option<f32> {
    // Pattern: "5+ years of experience", "10 years total experience"
    let exp_pattern = Regex::new(r"(?i)(\d+\.?\d*)\s*\+?\s*years?\s+(?:of\s+)?(?:total\s+|professional\s+)?experience").unwrap();
    if let Some(cap) = exp_pattern.captures(text) {
        if let Some(years) = cap.get(1) {
            return years.as_str().parse().ok();
        }
    }

    // Pattern: Find oldest job start date
    let date_pattern = Regex::new(r"(?i)(?:since|from)\s+(\d{4})").unwrap();
    let mut oldest_year = 2026;
    for cap in date_pattern.captures_iter(text) {
        if let Some(year) = cap.get(1) {
            if let Ok(y) = year.as_str().parse::<i32>() {
                if y < oldest_year && y > 1990 {
                    oldest_year = y;
                }
            }
        }
    }
    if oldest_year < 2026 {
        return Some((2026 - oldest_year) as f32);
    }

    None
}

// ============================================
// JD REQUIREMENTS EXTRACTION
// ============================================

fn extract_required_skills_from_jd(text: &str) -> Vec<String> {
    let mut skills = HashSet::new();
    let known_skills = get_common_technologies();

    // Pattern 1: "Experience with Python", "Knowledge of Java"
    let pattern1 = Regex::new(r"(?i)(?:experience|knowledge|proficiency|expertise|familiarity)\s+(?:with|in|of)\s+([a-z][a-z0-9+#.\-]{0,30})").unwrap();
    for cap in pattern1.captures_iter(text) {
        if let Some(skill) = cap.get(1) {
            if let Some(skill_name) = map_to_known_skill(skill.as_str(), &known_skills) {
                skills.insert(skill_name);
            }
        }
    }

    // Pattern 2: "Required: Python, Java, React"
    let pattern2 = Regex::new(r"(?i)(?:required|must have|should have|needs?|requirements?):\s*([a-z0-9+#.,\s\-/]+)").unwrap();
    for cap in pattern2.captures_iter(text) {
        if let Some(skill_list) = cap.get(1) {
            for skill in skill_list.as_str().split(&[',', '/', '\n'][..]) {
                if let Some(skill_name) = map_to_known_skill(skill.trim(), &known_skills) {
                    skills.insert(skill_name);
                }
            }
        }
    }

    // Pattern 3: Direct known technology mentions with strict boundary matching
    for skill in extract_known_skills_from_text(text, &known_skills) {
        skills.insert(skill);
    }

    skills.into_iter().collect()
}

fn extract_required_certificates_from_jd(text: &str) -> Vec<String> {
    let mut certificates = Vec::new();

    // Pattern: "AWS Certified", "certification in", etc
    let cert_pattern = Regex::new(r"(?i)(?:require[sd]|prefer(?:red)?|bonus).*?(aws certified|google cloud|microsoft certified|kubernetes|pmp|cissp|ceh|cisa|cism|comptia|itil|scrum master|csm|psm)[a-z\s\-]*(?:certification|certificate)?").unwrap();
    for cap in cert_pattern.captures_iter(text) {
        if let Some(cert) = cap.get(1) {
            certificates.push(cert.as_str().to_string());
        }
    }

    // Also use general certificate extraction
    certificates.extend(extract_certificates(text));

    let unique: HashSet<String> = certificates.into_iter().collect();
    unique.into_iter().collect()
}

fn extract_required_experience_from_jd(text: &str) -> Option<f32> {
    // Pattern: "3+ years", "5-7 years of experience", "0-1 years"
    let exp_pattern = Regex::new(r"(?i)(\d+\.?\d*)\s*\+?\s*(?:-\s*(\d+\.?\d*))?\s*years?\s+(?:of\s+)?experience").unwrap();
    if let Some(cap) = exp_pattern.captures(text) {
        if let Some(min_years) = cap.get(1) {
            if let Some(max_years) = cap.get(2) {
                let min_val: f32 = min_years.as_str().parse().unwrap_or(0.0);
                let max_val: f32 = max_years.as_str().parse().unwrap_or(min_val);
                return Some((min_val + max_val) / 2.0);
            }
            return min_years.as_str().parse().ok();
        }
    }

    None
}

fn extract_experience_level(text: &str) -> Option<String> {
    let level_pattern = Regex::new(r"(?i)\b(intern|fresher|junior|mid(?:-level)?|senior|lead|principal)\b").unwrap();
    level_pattern
        .captures(text)
        .and_then(|cap| cap.get(1).map(|m| normalize_experience_level(m.as_str()).to_string()))
}

fn normalize_experience_level(level: &str) -> &'static str {
    match level.trim().to_lowercase().as_str() {
        "intern" => "Intern",
        "fresher" => "Fresher",
        "junior" => "Junior",
        "mid" | "mid-level" => "Mid",
        "senior" => "Senior",
        "lead" => "Lead",
        "principal" => "Principal",
        _ => "Unknown",
    }
}

fn level_rank(level: &str) -> i32 {
    match level.to_lowercase().as_str() {
        "intern" => 0,
        "fresher" => 1,
        "junior" => 2,
        "mid" => 3,
        "senior" => 4,
        "lead" => 5,
        "principal" => 6,
        _ => -1,
    }
}

fn is_experience_match(
    cv_years: Option<f32>,
    jd_years: Option<f32>,
    cv_level: Option<&str>,
    jd_level: Option<&str>,
) -> bool {
    let years_match = match (cv_years, jd_years) {
        (Some(cv), Some(jd)) => cv >= jd,
        (_, None) => true,
        (None, Some(_)) => false,
    };

    let level_match = match (cv_level, jd_level) {
        (Some(cv), Some(jd)) => level_rank(cv) >= level_rank(jd),
        (_, None) => true,
        (None, Some(_)) => false,
    };

    years_match && level_match
}

// ============================================
// MATCHING LOGIC
// ============================================

fn match_skills(cv_skills: &[SkillMatch], jd_required: &[String]) -> (Vec<SkillMatch>, Vec<String>, Vec<SkillMatch>) {
    let mut matched = Vec::new();
    let mut missing = Vec::new();
    let mut extra = Vec::new();

    // Find matches and missing
    for required in jd_required {
        let required_lower = required.to_lowercase();
        if let Some(cv_skill) = cv_skills.iter().find(|s| s.skill_name.to_lowercase() == required_lower) {
            matched.push(cv_skill.clone());
        } else {
            missing.push(required.clone());
        }
    }

    // Find extra skills (in CV but not required in JD)
    let jd_skill_names: HashSet<String> = jd_required.iter()
        .map(|s| s.to_lowercase())
        .collect();

    for cv_skill in cv_skills {
        if !jd_skill_names.contains(&cv_skill.skill_name.to_lowercase()) {
            extra.push(cv_skill.clone());
        }
    }

    (matched, missing, extra)
}

fn match_certificates(cv_certs: &[String], jd_required_certs: &[String]) -> (Vec<String>, Vec<String>) {
    let mut matched = Vec::new();
    let mut missing = Vec::new();

    for required in jd_required_certs {
        let found = cv_certs.iter().any(|cv_cert| {
            cv_cert.to_lowercase().contains(&required.to_lowercase()) ||
            required.to_lowercase().contains(&cv_cert.to_lowercase())
        });

        if found {
            matched.push(required.clone());
        } else {
            missing.push(required.clone());
        }
    }

    (matched, missing)
}

fn calculate_match_percentage(
    matched_skills: usize,
    required_skills: usize,
    matched_certs: usize,
    required_certs: usize,
    cv_experience: &Option<f32>,
    jd_experience: &Option<f32>,
) -> u8 {
    let mut total_weight = 0.0;
    let mut achieved_weight = 0.0;

    // Skills (70% weight)
    if required_skills > 0 {
        total_weight += 70.0;
        achieved_weight += (matched_skills as f32 / required_skills as f32) * 70.0;
    }

    // Certificates (20% weight)
    if required_certs > 0 {
        total_weight += 20.0;
        achieved_weight += (matched_certs as f32 / required_certs as f32) * 20.0;
    }

    // Experience (10% weight)
    if let (Some(cv_exp), Some(jd_exp)) = (cv_experience, jd_experience) {
        total_weight += 10.0;
        if cv_exp >= jd_exp {
            achieved_weight += 10.0;
        } else {
            achieved_weight += (cv_exp / jd_exp) * 10.0;
        }
    }

    if total_weight == 0.0 {
        return 50; // Default if no requirements found
    }

    ((achieved_weight / total_weight) * 100.0).min(100.0) as u8
}

// ============================================
// HELPER FUNCTIONS
// ============================================

fn normalize_skill_name(skill: &str) -> String {
    let normalized = skill.trim().to_lowercase();
    
    // Map common aliases
    match normalized.as_str() {
        "js" => "JavaScript".to_string(),
        "ts" => "TypeScript".to_string(),
        "py" => "Python".to_string(),
        "k8s" => "Kubernetes".to_string(),
        "react.js" | "reactjs" => "React".to_string(),
        "node.js" | "nodejs" => "Node".to_string(),
        "vue.js" | "vuejs" => "Vue".to_string(),
        "csharp" | "c#" => "C#".to_string(),
        "cplusplus" | "c++" => "C++".to_string(),
        _ => {
            // Capitalize first letter
            let mut chars = normalized.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

fn map_to_known_skill(raw_skill: &str, known_skills: &[String]) -> Option<String> {
    let normalized = normalize_skill_name(raw_skill);
    let normalized_lower = normalized.to_lowercase();

    known_skills
        .iter()
        .find(|skill| skill.to_lowercase() == normalized_lower)
        .cloned()
}

fn extract_known_skills_from_text(text: &str, known_skills: &[String]) -> Vec<String> {
    let mut extracted = HashSet::new();
    let text_lower = text.to_lowercase();

    for skill in known_skills {
        if contains_skill_phrase(&text_lower, &skill.to_lowercase()) {
            extracted.insert(skill.clone());
        }
    }

    extracted.into_iter().collect()
}

fn contains_skill_phrase(text_lower: &str, skill_lower: &str) -> bool {
    let escaped = regex::escape(skill_lower);
    let pattern = if skill_lower.chars().all(|c| c.is_alphanumeric()) {
        format!(r"(?i)\b{}\b", escaped)
    } else {
        format!(r"(?i)(^|[^a-z0-9]){}($|[^a-z0-9])", escaped)
    };

    Regex::new(&pattern)
        .map(|re| re.is_match(text_lower))
        .unwrap_or(false)
}

fn is_valid_skill(skill: &str) -> bool {
    let skill_lower = skill.to_lowercase();
    
    // Filter out common false positives
    let blacklist = vec![
        "experience", "years", "year", "strong", "good", "excellent",
        "knowledge", "skills", "ability", "developed", "using",
        "working", "building", "creating", "project", "projects",
        "team", "company", "position", "role", "responsibilities"
    ];
    
    for word in &blacklist {
        if skill_lower == *word {
            return false;
        }
    }

    if skill_lower.chars().all(|c| c.is_numeric()) {
        return false;
    }
    
    // Must be at least 2 characters
    skill.len() >= 2
}

fn get_common_technologies() -> Vec<String> {
    vec![
        // Languages
        "Python", "Java", "JavaScript", "TypeScript", "Go", "Rust", "C", "C++", "C#",
        "Ruby", "PHP", "Swift", "Kotlin", "Scala", "Dart", "Elixir", "Haskell",
        
        // Frontend
        "React", "Vue", "Angular", "Svelte", "Next.js", "Nuxt", "HTML", "CSS", "SASS", "Tailwind",
        
        // Backend
        "Node", "Express", "Django", "Flask", "FastAPI", "Spring", "Rails", "Laravel",
        "Axum", "Actix", "Rocket",
        
        // Databases
        "PostgreSQL", "MySQL", "MongoDB", "Redis", "Elasticsearch", "Cassandra", "DynamoDB",
        "SQLite", "MariaDB", "Oracle",
        
        // DevOps
        "Docker", "Kubernetes", "AWS", "GCP", "Azure", "Terraform", "Ansible", "Jenkins",
        "GitLab", "GitHub", "CircleCI", "Travis",
        
        // Tools
        "Git", "Linux", "Nginx", "Apache", "GraphQL", "REST", "gRPC", "WebSocket",
        "RabbitMQ", "Kafka", "Spark", "Hadoop",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

fn deduplicate_skills(mut skills: Vec<SkillMatch>) -> Vec<SkillMatch> {
    let mut seen = HashMap::new();
    let mut result = Vec::new();

    for skill in skills.drain(..) {
        let key = skill.skill_name.to_lowercase();
        
        if let Some(existing) = seen.get(&key) {
            // Keep the one with more information
            let existing_skill: &SkillMatch = &result[*existing];
            let has_more_info = skill.years_experience.is_some() && existing_skill.years_experience.is_none()
                || skill.proficiency_level.is_some() && existing_skill.proficiency_level.is_none();
            
            if has_more_info {
                result[*existing] = skill;
            }
        } else {
            seen.insert(key, result.len());
            result.push(skill);
        }
    }

    result
}
