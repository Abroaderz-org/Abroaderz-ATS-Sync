use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    // 1. Primary extractor: pdf-extract (Fast, stream-based, low RAM footprint)
    if let Ok(text) = pdf_extract::extract_text(path) {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    // 2. Secondary fallback: lopdf (Bounded to first 4 pages)
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

    // 3. Raw byte stream fallback for binary PDFs
    if let Ok(mut file) = File::open(path) {
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            let latin_str: String = buffer
                .into_iter()
                .filter_map(|b| if b.is_ascii_graphic() || b == b' ' || b == b'\n' { Some(b as char) } else { None })
                .collect();
            if latin_str.len() > 100 {
                return Ok(latin_str);
            }
        }
    }

    Err("Unable to parse text stream from PDF".to_string())
}