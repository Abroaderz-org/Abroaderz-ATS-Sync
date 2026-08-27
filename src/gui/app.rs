use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, Stroke, Vec2};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::AppConfig;
use crate::engine::inference::infer_candidate_details;
use crate::export::{csv::export_candidates_to_csv, excel::export_candidates_to_excel};
use crate::parser::{image::extract_image_text, pdf::extract_pdf_text};

fn process_directory(dir_path: &Path) -> Result<(usize, String, String), String> {
    let mut candidates = Vec::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let raw_text = match ext.as_str() {
                "pdf" => extract_pdf_text(path).ok(),
                "png" | "jpg" | "jpeg" | "webp" => extract_image_text(path).ok(),
                _ => None,
            };

            if let Some(text) = raw_text {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                let candidate = infer_candidate_details(&text, file_name);
                candidates.push(candidate);
            }
        }
    }

    if candidates.is_empty() {
        return Err("No valid PDF or image resumes found in the chosen folder.".to_string());
    }

    let count = candidates.len();
    let csv_path = "Abroaderz_Candidates.csv".to_string();
    let excel_path = "Abroaderz_Candidates.xlsx".to_string();

    export_candidates_to_csv(&candidates, &csv_path)
        .map_err(|e| format!("CSV export failed: {}", e))?;

    export_candidates_to_excel(&candidates, &excel_path)
        .map_err(|e| format!("Excel export failed: {}", e))?;

    Ok((count, csv_path, excel_path))
}

pub struct AtsSyncApp {
    folder_path: Option<PathBuf>,
    status_message: String,
    candidates_count: Option<usize>,
    csv_file: Option<String>,
    excel_file: Option<String>,
    is_processing: bool,
    dark_mode: bool,
}

impl Default for AtsSyncApp {
    fn default() -> Self {
        let config = AppConfig::load();
        let default_dir = PathBuf::from(&config.input_directory);
        Self {
            folder_path: if default_dir.exists() { Some(default_dir) } else { None },
            status_message: "Ready. Select a folder to begin extraction.".to_string(),
            candidates_count: None,
            csv_file: None,
            excel_file: None,
            is_processing: false,
            dark_mode: true,
        }
    }
}

impl eframe::App for AtsSyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Global rounded styling for interactive widgets
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
        style.visuals.widgets.active.rounding = Rounding::same(6.0);
        ctx.set_style(style);

        let is_dark = self.dark_mode;
        let card_bg = if is_dark {
            Color32::from_rgb(26, 32, 44)
        } else {
            Color32::from_rgb(248, 250, 252)
        };
        let card_border = if is_dark {
            Color32::from_rgb(51, 65, 85)
        } else {
            Color32::from_rgb(226, 232, 240)
        };
        let brand_accent = Color32::from_rgb(14, 165, 233);

        egui::CentralPanel::default()
            .frame(Frame::none().inner_margin(Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_max_width(540.0);

                    // --- Header with circular Theme Toggle ---
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(
                                RichText::new("Abroaderz ATS Sync")
                                    .size(22.0)
                                    .strong()
                                    .color(brand_accent),
                            );
                            ui.label(
                                RichText::new("Automated Neural Resume Pipeline")
                                    .size(12.0)
                                    .color(if is_dark { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY }),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let theme_icon = if self.dark_mode { "☀️" } else { "🌙" };
                            let theme_btn = egui::Button::new(RichText::new(theme_icon).size(14.0))
                                .min_size(Vec2::new(32.0, 32.0));

                            if ui.add(theme_btn).clicked() {
                                self.dark_mode = !self.dark_mode;
                            }
                        });
                    });

                    ui.add_space(14.0);

                    // --- Card 1: Folder Selection ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(10.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📂 Source Directory").size(13.0).strong());
                            });
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                let mut path_display = self
                                    .folder_path
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "No folder selected...".to_string());

                                ui.add(
                                    egui::TextEdit::singleline(&mut path_display)
                                        .desired_width(385.0)
                                        .interactive(false),
                                );

                                let browse_btn = egui::Button::new(RichText::new("Browse").size(12.0).strong())
                                    .min_size(Vec2::new(75.0, 26.0));

                                if ui.add(browse_btn).clicked() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        self.folder_path = Some(folder);
                                        self.status_message = "Target directory set. Ready to parse.".to_string();
                                    }
                                }
                            });
                        });

                    ui.add_space(12.0);

                    // --- Card 2: Action Button ---
                    let can_run = self.folder_path.is_some() && !self.is_processing;
                    let action_label = if self.is_processing {
                        "⚡ Parsing Resumes..."
                    } else {
                        "⚡ Run ATS Extraction"
                    };

                    let run_btn = egui::Button::new(
                        RichText::new(action_label)
                            .size(14.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(if can_run { brand_accent } else { Color32::from_rgb(100, 116, 139) })
                    .min_size(Vec2::new(ui.available_width(), 38.0));

                    if ui.add_enabled(can_run, run_btn).clicked() {
                        if let Some(ref path) = self.folder_path {
                            self.is_processing = true;
                            self.status_message = "Extracting text and matching fields...".to_string();

                            match process_directory(path) {
                                Ok((count, csv, excel)) => {
                                    self.candidates_count = Some(count);
                                    self.csv_file = Some(csv);
                                    self.excel_file = Some(excel);
                                    self.status_message = format!("Successfully parsed {} candidate profiles.", count);
                                }
                                Err(err) => {
                                    self.status_message = format!("Error: {}", err);
                                }
                            }
                            self.is_processing = false;
                        }
                    }

                    ui.add_space(12.0);

                    // --- Card 3: Status & Direct Report Launchers ---
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(10.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📋 Status & Export").size(13.0).strong());
                            });
                            ui.add_space(6.0);

                            ui.label(
                                RichText::new(&self.status_message)
                                    .size(12.0)
                                    .italics()
                                    .color(if is_dark { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY }),
                            );

                            if let Some(count) = self.candidates_count {
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(format!("✓ {} Candidates Synced", count))
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::from_rgb(34, 197, 94)),
                                );

                                ui.add_space(8.0);
                                ui.horizontal_centered(|ui| {
                                    if let Some(ref csv) = self.csv_file {
                                        let csv_btn = egui::Button::new(RichText::new("📄 Open CSV").size(12.0).strong())
                                            .min_size(Vec2::new(140.0, 30.0));
                                        if ui.add(csv_btn).clicked() {
                                            let _ = open::that(csv);
                                        }
                                    }

                                    if let Some(ref excel) = self.excel_file {
                                        let xlsx_btn = egui::Button::new(RichText::new("📊 Open Excel").size(12.0).strong())
                                            .min_size(Vec2::new(140.0, 30.0));
                                        if ui.add(xlsx_btn).clicked() {
                                            let _ = open::that(excel);
                                        }
                                    }
                                });
                            }
                        });
                });
            });
    }
}