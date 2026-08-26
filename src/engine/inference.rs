use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub struct InferenceEngine;

impl InferenceEngine {
    pub fn process_candidate(raw_text: &str, file_name: &str) -> CandidateRecord {
        let mut candidate = CandidateRecord {
            name: file_name.to_string(),
            passport_no: "Not Found".to_string(),
            position: "Not Found".to_string(),
            education: "Not Found".to_string(),
            dob: "Not Found".to_string(),
            phone: "Not Found".to_string(),
            email: "Not Found".to_string(),
            local_experience: "Not Found".to_string(),
            overseas_experience: "Not Found".to_string(),
            total_experience: "Not Found".to_string(),
            state: "Not Found".to_string(),
            country: "Not Found".to_string(),
            score: None,
        };

        // 1. Contact Patterns
        if let Some(email) = find_match(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", raw_text) {
            candidate.email = email;
        }

        if let Some(phone) = find_match(r"(\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}", raw_text) {
            candidate.phone = phone;
        }

        // 2. Passport Format (Standard 1 letter followed by 7 digits)
        if let Some(passport) = find_match(r"\b[A-Z][0-9]{7}\b", raw_text) {
            candidate.passport_no = passport;
        }

        // 3. Technical Qualifications
        let edu_keywords = ["B.Tech", "B.E.", "Diploma", "ITI", "HSC", "SSLC", "Bachelor", "Master"];
        for edu in edu_keywords {
            if raw_text.to_lowercase().contains(&edu.to_lowercase()) {
                candidate.education = edu.to_string();
                break;
            }
        }

        // 4. Overseas & GCC Experience Detection
        let overseas_locations = ["Dubai", "UAE", "Saudi", "KSA", "Qatar", "Oman", "Kuwait", "Bahrain", "Gulf"];
        let is_overseas = overseas_locations.iter().any(|loc| raw_text.to_lowercase().contains(&loc.to_lowercase()));
        
        if is_overseas {
            candidate.overseas_experience = "GCC / Overseas Experience Found".to_string();
        }

        // 5. Total Experience Detection (e.g. "5 years", "3+ yrs")
        if let Some(exp) = find_match(r"(?i)\b\d{1,2}\+?\s*(years?|yrs?)\b", raw_text) {
            candidate.total_experience = exp;
        }

        candidate
    }
}

fn find_match(pattern: &str, text: &str) -> Option<String> {
    Regex::new(pattern).ok()?.find(text).map(|m| m.as_str().to_string())
}