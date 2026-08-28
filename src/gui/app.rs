use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke, Vec2,
};
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::engine::inference::infer_candidate_details;
use crate::export::{csv::export_candidates_to_csv, excel::export_candidates_to_excel};
use crate::parser::{docx::extract_docx_text, pdf::extract_pdf_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Both,
}

fn extract_file_content(path: &Path) -> Option<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "pdf" => extract_pdf_text(path).ok(),
        "docx" => extract_docx_text(path).ok(),
        "txt" => fs::read_to_string(path).ok(),
        _ => None,
    }
}

fn process_directory(
    dir_path: &Path,
    format: ExportFormat,
    jd_file_path: Option<&PathBuf>,
) -> Result<(usize, Option<String>, Option<String>), String> {
    let jd_text = jd_file_path
        .and_then(|p| extract_file_content(p))
        .unwrap_or_default();

    let mut candidates = Vec::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let jd_clone = jd_text.clone();

            let parsed_candidate = panic::catch_unwind(move || {
                let raw_text = extract_file_content(&path);
                raw_text.map(|text| infer_candidate_details(&text, &file_name, &jd_clone))
            });

            if let Ok(Some(candidate)) = parsed_candidate {
                candidates.push(candidate);
            }
        }
    }

    if candidates.is_empty() {
        return Err("No valid PDF or DOCX resumes found in chosen directory.".to_string());
    }

    if !jd_text.trim().is_empty() {
        candidates.sort_by(|a, b| {
            let sa = a.match_score.unwrap_or(0.0);
            let sb = b.match_score.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let count = candidates.len();
    let mut generated_csv = None;
    let mut generated_excel = None;

    match format {
        ExportFormat::Excel => {
            let excel_path = "Abroaderz_Candidates.xlsx".to_string();
            export_candidates_to_excel(&candidates, &excel_path)
                .map_err(|e| format!("Excel export failed: {}", e))?;
            generated_excel = Some(excel_path);
        }
        ExportFormat::Csv => {
            let csv_path = "Abroaderz_Candidates.csv".to_string();
            export_candidates_to_csv(&candidates, &csv_path)
                .map_err(|e| format!("CSV export failed: {}", e))?;
            generated_csv = Some(csv_path);
        }
        ExportFormat::Both => {
            let excel_path = "Abroaderz_Candidates.xlsx".to_string();
            let csv_path = "Abroaderz_Candidates.csv".to_string();

            export_candidates_to_excel(&candidates, &excel_path)
                .map_err(|e| format!("Excel export failed: {}", e))?;
            export_candidates_to_csv(&candidates, &csv_path)
                .map_err(|e| format!("CSV export failed: {}", e))?;

            generated_excel = Some(excel_path);
            generated_csv = Some(csv_path);
        }
    }

    Ok((count, generated_csv, generated_excel))
}

pub struct AtsSyncApp {
    folder_path: Option<PathBuf>,
    jd_file_path: Option<PathBuf>,
    status_message: String,
    candidates_count: Option<usize>,
    csv_file: Option<String>,
    excel_file: Option<String>,
    export_format: ExportFormat,
    dark_mode: bool,
}

impl Default for AtsSyncApp {
    fn default() -> Self {
        let fallback = PathBuf::from("./resumes");
        let initial_path = if fallback.exists() { Some(fallback) } else { None };

        Self {
            folder_path: initial_path,
            jd_file_path: None,
            status_message: "Ready. Select folder and optional JD to process.".to_string(),
            candidates_count: None,
            csv_file: None,
            excel_file: None,
            export_format: ExportFormat::Excel,
            dark_mode: true,
        }
    }
}

impl eframe::App for AtsSyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_dark = self.dark_mode;

        // --- Glassmorphism Palette Setup ---
        let base_bg = if is_dark {
            Color32::from_rgb(11, 15, 25)
        } else {
            Color32::from_rgb(243, 246, 251)
        };

        let card_bg = if is_dark {
            Color32::from_rgba_unmultiplied(22, 30, 49, 215) // ~84% slate-dark
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 210) // ~82% frosted white
        };

        let card_border = if is_dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 24) // Subtly lit glass rim
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 255) // Solid white rim
        };

        let brand_accent = Color32::from_rgb(14, 165, 233);

        let text_main = if is_dark {
            Color32::from_rgb(241, 245, 249)
        } else {
            Color32::from_rgb(30, 41, 59)
        };

        let text_sub = if is_dark {
            Color32::from_rgb(148, 163, 184)
        } else {
            Color32::from_rgb(100, 116, 139)
        };

        let mut style = (*ctx.style()).clone();
        style.visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        ctx.set_style(style);

        egui::CentralPanel::default()
            .frame(Frame::none().fill(base_bg))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();

                // --- Background Ambient Aurora Glows ---
                let glow_left = if is_dark {
                    Color32::from_rgba_unmultiplied(37, 99, 235, 40) // Deep Blue
                } else {
                    Color32::from_rgba_unmultiplied(186, 230, 253, 145) // Sky Blue
                };

                let glow_right = if is_dark {
                    Color32::from_rgba_unmultiplied(147, 51, 234, 35) // Rich Violet
                } else {
                    Color32::from_rgba_unmultiplied(254, 240, 138, 110) // Warm Amber
                };

                painter.circle_filled(rect.left_top() + egui::vec2(100.0, 70.0), 240.0, glow_left);
                painter.circle_filled(rect.right_top() + egui::vec2(-90.0, 90.0), 210.0, glow_right);

                ui.vertical_centered(|ui| {
                    ui.set_max_width(560.0);
                    ui.add_space(8.0);

                    // --- Header & Theme Toggle ---
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let theme_label = if self.dark_mode { "Light Mode" } else { "Dark Mode" };
                            let theme_btn = egui::Button::new(
                                RichText::new(theme_label).size(12.0).strong().color(text_main)
                            )
                            .fill(if is_dark { Color32::from_rgba_unmultiplied(255, 255, 255, 15) } else { Color32::from_rgba_unmultiplied(255, 255, 255, 180) })
                            .min_size(Vec2::new(88.0, 24.0));

                            if ui.add(theme_btn).clicked() {
                                self.dark_mode = !self.dark_mode;
                            }
                        });
                    });

                    ui.add_space(2.0);

                    ui.label(
                        RichText::new("ABROADERZ ATS SYNC")
                            .font(FontId::new(20.0, FontFamily::Proportional))
                            .strong()
                            .color(brand_accent),
                    );

                    ui.label(
                        RichText::new("Automated Neural Resume Pipeline")
                            .font(FontId::new(12.0, FontFamily::Proportional))
                            .color(text_sub),
                    );

                    ui.add_space(10.0);

                    // --- Glass Card 1: Directory Selection & JD Upload ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0, card_border))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Source Directory:").size(13.0).strong().color(text_main));
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let mut path_display = self
                                    .folder_path
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "No directory selected...".to_string());

                                ui.add(
                                    egui::TextEdit::singleline(&mut path_display)
                                        .desired_width(420.0)
                                        .margin(Margin::symmetric(6.0, 5.0))
                                        .interactive(false),
                                );

                                let browse_btn = egui::Button::new(RichText::new("Browse").size(12.0).strong())
                                    .min_size(Vec2::new(76.0, 26.0));

                                if ui.add(browse_btn).clicked() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        self.folder_path = Some(folder);
                                        self.status_message = "Source directory updated.".to_string();
                                    }
                                }
                            });

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Job Description File (Optional):").size(13.0).strong().color(text_main));
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let mut jd_display = self
                                    .jd_file_path
                                    .as_ref()
                                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                                    .unwrap_or_else(|| "No JD file selected (scoring disabled)".to_string());

                                ui.add(
                                    egui::TextEdit::singleline(&mut jd_display)
                                        .desired_width(340.0)
                                        .margin(Margin::symmetric(6.0, 5.0))
                                        .interactive(false),
                                );

                                let upload_btn = egui::Button::new(RichText::new("Upload JD").size(12.0).strong())
                                    .min_size(Vec2::new(76.0, 26.0));

                                if ui.add(upload_btn).clicked() {
                                    if let Some(file) = rfd::FileDialog::new()
                                        .add_filter("Documents", &["pdf", "docx", "txt"])
                                        .pick_file()
                                    {
                                        self.jd_file_path = Some(file);
                                        self.status_message = "Job description loaded for scoring.".to_string();
                                    }
                                }

                                if self.jd_file_path.is_some() {
                                    let clear_btn = egui::Button::new(RichText::new("Clear").size(12.0))
                                        .min_size(Vec2::new(60.0, 26.0));
                                    if ui.add(clear_btn).clicked() {
                                        self.jd_file_path = None;
                                    }
                                }
                            });
                        });

                    ui.add_space(8.0);

                    // --- Glass Card 2: Export Options & Execution Trigger ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0, card_border))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Export Target:").size(13.0).strong().color(text_main));
                                ui.add_space(8.0);
                                ui.radio_value(&mut self.export_format, ExportFormat::Excel, "Excel (.xlsx)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Csv, "CSV (.csv)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Both, "Both");
                            });

                            ui.add_space(8.0);

                            let can_run = self.folder_path.is_some();
                            let run_btn = egui::Button::new(
                                RichText::new("RUN BATCH ATS EXTRACTION")
                                    .font(FontId::new(13.0, FontFamily::Proportional))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if can_run { brand_accent } else { Color32::from_rgb(100, 116, 139) })
                            .min_size(Vec2::new(ui.available_width(), 36.0));

                            let response = ui.add_enabled(can_run, run_btn);

                            if response.clicked() {
                                if let Some(ref path) = self.folder_path {
                                    self.csv_file = None;
                                    self.excel_file = None;

                                    match process_directory(path, self.export_format, self.jd_file_path.as_ref()) {
                                        Ok((count, csv, excel)) => {
                                            self.candidates_count = Some(count);
                                            self.csv_file = csv;
                                            self.excel_file = excel;
                                            self.status_message = format!(
                                                "Extraction complete. {} candidate profiles parsed.",
                                                count
                                            );
                                        }
                                        Err(err) => {
                                            self.status_message = format!("Error: {}", err);
                                        }
                                    }
                                }
                            }
                        });

                    ui.add_space(8.0);

                    // --- Glass Card 3: Results & File Actions ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0, card_border))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Status & Results").size(13.0).strong().color(text_main));
                            });

                            ui.add_space(4.0);

                            ui.label(
                                RichText::new(&self.status_message)
                                    .font(FontId::new(12.0, FontFamily::Proportional))
                                    .italics()
                                    .color(text_sub),
                            );

                            if let Some(count) = self.candidates_count {
                                ui.add_space(6.0);

                                ui.label(
                                    RichText::new(format!("[SUCCESS] {} Candidates Ingested", count))
                                        .font(FontId::new(13.0, FontFamily::Proportional))
                                        .strong()
                                        .color(Color32::from_rgb(34, 197, 94)),
                                );

                                ui.add_space(6.0);
                                ui.horizontal_centered(|ui| {
                                    if let Some(ref excel) = self.excel_file {
                                        let xlsx_btn = egui::Button::new(
                                            RichText::new("Open Excel").size(12.0).strong(),
                                        )
                                        .min_size(Vec2::new(120.0, 28.0));
                                        if ui.add(xlsx_btn).clicked() {
                                            let _ = open::that(excel);
                                        }
                                    }

                                    if let Some(ref csv) = self.csv_file {
                                        let csv_btn = egui::Button::new(
                                            RichText::new("Open CSV").size(12.0).strong(),
                                        )
                                        .min_size(Vec2::new(120.0, 28.0));
                                        if ui.add(csv_btn).clicked() {
                                            let _ = open::that(csv);
                                        }
                                    }
                                });
                            }
                        });
                });
            });
    }
}