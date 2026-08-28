use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // 1. Primary Engine: lopdf across all pages of the document
    if let Ok(doc) = lopdf::Document::load_mem(&buffer) {
        let pages = doc.get_pages();
        let mut page_keys: Vec<u32> = pages.keys().cloned().collect();
        page_keys.sort_unstable();

        let mut acc = String::new();
        for page_num in page_keys {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                acc.push_str(&text);
                acc.push('\n');
            }
        }
        let clean = acc.trim();
        if clean.len() > 20 {
            return Ok(clean.to_string());
        }
    }

    // 2. Secondary Engine: pdf-extract
    if let Ok(text) = pdf_extract::extract_text_from_mem(&buffer) {
        let clean = text.trim();
        if clean.len() > 20 {
            return Ok(clean.to_string());
        }
    }

    Err("Could not extract readable text streams from PDF.".to_string())
}