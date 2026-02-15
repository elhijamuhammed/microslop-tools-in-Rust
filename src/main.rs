mod models;
mod collectors;
mod formats;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Microslop Tools (Rust)")
            .with_inner_size([1100.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Microslop Tools",
        options,
        Box::new(|_| Ok(Box::<models::App>::default())),
    )
}
