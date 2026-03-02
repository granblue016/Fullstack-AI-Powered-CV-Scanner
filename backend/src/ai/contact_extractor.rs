use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CandidateContactInfo {
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub personal_site: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HrContactInfo {
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub company_website: Option<String>,
    pub company_address: Option<String>,
    pub company_name: Option<String>,
    pub position_title: Option<String>,
}

pub fn extract_candidate_contact_info(cv_text: &str) -> CandidateContactInfo {
    CandidateContactInfo {
        full_name: extract_name(cv_text),
        phone: extract_phone(cv_text),
        email: extract_email(cv_text),
        personal_site: extract_website(cv_text),
        address: extract_address(cv_text),
    }
}

pub fn extract_hr_contact_info(jd_text: &str) -> HrContactInfo {
    HrContactInfo {
        full_name: extract_hr_name(jd_text),
        phone: extract_phone(jd_text),
        email: extract_email(jd_text),
        company_website: extract_website(jd_text),
        company_address: extract_address(jd_text),
        company_name: extract_company_name(jd_text),
        position_title: extract_position_title(jd_text),
    }
}

fn extract_email(text: &str) -> Option<String> {
    Regex::new(r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}")
        .ok()?
        .find(text)
        .map(|m| m.as_str().to_string())
}

fn extract_phone(text: &str) -> Option<String> {
    Regex::new(r"(?x)(\+?\d{1,3}[\s.-]?)?(\(?\d{2,4}\)?[\s.-]?)?\d{3,4}[\s.-]?\d{3,4}")
        .ok()?
        .find_iter(text)
        .map(|m| m.as_str().trim().to_string())
        .find(|value| value.chars().filter(|c| c.is_ascii_digit()).count() >= 9)
}

fn extract_website(text: &str) -> Option<String> {
    Regex::new(r"(?i)(https?://)?(www\.)?[a-z0-9-]+\.[a-z]{2,}(\/[\w./?%&=-]*)?")
        .ok()?
        .find_iter(text)
        .map(|m| m.as_str().trim().to_string())
        .find(|site| {
            let lower = site.to_lowercase();
            !lower.contains("@") && !lower.ends_with(".png") && !lower.ends_with(".jpg")
        })
}

fn extract_address(text: &str) -> Option<String> {
    for line in text.lines() {
        let normalized = line.trim();
        let lower = normalized.to_lowercase();
        if normalized.len() >= 10
            && (lower.contains("address")
                || lower.contains("địa chỉ")
                || lower.contains("street")
                || lower.contains("district")
                || lower.contains("city")
                || lower.contains("hà nội")
                || lower.contains("hồ chí minh")
                || lower.contains("tphcm"))
        {
            return Some(normalized.to_string());
        }
    }
    None
}

fn extract_name(text: &str) -> Option<String> {
    for line in text.lines().take(12) {
        let candidate = line.trim();
        if candidate.len() >= 4
            && candidate.len() <= 60
            && !candidate.contains('@')
            && !candidate.chars().any(|c| c.is_ascii_digit())
            && candidate.split_whitespace().count() >= 2
        {
            return Some(candidate.to_string());
        }
    }
    None
}

fn extract_hr_name(text: &str) -> Option<String> {
    Regex::new(r"(?i)(contact|liên hệ|hr|recruiter|hiring manager)\s*[:\-]?\s*([\p{L} .'-]{4,})")
        .ok()?
        .captures(text)
        .and_then(|caps| caps.get(2).map(|m| m.as_str().trim().to_string()))
}

fn extract_company_name(text: &str) -> Option<String> {
    Regex::new(r"(?i)(company|công ty|employer)\s*[:\-]?\s*([\p{L}0-9 &.,'-]{2,})")
        .ok()?
        .captures(text)
        .and_then(|caps| caps.get(2).map(|m| m.as_str().trim().to_string()))
}

fn extract_position_title(text: &str) -> Option<String> {
    Regex::new(r"(?i)(position|job title|vị trí|role)\s*[:\-]?\s*([\p{L}0-9 /&().,'+-]{2,})")
        .ok()?
        .captures(text)
        .and_then(|caps| caps.get(2).map(|m| m.as_str().trim().to_string()))
}
