use crate::engine::schema::CandidateRecord;
use regex::Regex;
use strsim::jaro_winkler;

pub fn infer_candidate_details(raw_text: &str, file_name: &str) -> CandidateRecord {
    let mut record = CandidateRecord::default();

    // 1. Clean Candidate Name from File Name
    let mut clean_name = file_name
        .trim_end_matches(".docx")
        .trim_end_matches(".doc")
        .trim_end_matches(".pdf")
        .trim_end_matches(".png")
        .trim_end_matches(".jpg")
        .trim_end_matches(".jpeg")
        .trim_end_matches(".webp")
        .replace(['_', '-'], " ");

    // 2. Extract Position / Trade (from text or filename)
    let pos_re = Regex::new(r"(?i)\b(civil site engineer|civil engineer|electrical engineer|mechanical engineer|hvac technician|safety officer|site engineer|project coordinator|electrician|plumber|foreman|pipe\s*fitter|welder|mason|carpenter|driver|scaffolder|surveyor|operator)\b").unwrap();
    if let Some(mat) = pos_re.find(raw_text) {
        let pos_str = mat.as_str();
        record.position = pos_str
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
    }

    // Strip common trade titles from the candidate's name if present in filename
    let trade_cleaner = Regex::new(r"(?i)\b(civil engineer|electrical engineer|site engineer|electrician|plumber|foreman|welder|driver|technician)\b").unwrap();
    clean_name = trade_cleaner.replace_all(&clean_name, "").to_string();
    record.name = clean_name.trim().to_string();

    // 3. Email Detection
    let email_re = Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").unwrap();
    if let Some(mat) = email_re.find(raw_text) {
        record.email = mat.as_str().to_lowercase();
    }

    // 4. Flexible International Phone Number Regex
    let phone_re = Regex::new(r"(\+\d{1,3}[\s-]?)?(\(?\d{2,4}\)?[\s-]?)?\d{3,4}[\s-]?\d{3,4}").unwrap();
    for line in raw_text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("phone") || lower.contains("contact") || lower.contains("mob") || lower.contains("tel") {
            if let Some(mat) = phone_re.find(line) {
                let num = mat.as_str().trim();
                if num.len() >= 8 {
                    record.phone = num.to_string();
                    break;
                }
            }
        }
    }
    if record.phone.is_empty() || record.phone == "Not Found" {
        if let Some(mat) = phone_re.find(raw_text) {
            let num = mat.as_str().trim();
            if num.len() >= 8 {
                record.phone = num.to_string();
            }
        }
    }

    // 5. Passport Detection (Standard 1 Letter + 7 Digits)
    let passport_re = Regex::new(r"(?i)\b(?:passport(?:\s*(?:no|num|number))?[:\s\-]*)?([a-z][0-9]{7})\b").unwrap();
    if let Some(caps) = passport_re.captures(raw_text) {
        if let Some(mat) = caps.get(1) {
            record.passport_no = mat.as_str().to_uppercase();
        }
    }

    // 6. Education / Degree Extraction
    let edu_re = Regex::new(r"(?i)\b(bachelor\s+of\s+science[^\n,]+|bachelor\s+of\s+engineering[^\n,]+|b\.?tech[^\n,]*|b\.?e\.?[^\n,]*|diploma[^\n,]*|m\.?tech|bca|mca|b\.?sc|high\s+school|iti)\b").unwrap();
    if let Some(mat) = edu_re.find(raw_text) {
        record.education = mat.as_str().trim().to_string();
    }

    // 7. GCC / Overseas Experience Matching
    let gcc_locations = [
        "Dubai",
        "UAE",
        "United Arab Emirates",
        "Saudi Arabia",
        "Qatar",
        "Oman",
        "Kuwait",
        "Bahrain",
        "Abu Dhabi",
    ];

    let mut gcc_matches = Vec::new();
    let lower_text = raw_text.to_lowercase();

    for &loc in &gcc_locations {
        let loc_lower = loc.to_lowercase();
        if lower_text.contains(&loc_lower) {
            let display_name = match loc {
                "United Arab Emirates" => "UAE",
                other => other,
            };
            if !gcc_matches.contains(&display_name.to_string()) {
                gcc_matches.push(display_name.to_string());
            }
        }
    }

    if !gcc_matches.is_empty() {
        record.overseas_experience = gcc_matches.join(", ");
    }

    record
}