use docx_rs::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_docx_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let mut file = File::open(path.as_ref()).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

    let docx = read_docx(&buf).map_err(|e| format!("DOCX parse error: {:?}", e))?;
    let mut text_lines = Vec::new();

    for child in docx.document.children {
        if let DocumentChild::Paragraph(p) = child {
            let mut paragraph_text = String::new();
            for run_child in p.children {
                if let ParagraphChild::Run(r) = run_child {
                    for r_child in r.children {
                        if let RunChild::Text(t) = r_child {
                            paragraph_text.push_str(&t.text);
                        }
                    }
                }
            }
            if !paragraph_text.trim().is_empty() {
                text_lines.push(paragraph_text);
            }
        }
    }

    Ok(text_lines.join("\n"))
}