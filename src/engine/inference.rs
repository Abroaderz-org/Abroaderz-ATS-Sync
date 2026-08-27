use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub fn infer_candidate_details(raw_text: &str, file_name: &str) -> CandidateRecord {
    let mut record = CandidateRecord::default();

    // 1. Candidate Name (Cleaned from filename)
    let mut clean_name = file_name
        .trim_end_matches(".docx")
        .trim_end_matches(".doc")
        .trim_end_matches(".pdf")
        .trim_end_matches(".png")
        .trim_end_matches(".jpg")
        .trim_end_matches(".jpeg")
        .trim_end_matches(".webp")
        .replace(['_', '-'], " ");

    let trade_cleaner = Regex::new(r"(?i)\b(civil site engineer|civil engineer|electrical engineer|mechanical engineer|hvac technician|safety officer|site engineer|project coordinator|electrician|plumber|foreman|pipe\s*fitter|welder|mason|carpenter|driver|scaffolder|surveyor|operator|technician|engineer)\b").unwrap();
    clean_name = trade_cleaner.replace_all(&clean_name, "").to_string();
    record.name = clean_name.trim().to_string();

    // 2. Position / Designation
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

    // 3. Email
    let email_re = Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").unwrap();
    if let Some(mat) = email_re.find(raw_text) {
        record.email = mat.as_str().to_lowercase();
    }

    // 4. Phone Number
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
    if record.phone == "None" || record.phone.is_empty() {
        if let Some(mat) = phone_re.find(raw_text) {
            let num = mat.as_str().trim();
            if num.len() >= 8 {
                record.phone = num.to_string();
            }
        }
    }

    // 5. Passport Number (1 Letter + 7 Digits)
    let passport_re = Regex::new(r"(?i)\b(?:passport(?:\s*(?:no|num|number))?[:\s\-]*)?([a-z][0-9]{7})\b").unwrap();
    if let Some(caps) = passport_re.captures(raw_text) {
        if let Some(mat) = caps.get(1) {
            record.passport_no = mat.as_str().to_uppercase();
        }
    }

    // 6. Date of Birth (DOB)
    let dob_re = Regex::new(r"(?i)(?:dob|date\s+of\s+birth|birth\s*date)[:\s]*([0-9]{1,2}[\/\-\.][0-9]{1,2}[\/\-\.][0-9]{2,4}|[0-9]{1,2}\s+[a-z]{3,9}\s+[0-9]{4})").unwrap();
    if let Some(caps) = dob_re.captures(raw_text) {
        if let Some(mat) = caps.get(1) {
            record.dob = mat.as_str().trim().to_string();
        }
    }

    // 7. Education Degree (Cleaned, isolated from universities and dates)
    let edu_re = Regex::new(r"(?i)\b(bachelor\s+of\s+science\s+in\s+[a-z\s]+|bachelor\s+of\s+engineering\s+in\s+[a-z\s]+|b\.?tech(?:\s+in\s+[a-z\s]+)?|b\.?e\.?(?:\s+in\s+[a-z\s]+)?|b\.?sc(?:\s+in\s+[a-z\s]+)?|diploma\s+in\s+[a-z\s]+|diploma|iti|high\s+school|matriculation)\b").unwrap();
    if let Some(mat) = edu_re.find(raw_text) {
        let clean_edu = mat
            .as_str()
            .split(['-', '—', '–', '(', '|', '\n', ':'])
            .next()
            .unwrap_or("")
            .trim();
        if !clean_edu.is_empty() {
            record.education = clean_edu
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
    }

    // 8. Total Experience
    let exp_re = Regex::new(r"(?i)\b(\d{1,2}\+?)\s*(?:years?|yrs?)(?:\s+of)?\s*(?:experience|professional\s+experience|exp|regional\s+experience)\b").unwrap();
    if let Some(caps) = exp_re.captures(raw_text) {
        if let Some(mat) = caps.get(1) {
            record.total_experience = format!("{} Years", mat.as_str());
        }
    }

    // 9. Location Detection (State & Country)
    let loc_re = Regex::new(r"(?i)(?:Location|Address)[:\s]*([A-Za-z\s]+),\s*([A-Z]{2})\b").unwrap();
    if let Some(caps) = loc_re.captures(raw_text) {
        if let Some(city) = caps.get(1) {
            record.state = format!("{}, {}", city.as_str().trim(), caps.get(2).map_or("", |m| m.as_str()));
            record.country = "USA".to_string();
            record.local_experience = "Domestic (USA)".to_string();
        }
    }

    // 10. GCC / Overseas Experience Matching
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
        if record.country == "None" || record.country.is_empty() {
            record.country = "GCC / Overseas".to_string();
        }
    }

    record
}