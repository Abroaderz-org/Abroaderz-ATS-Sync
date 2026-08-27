use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::path::Path;

pub fn extract_image_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let img = image::open(path.as_ref())
        .map_err(|e| format!("Failed to open image: {}", e))?
        .into_rgb8();

    let (width, height) = img.dimensions();
    let img_source = ImageSource::from_bytes(img.as_raw(), (width, height))
        .map_err(|e| format!("Failed to create ImageSource: {}", e))?;

    let detection_model_bytes = include_bytes!("../../models/text-detection.rten");
    let rec_model_bytes = include_bytes!("../../models/text-recognition.rten");

    let detection_model = Model::load(detection_model_bytes.to_vec())
        .map_err(|e| format!("Failed to load text detection model: {}", e))?;
    let recognition_model = Model::load(rec_model_bytes.to_vec())
        .map_err(|e| format!("Failed to load text recognition model: {}", e))?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    }).map_err(|e| format!("Failed to initialize OCR engine: {}", e))?;

    let ocr_input = engine.prepare_input(img_source)
        .map_err(|e| format!("OCR preparation failed: {}", e))?;
    
    let word_rects = engine.detect_words(&ocr_input)
        .map_err(|e| format!("Word detection failed: {}", e))?;
        
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let texts = engine.recognize_text(&ocr_input, &line_rects)
        .map_err(|e| format!("Text recognition failed: {}", e))?;

    let full_text = texts
        .into_iter()
        .filter_map(|line| line.map(|l| l.to_string()))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(full_text)
}