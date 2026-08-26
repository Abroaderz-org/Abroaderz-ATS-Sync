mod config;
mod engine;
mod export;
mod parser;

use config::AppConfig;
use engine::schema::CandidateRecord;
use export::{csv::export_candidates_to_csv, excel::export_candidates_to_excel};
use parser::{image::extract_image_text, pdf::extract_pdf_text};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn extract_candidate_from_file(path: &Path) -> Result<CandidateRecord, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Candidate")
        .replace('_', " ");

    let raw_text = match ext.as_str() {
        "pdf" => {
            let text = extract_pdf_text(path)?;
            if text.trim().len() < 50 {
                format!("[Scanned PDF detected: {}]", path.display())
            } else {
                text
            }
        }
        "png" | "jpg" | "jpeg" | "webp" => extract_image_text(path)?,
        "txt" => std::fs::read_to_string(path).map_err(|e| e.to_string())?,
        _ => return Err(format!("Unsupported file extension: .{}", ext)),
    };

    let mut candidate = CandidateRecord {
        name: file_name,
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

    if let Some(email) = find_regex_match(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", &raw_text) {
        candidate.email = email;
    }

    if let Some(phone) = find_regex_match(r"(\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}", &raw_text) {
        candidate.phone = phone;
    }

    Ok(candidate)
}

fn find_regex_match(pattern: &str, text: &str) -> Option<String> {
    regex::Regex::new(pattern)
        .ok()?
        .find(text)
        .map(|m| m.as_str().to_string())
}

fn collect_resume_paths<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let supported = ["pdf", "png", "jpg", "jpeg", "webp", "txt"];
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| supported.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect()
}

fn main() {
    println!("💼 Abroaderz ATS Sync — High-Performance Resume Extractor");
    println!("----------------------------------------------------------");

    let cfg = AppConfig::load_or_create("config.toml");

    let target_path = Path::new(&cfg.input_directory);
    if !target_path.exists() {
        println!("📁 Creating input directory: {}", cfg.input_directory);
        let _ = std::fs::create_dir_all(target_path);
        println!("👉 Place resume files into '{}' and rerun the command.", cfg.input_directory);
        return;
    }

    let files = collect_resume_paths(target_path);
    if files.is_empty() {
        println!("⚠️  No compatible resumes found in '{}'.", cfg.input_directory);
        println!("Supported formats: .pdf, .png, .jpg, .jpeg, .webp, .txt");
        return;
    }

    let process_count = files.len().min(cfg.batch_limit);
    println!("⚡ Found {} resume(s). Processing batch of {}...", files.len(), process_count);

    let mut candidates: Vec<CandidateRecord> = Vec::new();

    for (idx, file_path) in files.iter().take(process_count).enumerate() {
        println!("  [{}/{}] Parsing: {}", idx + 1, process_count, file_path.display());
        match extract_candidate_from_file(file_path) {
            Ok(record) => candidates.push(record),
            Err(err) => eprintln!("    ❌ Failed to process {}: {}", file_path.display(), err),
        }
    }

    if candidates.is_empty() {
        println!("❌ No candidate records were extracted.");
        return;
    }

    println!("📊 Exporting Excel report to '{}'...", cfg.excel_output_path);
    if let Err(e) = export_candidates_to_excel(&candidates, &cfg.excel_output_path) {
        eprintln!("❌ Excel export error: {}", e);
    } else {
        println!("✅ Excel export completed.");
    }

    println!("📄 Exporting CSV report to '{}'...", cfg.csv_output_path);
    if let Err(e) = export_candidates_to_csv(&candidates, &cfg.csv_output_path) {
        eprintln!("❌ CSV export error: {}", e);
    } else {
        println!("✅ CSV export completed.");
    }

    println!("🎉 Batch run complete! Processed {} records.", candidates.len());
}