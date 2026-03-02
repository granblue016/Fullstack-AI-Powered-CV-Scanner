use std::collections::{HashMap, HashSet};

pub struct KeywordMatchResult {
    pub matching_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub skill_diversity: usize,
    pub semantic_alignment: f32,
}

pub fn rush_bert_regex_match(
    cv_text: &str,
    jd_text: &str,
    technical_skills: &[&str],
) -> KeywordMatchResult {
    let cv = cv_text.to_lowercase();
    let jd = jd_text.to_lowercase();

    let cv_tokens = tokenize(&cv);
    let jd_tokens = tokenize(&jd);

    let jd_keywords = extract_job_keywords(&jd, &jd_tokens, technical_skills);

    let mut matching_skills = Vec::new();
    let mut missing_skills = Vec::new();

    for keyword in &jd_keywords {
        if contains_semantic_keyword(&cv, &cv_tokens, keyword) {
            matching_skills.push(keyword.to_uppercase());
        } else {
            missing_skills.push(keyword.to_uppercase());
        }
    }

    let skill_diversity = technical_skills
        .iter()
        .filter(|skill| contains_semantic_keyword(&cv, &cv_tokens, skill))
        .count();

    let semantic_alignment = semantic_alignment_score(&cv_tokens, &jd_tokens);

    KeywordMatchResult {
        matching_skills,
        missing_skills,
        skill_diversity,
        semantic_alignment,
    }
}

fn tokenize(text: &str) -> HashSet<String> {
    text
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
        .filter(|token| token.len() >= 2)
        .map(|token| normalize_alias(token))
        .filter(|token| !STOP_WORDS.contains(&token.as_str()))
        .collect()
}

fn extract_job_keywords(
    jd_text: &str,
    jd_tokens: &HashSet<String>,
    technical_skills: &[&str],
) -> Vec<String> {
    let mut keywords: HashSet<String> = HashSet::new();

    for &skill in technical_skills {
        if contains_semantic_keyword(jd_text, jd_tokens, skill) {
            keywords.insert(skill.to_string());
        }
    }

    let mut token_frequency: HashMap<String, usize> = HashMap::new();
    for token in jd_text
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
        .filter(|token| token.len() >= 4)
        .map(normalize_alias)
        .filter(|token| !STOP_WORDS.contains(&token.as_str()))
    {
        *token_frequency.entry(token).or_insert(0) += 1;
    }

    let mut frequent_tokens: Vec<(String, usize)> = token_frequency.into_iter().collect();
    frequent_tokens.sort_by(|a, b| b.1.cmp(&a.1));

    for (token, _) in frequent_tokens.into_iter().take(12) {
        keywords.insert(token);
    }

    let mut collected: Vec<String> = keywords.into_iter().collect();
    collected.sort();
    collected
}

fn semantic_alignment_score(cv_tokens: &HashSet<String>, jd_tokens: &HashSet<String>) -> f32 {
    if jd_tokens.is_empty() {
        return 0.0;
    }

    let overlap = jd_tokens.intersection(cv_tokens).count() as f32;
    let direct_ratio = overlap / jd_tokens.len() as f32;

    let soft_bonus = jd_tokens
        .iter()
        .filter(|token| has_semantic_neighbor(cv_tokens, token))
        .count() as f32
        / jd_tokens.len() as f32;

    (direct_ratio * 0.75 + soft_bonus * 0.25).clamp(0.0, 1.0)
}

fn contains_semantic_keyword(text: &str, tokens: &HashSet<String>, keyword: &str) -> bool {
    let normalized = normalize_alias(keyword);
    if tokens.contains(&normalized) || text.contains(keyword) {
        return true;
    }

    semantic_aliases(&normalized)
        .iter()
        .any(|alias| tokens.contains(*alias) || text.contains(alias))
}

fn has_semantic_neighbor(tokens: &HashSet<String>, token: &str) -> bool {
    semantic_aliases(token)
        .iter()
        .any(|alias| tokens.contains(*alias))
}

fn normalize_alias(token: &str) -> String {
    match token.trim().to_lowercase().as_str() {
        "js" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "node" => "nodejs".to_string(),
        "node.js" => "nodejs".to_string(),
        "postgres" => "postgresql".to_string(),
        "k8s" => "kubernetes".to_string(),
        other => other.to_string(),
    }
}

fn semantic_aliases(token: &str) -> Vec<&'static str> {
    match token {
        "javascript" => vec!["js", "ecmascript"],
        "typescript" => vec!["ts"],
        "nodejs" => vec!["node", "node.js", "express"],
        "postgresql" => vec!["postgres"],
        "kubernetes" => vec!["k8s"],
        "react" => vec!["nextjs", "jsx"],
        "python" => vec!["django", "flask", "fastapi"],
        "aws" => vec!["ec2", "s3", "lambda"],
        _ => Vec::new(),
    }
}

const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "you", "your", "our", "are",
    "have", "has", "will", "all", "can", "not", "more", "about", "years", "year",
    "kinh", "nghiem", "nam", "voi", "cho", "mot", "cac", "khong", "trong", "neu", "ung",
    "vien", "cong", "ty", "vi", "tri", "yeu", "cau", "candidate", "company", "role", "job",
];
