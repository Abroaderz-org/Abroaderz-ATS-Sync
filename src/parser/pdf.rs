use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    // 1. Primary extractor: pdf-extract (Handles raw text streams)
    if let Ok(text) = pdf_extract::extract_text(path) {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    // 2. Secondary fallback: lopdf (Strictly limited to the first 4 pages)
    if let Ok(doc) = lopdf::Document::load(path) {
        let pages = doc.get_pages();
        let max_pages = 4.min(pages.len());
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

    Err("No readable text stream in PDF header".to_string())
}