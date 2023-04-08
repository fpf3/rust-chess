use eframe::egui;
mod board;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2{x: 1000.0, y: 700.0}),
        ..Default::default()
    };
    eframe::run_native(
        "rust_chess",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    svg_image: egui_extras::RetainedImage,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            svg_image: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_king.svg",
                include_bytes!("../resource/svg/pieces/white_king.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rust_chess");
            ui.label("This is the white king from the hit game 'chess'");

            ui.separator();

            let max_size = ui.available_size();
            self.svg_image.show_size(ui, max_size);
        });
    }
}