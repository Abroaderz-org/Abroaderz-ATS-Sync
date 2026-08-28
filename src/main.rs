#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod export;
pub mod gui;
pub mod parser;

use eframe::egui::{self, Vec2};
use gui::AtsSyncApp;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn SetProcessDpiAwarenessContext(value: isize) -> i32;
        }
        let _ = SetProcessDpiAwarenessContext(-4);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(760.0, 680.0))
            .with_min_inner_size(Vec2::new(640.0, 560.0))
            .with_transparent(true)
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