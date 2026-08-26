use image::GenericImageView;
use std::path::Path;

pub fn load_image_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, String> {
    let path_ref = path.as_ref();
    
    let img = image::open(path_ref)
        .map_err(|e| format!("Failed to read image {:?}: {}", path_ref.file_name(), e))?;

    let (width, height) = img.dimensions();
    if width == 0 || height == 0 {
        return Err("Image contains invalid zero dimensions.".to_string());
    }

    std::fs::read(path_ref)
        .map_err(|e| format!("Failed to load raw bytes: {}", e))
}

pub fn extract_image_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let _bytes = load_image_bytes(&path)?;
    
    Ok(format!(
        "[Image Resume Detected: {}]",
        path.as_ref().display()
    ))
}