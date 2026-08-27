use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke, Vec2,
};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::engine::inference::infer_candidate_details;
use crate::export::{csv::export_candidates_to_csv, excel::export_candidates_to_excel};
use crate::parser::{docx::extract_docx_text, image::extract_image_text, pdf::extract_pdf_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Both,
}

fn process_directory(
    dir_path: &Path,
    format: ExportFormat,
    jd_text: &str,
) -> Result<(usize, Option<String>, Option<String>), String> {
    let mut candidates = Vec::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Safe parsing per format
            let raw_text = match ext.as_str() {
                "pdf" => extract_pdf_text(path).ok(),
                "docx" => extract_docx_text(path).ok(),
                "png" | "jpg" | "jpeg" | "webp" => extract_image_text(path).ok(),
                _ => None,
            };

            if let Some(text) = raw_text {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                let candidate = infer_candidate_details(&text, file_name, jd_text);
                candidates.push(candidate);
            }
        }
    }

    if candidates.is_empty() {
        return Err("No valid PDF, DOCX, or image resumes found in the chosen folder.".to_string());
    }

    // Sort candidates descending by match score
    candidates.sort_by(|a, b| {
        let score_a: f32 = a.match_score.parse().unwrap_or(0.0);
        let score_b: f32 = b.match_score.parse().unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap()
    });

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
    jd_input: String,
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
            jd_input: "Mechanical Supervisor Static Equipment Shutdown Refinery Aramco Sabic".to_string(),
            status_message: "Ready. Select a folder to begin extraction.".to_string(),
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
        let visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(visuals);

        let is_dark = self.dark_mode;
        let card_bg = if is_dark {
            Color32::from_rgb(22, 27, 34)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let card_border = if is_dark {
            Color32::from_rgb(48, 54, 61)
        } else {
            Color32::from_rgb(226, 232, 240)
        };
        let brand_accent = Color32::from_rgb(14, 165, 233);

        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
        style.visuals.widgets.active.rounding = Rounding::same(6.0);
        ctx.set_style(style);

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(Margin::symmetric(20.0, 8.0)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_max_width(520.0);

                    // --- Top Utility Row ---
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let theme_label = if self.dark_mode { "Light Mode" } else { "Dark Mode" };
                            let theme_btn = egui::Button::new(RichText::new(theme_label).size(10.5).strong())
                                .min_size(Vec2::new(80.0, 22.0));

                            if ui.add(theme_btn).clicked() {
                                self.dark_mode = !self.dark_mode;
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // --- Branding ---
                    ui.label(
                        RichText::new("ABROADERZ ATS SYNC")
                            .font(FontId::new(18.0, FontFamily::Proportional))
                            .strong()
                            .color(brand_accent),
                    );

                    ui.label(
                        RichText::new("Automated Neural Resume Pipeline")
                            .font(FontId::new(10.5, FontFamily::Proportional))
                            .color(if is_dark {
                                Color32::from_rgb(148, 163, 184)
                            } else {
                                Color32::from_rgb(100, 116, 139)
                            }),
                    );

                    ui.add_space(6.0);

                    // --- Card 1: Directory Selection & JD Input ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Source Directory:").size(11.0).strong());
                            });

                            ui.add_space(2.0);

                            ui.horizontal(|ui| {
                                let mut path_display = self
                                    .folder_path
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "No folder selected...".to_string());

                                ui.add(
                                    egui::TextEdit::singleline(&mut path_display)
                                        .desired_width(395.0)
                                        .margin(Margin::symmetric(6.0, 3.0))
                                        .interactive(false),
                                );

                                let browse_btn = egui::Button::new(RichText::new("Browse").size(10.5).strong())
                                    .min_size(Vec2::new(72.0, 22.0));

                                if ui.add(browse_btn).clicked() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        self.folder_path = Some(folder);
                                        self.status_message =
                                            "Target directory set. Ready for extraction.".to_string();
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Job Description Match Query:").size(10.5).strong());
                            });
                            ui.add_space(1.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.jd_input)
                                    .desired_width(470.0)
                                    .margin(Margin::symmetric(6.0, 3.0)),
                            );
                        });

                    ui.add_space(6.0);

                    // --- Card 2: Export Options & Execution Trigger ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Export Format:").size(11.0).strong());
                                ui.add_space(6.0);
                                ui.radio_value(&mut self.export_format, ExportFormat::Excel, "Excel (.xlsx)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Csv, "CSV (.csv)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Both, "Both");
                            });

                            ui.add_space(6.0);

                            let can_run = self.folder_path.is_some();
                            let run_btn = egui::Button::new(
                                RichText::new("RUN ATS EXTRACTION & RANKING")
                                    .font(FontId::new(12.0, FontFamily::Proportional))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if can_run { brand_accent } else { Color32::from_rgb(100, 116, 139) })
                            .min_size(Vec2::new(ui.available_width(), 30.0));

                            let response = ui.add_enabled(can_run, run_btn);

                            if response.hovered() && can_run {
                                ui.ctx().request_repaint();
                            }

                            if response.clicked() {
                                if let Some(ref path) = self.folder_path {
                                    self.csv_file = None;
                                    self.excel_file = None;

                                    match process_directory(path, self.export_format, &self.jd_input) {
                                        Ok((count, csv, excel)) => {
                                            self.candidates_count = Some(count);
                                            self.csv_file = csv;
                                            self.excel_file = excel;
                                            self.status_message = format!(
                                                "Pipeline completed. {} profiles ranked and exported.",
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

                    ui.add_space(6.0);

                    // --- Card 3: Results & Actions ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Status & Results").size(11.0).strong());
                            });

                            ui.add_space(2.0);

                            ui.label(
                                RichText::new(&self.status_message)
                                    .font(FontId::new(10.5, FontFamily::Proportional))
                                    .italics()
                                    .color(if is_dark {
                                        Color32::from_rgb(148, 163, 184)
                                    } else {
                                        Color32::from_rgb(100, 116, 139)
                                    }),
                            );

                            if let Some(count) = self.candidates_count {
                                ui.add_space(4.0);

                                ui.label(
                                    RichText::new(format!("[SUCCESS] {} Candidates Synchronized & Scored", count))
                                        .font(FontId::new(11.0, FontFamily::Proportional))
                                        .strong()
                                        .color(Color32::from_rgb(34, 197, 94)),
                                );

                                ui.add_space(4.0);
                                ui.horizontal_centered(|ui| {
                                    if let Some(ref excel) = self.excel_file {
                                        let xlsx_btn = egui::Button::new(
                                            RichText::new("Open Excel").size(10.5).strong(),
                                        )
                                        .min_size(Vec2::new(110.0, 24.0));
                                        if ui.add(xlsx_btn).clicked() {
                                            let _ = open::that(excel);
                                        }
                                    }

                                    if let Some(ref csv) = self.csv_file {
                                        let csv_btn = egui::Button::new(
                                            RichText::new("Open CSV").size(10.5).strong(),
                                        )
                                        .min_size(Vec2::new(110.0, 24.0));
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