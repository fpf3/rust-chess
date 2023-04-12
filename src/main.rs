use eframe::egui;
use epaint;
use std::collections::HashMap;
mod board;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2{x: 1000.0, y: 700.0}),
        ..Default::default()
    };
    eframe::run_native(
        "rust_chess",
        options,
        Box::new(|_cc| Box::new(ChessApp::default())),
    )
}

struct ChessApp {
    game: board::Board,
    piece_assets: HashMap<(board::Color, board::PieceType), egui_extras::RetainedImage>
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            game: board::Board::from_fen(board::START_FEN).unwrap(),
            piece_assets: Self::gen_piece_assets(),
        }
    }
}

impl ChessApp{
    const DARK_SQ_COLOR: epaint::Color32 =  epaint::Color32::from_rgb(115,66,7);
    const LIGHT_SQ_COLOR: epaint::Color32 = epaint::Color32::from_rgb(237,178,107);

    fn gen_piece_assets() -> HashMap<(board::Color, board::PieceType), egui_extras::RetainedImage> {
        HashMap::from([
            ((board::Color::White, board::PieceType::King), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_king.svg",
                include_bytes!("../resource/svg/pieces/white_king.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::White, board::PieceType::Queen), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_queen.svg",
                include_bytes!("../resource/svg/pieces/white_queen.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::White, board::PieceType::Bishop), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_bishop.svg",
                include_bytes!("../resource/svg/pieces/white_bishop.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::White, board::PieceType::Knight), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_knight.svg",
                include_bytes!("../resource/svg/pieces/white_knight.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::White, board::PieceType::Rook), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/white_rook.svg",
                include_bytes!("../resource/svg/pieces/white_rook.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::Black, board::PieceType::King), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/black_king.svg",
                include_bytes!("../resource/svg/pieces/black_king.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::Black, board::PieceType::Queen), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/black_queen.svg",
                include_bytes!("../resource/svg/pieces/black_queen.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::Black, board::PieceType::Bishop), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/black_bishop.svg",
                include_bytes!("../resource/svg/pieces/black_bishop.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::Black, board::PieceType::Knight), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/black_knight.svg",
                include_bytes!("../resource/svg/pieces/black_knight.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
            ((board::Color::Black, board::PieceType::Rook), egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../resource/svg/pieces/black_rook.svg",
                include_bytes!("../resource/svg/pieces/black_rook.svg"),
                egui_extras::image::FitTo::Original,
            ).unwrap()),
        ])
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(match self.game.to_play {
                board::Color::White => "White to play...",
                board::Color::Black => "Black to play..."
            });

            ui.separator();

            let painter = egui::Painter::new(egui::Context::default(), egui::layers::LayerId::new(
                egui::layers::Order::Foreground,
                egui::Id::new("master painter")),
                egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size())
            );

            //ui.add(egui::Painter);

            for j in 0..self.game.shape.1 {
                for i in 0..self.game.shape.0 {
                    let index = i*self.game.shape.1 + j;
                    let square_color = match (i^j)&1 {
                        0 => Self::LIGHT_SQ_COLOR,
                        1 => Self::DARK_SQ_COLOR,
                        _ => panic!("wtf..."),
                    };

                    let thisrect = egui::Rect{
                        min: egui::Pos2{x: (j as f32) * 50., y: (i as f32) * 50.},
                        max: egui::Pos2{x: (j as f32) * 50. + 50., y: (i as f32) * 50. + 50.},
                    };

                    painter.rect_filled(thisrect, 0.0, square_color);
                } 
            }
        });
    }
}