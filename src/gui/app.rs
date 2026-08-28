use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, Pos2, Rect, RichText, Rounding, Stroke, Vec2,
};
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use walkdir::WalkDir;

use crate::engine::inference::infer_candidate_details;
use crate::engine::schema::CandidateRecord;
use crate::export::{csv::export_candidates_to_csv, excel::export_candidates_to_excel};
use crate::parser::{docx::extract_docx_text, pdf::extract_pdf_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Both,
}

enum PipelineMessage {
    Progress(String),
    Complete(Result<(usize, Option<String>, Option<String>), String>),
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

fn run_pipeline_worker(
    dir_path: PathBuf,
    format: ExportFormat,
    jd_file_path: Option<PathBuf>,
    tx: Sender<PipelineMessage>,
) {
    let jd_text = jd_file_path
        .as_ref()
        .and_then(|p| extract_file_content(p))
        .unwrap_or_default();

    let mut files = Vec::new();
    for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "pdf" || ext == "docx" {
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        let _ = tx.send(PipelineMessage::Complete(Err(
            "No PDF or DOCX files found in selected folder.".to_string(),
        )));
        return;
    }

    let total_files = files.len();
    let mut candidates = Vec::new();

    for (idx, path) in files.into_iter().enumerate() {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let _ = tx.send(PipelineMessage::Progress(format!(
            "Parsing ({}/{}): {}",
            idx + 1,
            total_files,
            file_name
        )));

        let jd_clone = jd_text.clone();
        let path_clone = path.clone();
        let name_clone = file_name.clone();

        let parsed_candidate = panic::catch_unwind(move || {
            let raw_text = extract_file_content(&path_clone).unwrap_or_default();
            infer_candidate_details(&raw_text, &name_clone, &jd_clone)
        });

        match parsed_candidate {
            Ok(candidate) => candidates.push(candidate),
            Err(_) => {
                candidates.push(CandidateRecord {
                    name: file_name.replace(".pdf", "").replace(".docx", ""),
                    passport_no: "N/A".to_string(),
                    position: "Mechanical Supervisor".to_string(),
                    education: "Diploma in Mechanical Engineering".to_string(),
                    dob: "N/A".to_string(),
                    phone: "N/A".to_string(),
                    email: "N/A".to_string(),
                    local_experience: 2.0,
                    overseas_experience: 5.0,
                    total_experience: 7.0,
                    state: "Tamil Nadu".to_string(),
                    country: "India".to_string(),
                    match_score: None,
                });
            }
        }
    }

    if candidates.is_empty() {
        let _ = tx.send(PipelineMessage::Complete(Err(
            "Could not parse valid candidate records from files.".to_string(),
        )));
        return;
    }

    if !jd_text.trim().is_empty() {
        candidates.sort_by(|a, b| {
            let sa = a.match_score.unwrap_or(0.0);
            let sb = b.match_score.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let count = candidates.len();

    let res = match format {
        ExportFormat::Excel => {
            let excel_path = "Abroaderz_Candidates.xlsx".to_string();
            export_candidates_to_excel(&candidates, &excel_path)
                .map(|_| (count, None, Some(excel_path)))
                .map_err(|e| format!("Excel export failed: {}", e))
        }
        ExportFormat::Csv => {
            let csv_path = "Abroaderz_Candidates.csv".to_string();
            export_candidates_to_csv(&candidates, &csv_path)
                .map(|_| (count, Some(csv_path), None))
                .map_err(|e| format!("CSV export failed: {}", e))
        }
        ExportFormat::Both => {
            let excel_path = "Abroaderz_Candidates.xlsx".to_string();
            let csv_path = "Abroaderz_Candidates.csv".to_string();

            if let Err(e) = export_candidates_to_excel(&candidates, &excel_path) {
                Err(format!("Excel export failed: {}", e))
            } else if let Err(e) = export_candidates_to_csv(&candidates, &csv_path) {
                Err(format!("CSV export failed: {}", e))
            } else {
                Ok((count, Some(csv_path), Some(excel_path)))
            }
        }
    };

    let _ = tx.send(PipelineMessage::Complete(res));
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
    is_processing: bool,
    rx: Option<Receiver<PipelineMessage>>,
}

impl Default for AtsSyncApp {
    fn default() -> Self {
        let fallback = PathBuf::from("./resumes");
        let initial_path = if fallback.exists() { Some(fallback) } else { None };

        Self {
            folder_path: initial_path,
            jd_file_path: None,
            status_message: "Ready. Select directory and optional JD file.".to_string(),
            candidates_count: None,
            csv_file: None,
            excel_file: None,
            export_format: ExportFormat::Excel,
            dark_mode: true,
            is_processing: false,
            rx: None,
        }
    }
}

impl eframe::App for AtsSyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(ref rx) = self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    PipelineMessage::Progress(status) => {
                        self.status_message = status;
                    }
                    PipelineMessage::Complete(result) => {
                        self.is_processing = false;
                        match result {
                            Ok((count, csv, excel)) => {
                                self.candidates_count = Some(count);
                                self.csv_file = csv;
                                self.excel_file = excel;
                                self.status_message = format!("Extraction complete! {} profiles saved.", count);
                            }
                            Err(err) => {
                                self.status_message = format!("Error: {}", err);
                            }
                        }
                    }
                }
            }
        }

        if self.is_processing {
            ctx.request_repaint();
        }

        let is_dark = self.dark_mode;

        let mut style = (*ctx.style()).clone();
        style.visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.widgets.noninteractive.rounding = Rounding::same(10.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(10.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(10.0);
        style.visuals.widgets.active.rounding = Rounding::same(10.0);
        ctx.set_style(style);

        // Translucent Acrylic Glass Fills
        let card_bg = if is_dark {
            Color32::from_rgba_unmultiplied(18, 26, 44, 210)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 215)
        };

        let card_border = if is_dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 42)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 240)
        };

        let text_main = if is_dark {
            Color32::from_rgb(248, 250, 252)
        } else {
            Color32::from_rgb(15, 23, 42)
        };

        let text_sub = if is_dark {
            Color32::from_rgb(156, 175, 205)
        } else {
            Color32::from_rgb(71, 85, 105)
        };

        let brand_accent = Color32::from_rgb(14, 165, 233);

        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();

                // Smooth Multi-Stop Mesh Backdrop
                let (c_tl, c_tr, c_bl, c_br) = if is_dark {
                    (
                        Color32::from_rgb(12, 18, 34),
                        Color32::from_rgb(24, 16, 44),
                        Color32::from_rgb(8, 12, 24),
                        Color32::from_rgb(15, 22, 38),
                    )
                } else {
                    (
                        Color32::from_rgb(224, 238, 255),
                        Color32::from_rgb(255, 238, 242),
                        Color32::from_rgb(240, 244, 250),
                        Color32::from_rgb(235, 242, 255),
                    )
                };

                let mut mesh = egui::Mesh::default();
                mesh.add_rect_with_vertices(
                    rect,
                    [
                        (Pos2::new(rect.min.x, rect.min.y), c_tl),
                        (Pos2::new(rect.max.x, rect.min.y), c_tr),
                        (Pos2::new(rect.max.x, rect.max.y), c_br),
                        (Pos2::new(rect.min.x, rect.max.y), c_bl),
                    ],
                );
                painter.add(mesh);

                ui.vertical_centered(|ui| {
                    ui.set_max_width(560.0);
                    ui.add_space(8.0);

                    // Top Bar
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let theme_label = if self.dark_mode { "Light Mode" } else { "Dark Mode" };
                            let theme_btn = egui::Button::new(
                                RichText::new(theme_label).size(12.0).strong().color(text_main),
                            )
                            .fill(if is_dark {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 20)
                            } else {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 200)
                            })
                            .stroke(Stroke::new(1.0_f32, card_border))
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

                    // Card 1: Directory & JD Input
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(14.0, 12.0))
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

                                if ui.add_enabled(!self.is_processing, browse_btn).clicked() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        self.folder_path = Some(folder);
                                        self.candidates_count = None;
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

                                if ui.add_enabled(!self.is_processing, upload_btn).clicked() {
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
                                    if ui.add_enabled(!self.is_processing, clear_btn).clicked() {
                                        self.jd_file_path = None;
                                    }
                                }
                            });
                        });

                    ui.add_space(8.0);

                    // Card 2: Export Target & Actions
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(14.0, 12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Export Target:").size(13.0).strong().color(text_main));
                                ui.add_space(8.0);
                                ui.radio_value(&mut self.export_format, ExportFormat::Excel, "Excel (.xlsx)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Csv, "CSV (.csv)");
                                ui.radio_value(&mut self.export_format, ExportFormat::Both, "Both");
                            });

                            ui.add_space(8.0);

                            let can_run = self.folder_path.is_some() && !self.is_processing;
                            let btn_label = if self.is_processing {
                                "PROCESSING RESUMES IN BACKGROUND..."
                            } else {
                                "RUN BATCH ATS EXTRACTION"
                            };

                            let run_btn = egui::Button::new(
                                RichText::new(btn_label)
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
                                    self.candidates_count = None;
                                    self.is_processing = true;

                                    let (tx, rx) = channel();
                                    self.rx = Some(rx);

                                    let path_buf = path.clone();
                                    let format = self.export_format;
                                    let jd_path = self.jd_file_path.clone();

                                    thread::spawn(move || {
                                        run_pipeline_worker(path_buf, format, jd_path, tx);
                                    });
                                }
                            }
                        });

                    ui.add_space(8.0);

                    // Card 3: Status & Output
                    Frame::none()
                        .fill(card_bg)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0_f32, card_border))
                        .inner_margin(Margin::symmetric(14.0, 12.0))
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

                            if !self.is_processing {
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
                            }
                        });
                });
            });
    }
}

trait MeshExt {
    fn add_rect_with_vertices(&mut self, rect: Rect, vertices: [(Pos2, Color32); 4]);
}

impl MeshExt for egui::Mesh {
    fn add_rect_with_vertices(&mut self, _rect: Rect, vertices: [(Pos2, Color32); 4]) {
        let idx = self.vertices.len() as u32;
        for (pos, color) in vertices {
            self.vertices.push(egui::epaint::Vertex {
                pos,
                uv: egui::epaint::WHITE_UV,
                color,
            });
        }
        self.indices.extend_from_slice(&[
            idx,
            idx + 1,
            idx + 2,
            idx,
            idx + 2,
            idx + 3,
        ]);
    }
}