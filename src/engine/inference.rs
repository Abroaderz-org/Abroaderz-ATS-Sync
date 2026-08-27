use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub fn infer_candidate_details(text: &str, file_name: &str, jd_text: &str) -> CandidateRecord {
    let clean_text = text.replace('\r', " ");

    // 1. Candidate Name Extraction
    let name = extract_name(&clean_text).unwrap_or_else(|| {
        file_name
            .trim_end_matches(".pdf")
            .trim_end_matches(".docx")
            .replace('_', " ")
            .replace('-', " ")
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

    // 5. Date of Birth
    let dob_re = Regex::new(r"(?i)(?:DOB|Date of Birth|Birth Date)[\s:]*([0-9]{1,2}[-/.][0-9]{1,2}[-/.][0-9]{2,4})").unwrap();
    let dob = dob_re
        .captures(&clean_text)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "N/A".to_string());

    // 6. Phone Number
    let phone_re = Regex::new(r"(?:\+?91[\-\s]?)?[6-9]\d{9}").unwrap();
    let phone = phone_re
        .captures(&clean_text)
        .map(|c| c[0].to_string())
        .unwrap_or_else(|| "N/A".to_string());

    // 7. Email Address
    let email_re = Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();
    let email = email_re
        .captures(&clean_text)
        .map(|c| c[0].to_string())
        .unwrap_or_else(|| "N/A".to_string());

    // 8. Experience Breakdown (Years)
    let (local_exp_years, overseas_exp_years) = extract_experience_years(&clean_text);
    let total_exp_years = local_exp_years + overseas_exp_years;

    // 9. Location
    let (state, country) = extract_location(&clean_text);

    // 10. Compute Match Score vs Job Description
    let match_score = calculate_jd_score(&clean_text, &position, total_exp_years, jd_text);

    CandidateRecord {
        name,
        passport_no,
        position,
        education,
        dob,
        phone,
        email,
        local_exp_years,
        overseas_exp_years,
        total_exp_years,
        state,
        country,
        match_score,
    }
}

fn extract_name(text: &str) -> Option<String> {
    let name_patterns = [
        r"(?i)(?:Name\s*[:\-\. ]\s*|Candidate\s*Name\s*[:\-\. ]\s*|This is to certify that Mr\.?\s*)([A-Z][A-Z\s\.]{2,35})",
        r"(?i)(?:RESUME|CURRICULUM VITAE)\s*\n+([A-Z][A-Z\s\.]{2,30})",
    ];

    for pat in &name_patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(caps) = re.captures(text) {
                let found = caps[1].trim();
                if found.split_whitespace().count() >= 1 && !found.to_lowercase().contains("supervisor") {
                    return Some(found.to_string());
                }
            }
        }
    }
    None
}

fn extract_position(text: &str) -> String {
    let roles = [
        "Mechanical Supervisor",
        "Mechanical Foreman",
        "Mechanical Engineer",
        "Mechanical Technician",
        "Confined Space Supervisor",
        "Valve Technician",
        "Pipe Installer",
        "Mechanical Fitter",
    ];

    for role in roles {
        if text.to_lowercase().contains(&role.to_lowercase()) {
            return role.to_string();
        }
    }
    "Technician / Specialist".to_string()
}

fn extract_education(text: &str) -> String {
    let degrees = [
        ("Diploma in Mechanical Engineering", "Diploma in Mechanical Engineering"),
        ("Diploma in Electrical", "Diploma in Electrical & Electronics"),
        ("Diploma in Fire & Safety", "Diploma in Fire & Safety"),
        ("DME", "Diploma in Mechanical Engineering (DME)"),
        ("B.E. Mechanical", "B.E. Mechanical Engineering"),
        ("B.Tech", "B.Tech Engineering"),
        ("SSLC", "Secondary School (SSLC)"),
    ];

    for (needle, val) in degrees {
        if text.to_lowercase().contains(&needle.to_lowercase()) {
            return val.to_string();
        }
    }
    "Technical Certification / Diploma".to_string()
}

fn extract_experience_years(text: &str) -> (f32, f32) {
    let lower = text.to_lowercase();
    let overseas_keywords = ["saudi arabia", "kuwait", "qatar", "uae", "abu dhabi", "oman", "bahrain", "aramco", "sabic", "knpc", "descon", "anabeeb"];
    
    // Look for explicit "X+ years experience" claims
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

    // Default heuristics based on certifications count
    if is_overseas {
        (2.0, 5.0)
    } else {
        (3.0, 0.0)
    }
}

fn extract_location(text: &str) -> (String, String) {
    let lower = text.to_lowercase();
    let state = if lower.contains("tamil nadu") || lower.contains("tamilnadu") || lower.contains("kanyakumari") || lower.contains("chennai") {
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
    if jd_text.trim().is_empty() {
        // Base score calculated from experience and role
        return (60.0 + (total_exp * 3.5)).min(98.0);
    }

    let resume_lower = resume_text.to_lowercase();
    let jd_lower = jd_text.to_lowercase();

    let jd_keywords: Vec<&str> = jd_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
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
    let exp_score = (total_exp * 4.0).min(30.0);
    let role_score = if jd_lower.contains(&position.to_lowercase()) { 20.0 } else { 10.0 };

    ((keyword_score + exp_score + role_score) * 10.0).round() / 10.0
}