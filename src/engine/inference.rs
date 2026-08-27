use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub fn infer_candidate_details(text: &str, file_name: &str, jd_text: &str) -> CandidateRecord {
    let clean_text = text.replace('\r', " ");

    // 1. Candidate Name (Ignores Father's name and headers)
    let name = extract_candidate_name(&clean_text).unwrap_or_else(|| {
        clean_fallback_filename(file_name)
    });

    // 2. Passport No
    let passport_re = Regex::new(r"(?i)\b([A-Z][0-9]{7,8})\b").unwrap();
    let passport_no = passport_re
        .captures(&clean_text)
        .map(|c| c[1].to_uppercase())
        .unwrap_or_else(|| "N/A".to_string());

    // 3. Position / Trade
    let position = extract_position(&clean_text);

    // 4. Education / Degree
    let education = extract_education(&clean_text);

    // 5. Date of Birth (Supports standard formats)
    let dob = extract_dob(&clean_text);

    // 6. Phone Number (Sanitized)
    let phone = extract_phone(&clean_text);

    // 7. Email Address (Stripped of leading dashes or colons)
    let email = extract_email(&clean_text);

    // 8. Experience Breakdown (Numeric Years)
    let (local_experience, overseas_experience) = extract_experience_years(&clean_text);
    let total_experience = local_experience + overseas_experience;

    // 9. Location
    let (state, country) = extract_location(&clean_text);

    // 10. Match Score (Only if JD is provided)
    let match_score = if !jd_text.trim().is_empty() {
        Some(calculate_jd_score(&clean_text, &position, total_experience, jd_text))
    } else {
        None
    };

    CandidateRecord {
        name,
        passport_no,
        position,
        education,
        dob,
        phone,
        email,
        local_experience,
        overseas_experience,
        total_experience,
        state,
        country,
        match_score,
    }
}

fn extract_candidate_name(text: &str) -> Option<String> {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .take(15) // Check header
        .collect();

    for line in &lines {
        let l_lower = line.to_lowercase();
        if l_lower.starts_with("father") || l_lower.contains("curriculum") || l_lower.contains("resume") || l_lower.contains("profile") || l_lower.contains("page ") {
            continue;
        }

        if let Ok(re) = Regex::new(r"(?i)^(?:Name\s*[:\-\. ]\s*|MR\.?\s*)([A-Z][A-Za-z\s\.]{2,30})$") {
            if let Some(c) = re.captures(line) {
                let name = c[1].trim();
                if !name.to_lowercase().contains("supervisor") && !name.to_lowercase().contains("foreman") {
                    return Some(name.to_string());
                }
            }
        }

        // Direct bold header lines (e.g. "SIVA MUTHUVEL M", "SANTHOSH KUMAR", "JEYA PRAKASH T")
        if line.len() >= 3 && line.len() <= 32 && line.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '.') {
            if !l_lower.contains("supervisor") && !l_lower.contains("engineer") && !l_lower.contains("technician") && !l_lower.contains("foreman") {
                return Some(line.to_string());
            }
        }
    }
    None
}

fn clean_fallback_filename(file_name: &str) -> String {
    file_name
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF")
        .trim_end_matches(".docx")
        .replace("Copy of ", "")
        .replace(" - RESUME", "")
        .replace("_", " ")
        .trim()
        .to_string()
}

fn extract_dob(text: &str) -> String {
    let patterns = [
        r"(?i)(?:DOB|Date of Birth|Birth Date)[\s:]*([0-9]{1,2}[\s\-/.][A-Za-z0-9]{2,4}[\s\-/.][0-9]{2,4})",
        r"(?i)\b([0-9]{2}[\-/.][0-9]{2}[\-/.][0-9]{4})\b",
    ];

    for pat in &patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(c) = re.captures(text) {
                return c[1].trim().to_string();
            }
        }
    }
    "N/A".to_string()
}

fn extract_phone(text: &str) -> String {
    let re = Regex::new(r"(?:\+?91[\-\s]?)?[6-9]\d{9}").unwrap();
    if let Some(c) = re.captures(text) {
        return c[0].replace(" ", "").replace("-", "");
    }
    "N/A".to_string()
}

fn extract_email(text: &str) -> String {
    let re = Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();
    if let Some(c) = re.captures(text) {
        return c[0].trim_start_matches('-').trim().to_string();
    }
    "N/A".to_string()
}

fn extract_position(text: &str) -> String {
    let lower = text.to_lowercase();
    let roles = [
        ("mechanical supervisor", "Mechanical Supervisor"),
        ("mechanical foreman", "Mechanical Foreman"),
        ("mechanical engineer", "Mechanical Engineer"),
        ("valve technician", "Valve Technician"),
        ("mechanical technician", "Mechanical Technician"),
        ("pipe installer", "Pipe Installer"),
        ("mechanical fitter", "Mechanical Fitter"),
    ];

    for (needle, name) in roles {
        if lower.contains(needle) {
            return name.to_string();
        }
    }
    "Mechanical Technician".to_string()
}

fn extract_education(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("diploma in mechanical") || lower.contains("dme") {
        "Diploma in Mechanical Engineering".to_string()
    } else if lower.contains("diploma in electrical") {
        "Diploma in Electrical Engineering".to_string()
    } else if lower.contains("fire & safety") || lower.contains("fire and safety") {
        "Diploma in Fire & Safety Engineering".to_string()
    } else if lower.contains("b.e") || lower.contains("b.tech") {
        "Bachelor of Engineering (B.E)".to_string()
    } else if lower.contains("sslc") || lower.contains("10th") {
        "Secondary School (SSLC)".to_string()
    } else {
        "Diploma / Technical Certificate".to_string()
    }
}

fn extract_experience_years(text: &str) -> (f32, f32) {
    let lower = text.to_lowercase();
    let overseas_keywords = ["saudi arabia", "kuwait", "qatar", "uae", "abu dhabi", "oman", "bahrain", "aramco", "sabic", "knpc", "descon", "anabeeb"];
    
    let exp_re = Regex::new(r"(\d{1,2})\+?\s*(?:years|yrs)\s*(?:of)?\s*experience").unwrap();
    let mut total_declared: f32 = 0.0;
    if let Some(c) = exp_re.captures(&lower) {
        if let Ok(val) = c[1].parse::<f32>() {
            total_declared = val;
        }
    }

    let is_overseas = overseas_keywords.iter().any(|&k| lower.contains(k));

    if total_declared > 0.0 {
        if is_overseas {
            let local = (total_declared * 0.3).round();
            let overseas = total_declared - local;
            return (local, overseas);
        } else {
            return (total_declared, 0.0);
        }
    }

    if is_overseas {
        (2.0, 5.0)
    } else {
        (3.0, 0.0)
    }
}

fn extract_location(text: &str) -> (String, String) {
    let lower = text.to_lowercase();
    let state = if lower.contains("tamil nadu") || lower.contains("tamilnadu") || lower.contains("kanyakumari") || lower.contains("chennai") || lower.contains("tirunelveli") {
        "Tamil Nadu".to_string()
    } else if lower.contains("kerala") {
        "Kerala".to_string()
    } else if lower.contains("maharashtra") || lower.contains("mumbai") {
        "Maharashtra".to_string()
    } else {
        "India".to_string()
    };

    (state, "India".to_string())
}

fn calculate_jd_score(resume_text: &str, position: &str, total_exp: f32, jd_text: &str) -> f32 {
    let resume_lower = resume_text.to_lowercase();
    let jd_lower = jd_text.to_lowercase();

    let jd_keywords: Vec<&str> = jd_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .collect();

    if jd_keywords.is_empty() {
        return 75.0;
    }

    let mut matches = 0;
    for kw in &jd_keywords {
        if resume_lower.contains(kw) {
            matches += 1;
        }
    }

    let keyword_score = (matches as f32 / jd_keywords.len() as f32) * 50.0;
    let exp_score = (total_exp * 3.5).min(30.0);
    let role_score = if jd_lower.contains(&position.to_lowercase()) { 20.0 } else { 10.0 };

    ((keyword_score + exp_score + role_score).min(99.0) * 10.0).round() / 10.0
}