pub const ANALYSIS_PROMPT: &str = r#"You are an expert HR analyst. Compare the candidate's CV against the Job Description.

Instructions:
1. Calculate an overall match score (0-100)
2. Identify MATCHING skills (explicitly stated in CV that match JD)
3. Identify MISSING skills (in JD but not in CV)
4. Identify GHOST SKILLS (skills implied by experience but not explicitly listed)
5. Flag RED FLAGS (employment gaps >6 months, frequent job changes <1 year)
6. Provide a verdict: "Hire" (80+), "Maybe" (60-79), "Pass" (<60)

Return ONLY valid JSON matching this exact structure:
{
  "overall_score": 85,
  "verdict": "Hire",
  "matching_skills": ["Python", "React"],
  "missing_skills": ["Kubernetes"],
  "ghost_skills": ["Team leadership"],
  "red_flags": ["6-month gap in 2023"],
  "confidence_level": 0.95
}

Do not include any markdown formatting, explanations, or additional text. Only return the JSON object.
"#;
