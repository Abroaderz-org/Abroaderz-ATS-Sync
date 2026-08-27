#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod engine;
mod export;
pub mod gui;
pub mod parser;

use eframe::egui;
use gui::AtsSyncApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 520.0])
            .with_min_inner_size([500.0, 420.0])
            .with_resizable(true)
            .with_title("Abroaderz ATS Sync"),
        ..Default::default()
    };

    eframe::run_native(
        "Abroaderz ATS Sync",
        options,
        Box::new(|_cc| Ok(Box::new(AtsSyncApp::default()))),
    )
}