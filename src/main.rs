use rust_chess::gui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2{x: 1000.0, y: 700.0}),
        ..Default::default()
    };
    eframe::run_native(
        "rust_chess",
        options,
        Box::new(|_cc| Box::new(gui::ChessGUI::default())),
    )
}
