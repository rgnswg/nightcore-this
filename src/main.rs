use eframe::egui;
mod app;
mod audio;
mod ui;
mod state;

use app::NightcoreApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        default_theme: eframe::Theme::Dark,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([600.0, 300.0])
            .with_max_inner_size([600.0, 300.0])
            .with_resizable(false)
            .with_decorations(true)
            .with_window_level(egui::WindowLevel::AlwaysOnTop)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Nightcore This",
        options,
        Box::new(|_cc| Box::new(NightcoreApp::default())),
    )
}
