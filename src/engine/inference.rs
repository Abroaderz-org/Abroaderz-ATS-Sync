use crate::engine::schema::CandidateRecord;
use regex::Regex;
use strsim::jaro_winkler;

pub fn infer_candidate_details(raw_text: &str, file_name: &str) -> CandidateRecord {
    let mut record = CandidateRecord::default();

    // Candidate Name from sanitized file name
    let clean_name = file_name
        .trim_end_matches(".pdf")
        .trim_end_matches(".png")
        .trim_end_matches(".jpg")
        .trim_end_matches(".jpeg")
        .trim_end_matches(".webp")
        .replace(['_', '-'], " ");
    record.name = clean_name;

    // Contact matching
    let email_re = Regex::new(r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}").unwrap();
    if let Some(mat) = email_re.find(raw_text) {
        record.email = mat.as_str().to_string();
    }

    let phone_re = Regex::new(r"(\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}").unwrap();
    if let Some(mat) = phone_re.find(raw_text) {
        record.phone = mat.as_str().to_string();
    }

    // Passport detection (standard format: 1 Letter + 7 Digits)
    let passport_re = Regex::new(r"\b[A-Z][0-9]{7}\b").unwrap();
    if let Some(mat) = passport_re.find(raw_text) {
        record.passport_no = mat.as_str().to_string();
    }

    // Fuzzy GCC / Overseas Experience Matching
    let gcc_locations = ["Dubai", "UAE", "Saudi Arabia", "Qatar", "Oman", "Kuwait", "Bahrain", "Abu Dhabi"];
    let mut gcc_matches = Vec::new();

    for word in raw_text.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
        for &loc in &gcc_locations {
            if jaro_winkler(clean_word, loc) > 0.88 {
                if !gcc_matches.contains(&loc.to_string()) {
                    gcc_matches.push(loc.to_string());
                }
            }
        }
    }

    if !gcc_matches.is_empty() {
        record.overseas_experience = gcc_matches.join(", ");
    }

    record
}