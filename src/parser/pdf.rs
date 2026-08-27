use std::path::Path;

pub fn extract_pdf_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let path_ref = path.as_ref();
    
    match pdf_extract::extract_text(path_ref) {
        Ok(text) if !text.trim().is_empty() => Ok(text),
        _ => Err(format!(
            "PDF contains no selectable text (scanned document): {}",
            path_ref.display()
        )),
    }
}