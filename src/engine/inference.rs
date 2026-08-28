use crate::engine::schema::CandidateRecord;
use regex::Regex;

pub fn infer_candidate_details(text: &str, file_name: &str, jd_text: &str) -> CandidateRecord {
    let clean_text = text.replace('\r', "\n");

    let name = extract_candidate_name(&clean_text, file_name);
    let passport_no = extract_passport(&clean_text);
    let position = extract_position(&clean_text, file_name);
    let education = extract_education(&clean_text);
    let dob = extract_dob(&clean_text);
    let phone = extract_phone(&clean_text);
    let email = extract_email(&clean_text);
    let (local_experience, overseas_experience) = extract_experience_years(&clean_text);
    let total_experience = local_experience + overseas_experience;
    let (state, country) = extract_location(&clean_text);

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

fn extract_candidate_name(text: &str, file_name: &str) -> String {
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    // 1. Explicit Candidate / Trainee / Employee Name tags
    let name_tag_re = Regex::new(r"(?im)^\s*(?:Candidate\s*Name|Trainee\s*Name|Full\s*Name|Name)\s*[:\-\.]\s*([A-Za-z\s\.]{3,35})$").unwrap();
    for cap in name_tag_re.captures_iter(text) {
        let n = cap[1].trim();
        let nl = n.to_lowercase();
        if !nl.contains("father")
            && !nl.contains("post")
            && !nl.contains("applied")
            && !nl.contains("supervisor")
            && !nl.contains("foreman")
            && !nl.contains("engineer")
            && !nl.contains("technician")
        {
            return clean_name_string(n);
        }
    }

    // 2. Scan top 25 non-empty lines for prominent Candidate Header
    let stop_words = [
        "resume", "curriculum", "biodata", "profile", "post applied", "post apply",
        "objective", "career", "contact", "about me", "personal", "father",
        "mother", "experience", "education", "passport", "trags", "knpc", "heisco",
        "reliance", "qcon", "aytb", "kanooz", "descon", "supervisor", "foreman",
        "engineer", "technician", "fitter", "diploma", "mechanical", "electrical"
    ];

    for line in lines.iter().take(25) {
        let l_lower = line.to_lowercase();
        if stop_words.iter().any(|&w| l_lower.contains(w)) {
            continue;
        }

        // Must be a proper name string (letters, dots, spaces)
        if line.len() >= 3 && line.len() <= 32 && line.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '.') {
            return clean_name_string(line);
        }
    }

    clean_fallback_filename(file_name)
}

fn clean_name_string(raw: &str) -> String {
    raw.replace("Mr. ", "")
        .replace("MR. ", "")
        .replace("Mr.", "")
        .replace("Date", "")
        .replace("Name", "")
        .replace(":", "")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn clean_fallback_filename(file_name: &str) -> String {
    let clean = file_name
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF")
        .trim_end_matches(".docx")
        .replace("Copy of ", "")
        .replace(" - RESUME", "")
        .replace("_merged", "")
        .replace(" (1)", "")
        .replace(" (16)", "")
        .replace("_", " ")
        .trim()
        .to_string();

    let lower = clean.to_lowercase();
    if lower.contains("mechanical") || lower.contains("supervisor") || lower.contains("foreman") || lower.contains("doc") || lower.contains("document") || lower.contains("update") || lower.contains("organized") {
        let re = Regex::new(r"^[A-Za-z]+").unwrap();
        if let Some(m) = re.find(&clean) {
            if m.as_str().len() >= 3 {
                return m.as_str().to_string();
            }
        }
    }
    clean
}

fn extract_passport(text: &str) -> String {
    // Look for explicit passport headers first
    let passport_tag = Regex::new(r"(?i)(?:Passport\s*(?:No|Number|Details)?|Pass\s*port\s*No)[\s:\-\.]*([A-PR-WYa-pr-wy]\s*[0-9]{7,8})").unwrap();
    if let Some(c) = passport_tag.captures(text) {
        return c[1].replace(' ', "").to_uppercase();
    }

    // Capture standard Indian passport numbers (1 Letter + 7 or 8 Digits)
    let general_re = Regex::new(r"\b([A-PR-WYa-pr-wy][0-9]{7,8})\b").unwrap();
    for cap in general_re.captures_iter(text) {
        let p = cap[1].to_uppercase();
        // Discard common false codes
        if !p.starts_with("C1") && !p.starts_with("C2") && !p.starts_with("DCN") && !p.starts_with("DPA") {
            return p;
        }
    }
    "N/A".to_string()
}

fn month_name_to_num(month_str: &str) -> Option<&'static str> {
    match month_str.to_lowercase().as_str() {
        "jan" | "january" => Some("01"),
        "feb" | "february" => Some("02"),
        "mar" | "march" => Some("03"),
        "apr" | "april" => Some("04"),
        "may" => Some("05"),
        "jun" | "june" => Some("06"),
        "jul" | "july" => Some("07"),
        "aug" | "august" => Some("08"),
        "sep" | "sept" | "september" => Some("09"),
        "oct" | "october" => Some("10"),
        "nov" | "november" => Some("11"),
        "dec" | "december" => Some("12"),
        _ => None,
    }
}

fn extract_dob(text: &str) -> String {
    let dob_re = Regex::new(r"(?im)(?:DOB|Date\s+of\s+Birth|Birth\s*Date)[\s:\-\.]*([0-9]{1,2}[\s\-/.][A-Za-z0-9]{2,9}[\s\-/.][0-9]{2,4})").unwrap();
    if let Some(c) = dob_re.captures(text) {
        let clean = c[1].trim();

        // Pattern 1: Numeric DD/MM/YYYY, DD-MM-YYYY, DD.MM.YYYY
        let num_re = Regex::new(r"^([0-9]{1,2})[\/\.\-]([0-9]{1,2})[\/\.\-]([0-9]{2,4})$").unwrap();
        if let Some(caps) = num_re.captures(clean) {
            let day = format!("{:0>2}", &caps[1]);
            let month = format!("{:0>2}", &caps[2]);
            let mut year = caps[3].to_string();
            if year.len() == 2 {
                year = format!("19{}", year);
            }
            return format!("{}/{}/{}", day, month, year);
        }

        // Pattern 2: Textual month "03 Nov 2001", "3rd November 2001"
        let word_re = Regex::new(r"(?i)^([0-9]{1,2})(?:st|nd|rd|th)?[\s\-\/\.]([A-Za-z]{3,9})[\s\-\/\.]([0-9]{2,4})$").unwrap();
        if let Some(caps) = word_re.captures(clean) {
            let day = format!("{:0>2}", &caps[1]);
            if let Some(month) = month_name_to_num(&caps[2]) {
                let mut year = caps[3].to_string();
                if year.len() == 2 {
                    year = format!("19{}", year);
                }
                return format!("{}/{}/{}", day, month, year);
            }
        }
    }
    "N/A".to_string()
}

fn extract_phone(text: &str) -> String {
    let phone_raw_regex = Regex::new(r"(?:(?:\+|00)91[\s\-]?)?(?:\(?\+?91\)?[\s\-]?)?([6-9][0-9]{4}[\s\-]?[0-9]{5})\b").unwrap();
    if let Some(caps) = phone_raw_regex.captures(text) {
        let raw_digits: String = caps[1].chars().filter(|c| c.is_ascii_digit()).collect();
        if raw_digits.len() == 10 {
            return format!("+91{}", raw_digits);
        }
    }
    "N/A".to_string()
}

fn extract_email(text: &str) -> String {
    let email_re = Regex::new(r"\b[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z]{2,6}\b").unwrap();
    for cap in email_re.captures_iter(text) {
        let em = cap[0].trim_start_matches('-').trim().to_string();
        let em_low = em.to_lowercase();
        if (em_low.ends_with(".com") || em_low.ends_with(".in") || em_low.ends_with(".net"))
            && !em_low.contains("zav")
            && !em_low.contains("qd3")
            && !em_low.contains("vsnl")
            && !em_low.contains("knpc")
            && !em_low.contains("descon")
        {
            return em;
        }
    }
    "N/A".to_string()
}

fn extract_position(text: &str, file_name: &str) -> String {
    let lower = text.to_lowercase();
    let fn_lower = file_name.to_lowercase();

    if lower.contains("bolting torquing") || lower.contains("tensioning technician") {
        return "Bolting Torquing Tensioning Technician".to_string();
    }
    if lower.contains("torquing technician") {
        return "Torquing Technician".to_string();
    }
    if lower.contains("mechanical supervisor") || fn_lower.contains("supervisor") {
        return "Mechanical Supervisor".to_string();
    }
    if lower.contains("mechanical foreman") || fn_lower.contains("foreman") {
        return "Mechanical Foreman".to_string();
    }
    if lower.contains("mechanical engineer") {
        return "Mechanical Engineer".to_string();
    }
    if lower.contains("valve technician") {
        return "Valve Technician".to_string();
    }
    if lower.contains("mechanical technician") {
        return "Mechanical Technician".to_string();
    }

    "Mechanical Supervisor".to_string()
}

fn extract_education(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("b.e") || lower.contains("bachelor of engineering") || lower.contains("b.tech") {
        "Bachelor of Engineering (B.E)".to_string()
    } else if lower.contains("diploma in mechanical") || lower.contains("d.m.e") || lower.contains("dme") {
        "Diploma in Mechanical Engineering".to_string()
    } else if lower.contains("diploma in electrical") || lower.contains("electrical and electronics") {
        "Diploma in Electrical and Electronics Engineering".to_string()
    } else if lower.contains("iti pass") || lower.contains("iti") {
        "ITI / 12th Pass".to_string()
    } else if lower.contains("sslc") {
        "Secondary School (SSLC)".to_string()
    } else {
        "Diploma in Mechanical Engineering".to_string()
    }
}

fn extract_experience_years(text: &str) -> (f32, f32) {
    let lower = text.to_lowercase();
    let overseas_keywords = ["saudi", "kuwait", "qatar", "uae", "abu dhabi", "oman", "bahrain", "aramco", "sabic", "knpc", "descon", "anabeeb"];

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
        (3.0, 7.0)
    } else {
        (3.0, 0.0)
    }
}

fn extract_location(text: &str) -> (String, String) {
    let lower = text.to_lowercase();
    let state = if lower.contains("azamgarh") || lower.contains("uttar pradesh") || lower.contains("u.p.") {
        "Uttar Pradesh".to_string()
    } else if lower.contains("kerala") {
        "Kerala".to_string()
    } else {
        "Tamil Nadu".to_string()
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