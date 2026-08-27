use eframe::egui;
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
        return Err("No valid PDF or Image resumes found in selected directory.".to_string());
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
}

impl Default for AtsSyncApp {
    fn default() -> Self {
        let config = AppConfig::load();
        let default_dir = PathBuf::from(&config.input_dir);
        Self {
            folder_path: if default_dir.exists() { Some(default_dir) } else { None },
            status_message: "Ready. Select a folder containing candidate resumes.".to_string(),
            candidates_count: None,
            csv_file: None,
            excel_file: None,
            is_processing: false,
        }
    }
}

impl eframe::App for AtsSyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(
                    egui::RichText::new("Abroaderz ATS Sync")
                        .size(24.0)
                        .bold()
                        .color(egui::Color32::from_rgb(0, 168, 232)),
                );
                ui.label("Automated Resume Parsing & Export Tool");
                ui.add_space(15.0);
            });

            ui.group(|ui| {
                ui.heading("1. Resume Source Folder");
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    let mut path_str = self
                        .folder_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No folder selected...".to_string());

                    ui.add(
                        egui::TextEdit::singleline(&mut path_str)
                            .desired_width(450.0)
                            .interactive(false),
                    );

                    if ui.button("📁 Browse...").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.folder_path = Some(folder);
                            self.status_message = "Folder selected. Ready to sync.".to_string();
                        }
                    }
                });
            });

            ui.add_space(15.0);

            ui.group(|ui| {
                ui.heading("2. Synchronization Action");
                ui.add_space(8.0);

                let can_process = self.folder_path.is_some() && !self.is_processing;

                if ui
                    .add_enabled(
                        can_process,
                        egui::Button::new("⚡ Run ATS Sync Engine")
                            .min_size(egui::vec2(220.0, 36.0)),
                    )
                    .clicked()
                {
                    if let Some(ref path) = self.folder_path {
                        self.is_processing = true;
                        self.status_message = "Parsing candidate documents...".to_string();

                        match process_directory(path) {
                            Ok((count, csv, excel)) => {
                                self.candidates_count = Some(count);
                                self.csv_file = Some(csv);
                                self.excel_file = Some(excel);
                                self.status_message = format!("Complete! Processed {} resumes.", count);
                            }
                            Err(err) => {
                                self.status_message = format!("Error: {}", err);
                            }
                        }
                        self.is_processing = false;
                    }
                }
            });

            ui.add_space(15.0);

            ui.group(|ui| {
                ui.heading("3. Output Status & Reports");
                ui.add_space(5.0);

                ui.label(
                    egui::RichText::new(&self.status_message)
                        .size(13.0)
                        .italics(),
                );

                if let Some(count) = self.candidates_count {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("✓ {} Candidates Successfully Parsed", count))
                            .bold()
                            .color(egui::Color32::GREEN),
                    );

                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if let Some(ref csv) = self.csv_file {
                            if ui.button("📄 Open CSV Spreadsheet").clicked() {
                                let _ = open::that(csv);
                            }
                        }
                        if let Some(ref excel) = self.excel_file {
                            if ui.button("📊 Open Excel Report").clicked() {
                                let _ = open::that(excel);
                            }
                        }
                    });
                }
            });
        });
    }
}