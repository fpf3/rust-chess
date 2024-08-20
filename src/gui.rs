use eframe::egui;
use eframe::egui::pos2;
use epaint;
use epaint::Color32;
use epaint::Rect;

use std::collections::HashMap;

use crate::board;

pub struct ChessGUI {
    game: board::Board,
    piece_assets: HashMap<(board::Color, board::PieceType), egui_extras::RetainedImage>
}

impl Default for ChessGUI {
    fn default() -> Self {
        Self {
            game: board::Board::from_fen(board::START_FEN).unwrap(),
            piece_assets: Self::gen_piece_assets(),
        }
    }
}

impl ChessGUI{
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

impl eframe::App for ChessGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(match self.game.to_play {
                board::Color::White => "White to play...",
                board::Color::Black => "Black to play..."
            });

            ui.separator();

            let painter = egui::Painter::new(
                egui::Context::default(), 
                egui::layers::LayerId::new(
                    egui::layers::Order::Foreground,
                    egui::Id::new("master painter")
                ),
                egui::Rect::from_min_size(egui::Pos2::ZERO, ui.available_size())
            );

            let square_size: f32 = f32::min(ui.available_size().x, ui.available_size().y) / 8.;

            for j in 0..self.game.shape.1 {
                for i in 0..self.game.shape.0 {
                    let index = i*self.game.shape.1 + j;
                    let square = self.game.squares[index];
                    let square_color = match (i^j)&1 {
                        0 => Self::LIGHT_SQ_COLOR,
                        1 => Self::DARK_SQ_COLOR,
                        _ => panic!("wtf..."),
                    };

                    let thisrect = egui::Rect{
                        min: egui::Pos2{x: (j as f32) * square_size, y: (i as f32) * square_size},
                        max: egui::Pos2{x: square_size*((j as f32)+1.0), y: square_size*((i as f32)+1.0)},
                    };

                    painter.rect_filled(thisrect, 0.0, square_color);
                    
                    //painter.image(
                    //    self.piece_assets[&(square.color, square.piece)].texture_id(&ctx),
                    //    thisrect,
                    //    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    //    Color32::WHITE
                    //);
                } 
            }
        });
    }
}
