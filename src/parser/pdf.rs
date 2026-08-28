use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // 1. Primary engine: lopdf page-by-page (CVs are ALWAYS pages 1-3)
    // By extracting only text streams from pages 1-3, we ignore 20+ pages of heavy scanned certificates!
    if let Ok(doc) = lopdf::Document::load_mem(&buffer) {
        let pages = doc.get_pages();
        let mut page_keys: Vec<u32> = pages.keys().cloned().collect();
        page_keys.sort_unstable();

        let mut acc = String::new();
        for page_num in page_keys.into_iter().take(3) {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                acc.push_str(&text);
                acc.push('\n');
            }
        }
        let clean = acc.trim();
        if clean.len() > 30 {
            return Ok(clean.to_string());
        }
    }

    // 2. Secondary fallback: pdf-extract
    if let Ok(text) = pdf_extract::extract_text_from_mem(&buffer) {
        let clean = text.trim();
        if clean.len() > 30 {
            return Ok(clean.to_string());
        }
    }

    Err("No extractable resume text found in first 3 pages".to_string())
}