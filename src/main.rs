#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod export;
pub mod gui;
pub mod parser;

use eframe::egui;
use gui::AtsSyncApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 560.0])
            .with_min_inner_size([640.0, 500.0])
            .with_resizable(true)
            .with_title("Abroaderz ATS Sync"),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "Abroaderz ATS Sync",
        options,
        Box::new(|_cc| Ok(Box::new(AtsSyncApp::default()))),
    )
}