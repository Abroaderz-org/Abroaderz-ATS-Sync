use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // 1. First attempt: standard pdf-extract
    if let Ok(text) = pdf_extract::extract_text_from_mem(&buffer) {
        let clean = text.trim();
        if !clean.is_empty() {
            return Ok(clean.to_string());
        }
    }

    // 2. Second attempt: lopdf across all pages
    if let Ok(doc) = lopdf::Document::load_mem(&buffer) {
        let pages = doc.get_pages();
        let page_numbers: Vec<u32> = pages.keys().cloned().collect();

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

    // 3. Fallback: Raw byte search for ASCII text streams
    let mut ascii_text = String::new();
    let mut temp = String::new();
    for &b in &buffer {
        if (32..=126).contains(&b) || b == b'\n' || b == b'\r' || b == b'\t' {
            temp.push(b as char);
        } else {
            if temp.len() > 3 {
                ascii_text.push_str(&temp);
                ascii_text.push(' ');
            }
            temp.clear();
        }
    }
    if !temp.is_empty() {
        ascii_text.push_str(&temp);
    }

    if !ascii_text.trim().is_empty() {
        return Ok(ascii_text);
    }

    Err("No text extractable from PDF document.".to_string())
}