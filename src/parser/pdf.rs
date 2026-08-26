use std::path::Path;

pub fn extract_pdf_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    pdf_extract::extract_text(path).map_err(|e| format!("PDF extraction error: {}", e))
}
