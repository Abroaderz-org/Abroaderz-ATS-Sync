use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub fn infer_candidate_details(text: &str, file_name: &str, jd_text: &str) -> CandidateRecord {
    let clean_text = text.replace('\r', "\n");

    // 1. Candidate Name Extraction
    let name = extract_candidate_name(&clean_text).unwrap_or_else(|| clean_fallback_filename(file_name));

    // 2. Passport Number
    let passport_re = Regex::new(r"(?i)(?:Passport\s*(?:No|Details)?[\s:]*|Pass\s*port\s*No[\s:]*)\s*([A-Z]\s*\d{7,8})").unwrap();
    let passport_no = if let Some(c) = passport_re.captures(&clean_text) {
        c[1].replace(' ', "").to_uppercase()
    } else {
        let general_re = Regex::new(r"\b([A-Z][0-9]{7,8})\b").unwrap();
        general_re.captures(&clean_text)
            .map(|c| c[1].to_uppercase())
            .unwrap_or_else(|| "N/A".to_string())
    };

    // 3. Position / Trade
    let position = extract_position(&clean_text);

    // 4. Education / Degree
    let education = extract_education(&clean_text);

    // 5. Date of Birth
    let dob = extract_dob(&clean_text);

    // 6. Phone Number
    let phone = extract_phone(&clean_text);

    // 7. Email Address (Sanitized against OCR noise)
    let email = extract_email(&clean_text);

    // 8. Experience Breakdown (Numeric Years)
    let (local_experience, overseas_experience) = extract_experience_years(&clean_text);
    let total_experience = local_experience + overseas_experience;

    // 9. Location (State & Country)
    let (state, country) = extract_location(&clean_text);

    // 10. Match Score (Appended only when JD text exists)
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
    // Check explicit name tags first
    let name_tag_re = Regex::new(r"(?im)^\s*(?:Candidate\s*Name|Name)\s*[:\-\.]\s*([A-Za-z\s\.]{3,35})$").unwrap();
    for cap in name_tag_re.captures_iter(text) {
        let n = cap[1].trim();
        let nl = n.to_lowercase();
        if !nl.contains("supervisor") && !nl.contains("foreman") && !nl.contains("engineer") {
            return Some(clean_name_string(n));
        }
    }

    // Inspect top 20 lines of document
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).take(20).collect();
    for line in &lines {
        let l_lower = line.to_lowercase();
        if l_lower.starts_with("father") || l_lower.contains("resume") || l_lower.contains("curriculum") || l_lower.contains("profile") || l_lower.contains("page ") || l_lower.contains("personal") {
            continue;
        }

        // Direct candidate names printed in uppercase/titlecase
        if line.len() >= 3 && line.len() <= 32 && line.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '.') {
            if !l_lower.contains("supervisor") && !l_lower.contains("engineer") && !l_lower.contains("technician") && !l_lower.contains("foreman") && !l_lower.contains("fitter") && !l_lower.contains("diploma") {
                return Some(clean_name_string(line));
            }
        }
    }
    None
}

fn clean_name_string(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn clean_fallback_filename(file_name: &str) -> String {
    let clean = file_name
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF")
        .trim_end_matches(".docx")
        .replace("Copy of ", "")
        .replace(" - RESUME", "")
        .replace("_", " ")
        .trim()
        .to_string();

    if clean.to_lowercase().contains("mechanical") || clean.to_lowercase().contains("supervisor") {
        "Candidate (Check CV)".to_string()
    } else {
        clean
    }
}

fn extract_dob(text: &str) -> String {
    let dob_re = Regex::new(r"(?im)(?:DOB|Date of Birth|Birth Date)\s*[:\-\.]\s*([0-9]{1,2}[\s\-/.][A-Za-z0-9]{2,4}[\s\-/.][0-9]{2,4})").unwrap();
    if let Some(c) = dob_re.captures(text) {
        return c[1].trim().to_string();
    }
    "N/A".to_string()
}

fn extract_phone(text: &str) -> String {
    let phone_re = Regex::new(r"(?:\+?91[\-\s]?)?[6-9]\d{9}").unwrap();
    if let Some(c) = phone_re.captures(text) {
        return c[0].replace(" ", "").replace("-", "");
    }
    "N/A".to_string()
}

fn extract_email(text: &str) -> String {
    let email_re = Regex::new(r"\b[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z]{2,6}\b").unwrap();
    for cap in email_re.captures_iter(text) {
        let em = cap[0].trim_start_matches('-').trim().to_string();
        let em_low = em.to_lowercase();
        // Discard binary compression artifacts
        if em_low.ends_with(".com") || em_low.ends_with(".in") || em_low.ends_with(".org") || em_low.ends_with(".net") {
            if !em_low.contains("zav") && !em_low.contains("qd3") {
                return em;
            }
        }
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
    } else if lower.contains("diploma in electrical") || lower.contains("diploma\n(electrical)") {
        "Diploma in Electrical Engineering".to_string()
    } else if lower.contains("fire & safety") || lower.contains("fire and safety") {
        "Diploma in Fire & Safety Engineering".to_string()
    } else if lower.contains("b.e") || lower.contains("b.tech") {
        "Bachelor of Engineering (B.E)".to_string()
    } else if lower.contains("sslc") {
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