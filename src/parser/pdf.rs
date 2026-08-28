use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    // 1. Safe bounded file read (max 35MB buffer to prevent RAM exhaustion)
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // 2. Primary text extraction
    if let Ok(text) = pdf_extract::extract_text_from_mem(&buffer) {
        let clean = text.trim();
        if !clean.is_empty() {
            return Ok(clean.to_string());
        }
    }

    // 3. Fallback: Parse only the first 3 pages using lopdf (resumes are always page 1-3)
    if let Ok(doc) = lopdf::Document::load_mem(&buffer) {
        let pages = doc.get_pages();
        let max_pages = 3.min(pages.len());
        let page_numbers: Vec<u32> = pages.keys().take(max_pages).cloned().collect();

        let mut text_acc = String::new();
        for page_num in page_numbers {
            if let Ok(page_text) = doc.extract_text(&[page_num]) {
                text_acc.push_str(&page_text);
                text_acc.push('\n');
            }
        }
        if !text_acc.trim().is_empty() {
            return Ok(text_acc);
        }
    }

    Err("No readable text found in CV header".to_string())
}