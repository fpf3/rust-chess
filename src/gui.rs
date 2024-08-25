use eframe::egui;
use eframe::egui::include_image;
use eframe::egui::pos2;

use epaint::{
    Color32,
    Rect,
};

use std::collections::HashMap;

use crate::board;

pub struct ChessGUI {
    game: board::Board,
    piece_assets: HashMap<(board::Color, board::PieceType), egui::Image>
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
    const DEF_SQ_SIZE: f32 = 75.;

    fn gen_piece_assets() -> HashMap<(board::Color, board::PieceType), egui::Image> {
        HashMap::from([
            ((board::Color::White, board::PieceType::Pawn),     egui::Image::new(egui::include_image!("../resource/svg/pieces/white_pawn.svg"))),
            ((board::Color::White, board::PieceType::King),     egui::Image::new(egui::include_image!("../resource/svg/pieces/white_king.svg"))),
            ((board::Color::White, board::PieceType::Queen),    egui::Image::new(egui::include_image!("../resource/svg/pieces/white_queen.svg"))),
            ((board::Color::White, board::PieceType::Bishop),   egui::Image::new(egui::include_image!("../resource/svg/pieces/white_bishop.svg"))),
            ((board::Color::White, board::PieceType::Knight),   egui::Image::new(egui::include_image!("../resource/svg/pieces/white_knight.svg"))),
            ((board::Color::White, board::PieceType::Rook),     egui::Image::new(egui::include_image!("../resource/svg/pieces/white_rook.svg"))),
            ((board::Color::Black, board::PieceType::Pawn),     egui::Image::new(egui::include_image!("../resource/svg/pieces/black_pawn.svg"))),
            ((board::Color::Black, board::PieceType::King),     egui::Image::new(egui::include_image!("../resource/svg/pieces/black_king.svg"))),
            ((board::Color::Black, board::PieceType::Queen),    egui::Image::new(egui::include_image!("../resource/svg/pieces/black_queen.svg"))),
            ((board::Color::Black, board::PieceType::Bishop),   egui::Image::new(egui::include_image!("../resource/svg/pieces/black_bishop.svg"))),
            ((board::Color::Black, board::PieceType::Knight),   egui::Image::new(egui::include_image!("../resource/svg/pieces/black_knight.svg"))),
            ((board::Color::Black, board::PieceType::Rook),     egui::Image::new(egui::include_image!("../resource/svg/pieces/black_rook.svg"))),
        ])
    }
}

impl eframe::App for ChessGUI {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let total_window = ui.available_size();
            ui.heading(match self.game.to_play {
                board::Color::White => "White to play...",
                board::Color::Black => "Black to play..."
            });

            ui.separator();

            let draw_window = ui.available_size();

            let painter = egui::Painter::new(ctx.clone(), egui::layers::LayerId::new(
                egui::layers::Order::Foreground,
                egui::Id::new("master painter")),
                egui::Rect::from_min_size(egui::Pos2::ZERO, draw_window)
            );


            let sq_size = f32::min(Self::DEF_SQ_SIZE, f32::min(draw_window.x/8., draw_window.y/8.));

            let x_pad = (|x: &f32| {
                if x < &Self::DEF_SQ_SIZE {
                    0.
                } else {
                    (draw_window.x - (8.*x)) / 2.
                }
            })(&(draw_window.x/8.));

            let y_pad = total_window.y - draw_window.y;

            for j in 0..self.game.shape.1 {
                for i in 0..self.game.shape.0 {
                    let index = i*self.game.shape.1 + j;
                    let square = &self.game.squares[index];
                    let square_color = match (i^j)&1 {
                        0 => Self::LIGHT_SQ_COLOR,
                        1 => Self::DARK_SQ_COLOR,
                        _ => panic!("wtf..."),
                    };

                    let thisrect = egui::Rect{
                        min: egui::Pos2{x: (j as f32) * sq_size + x_pad, y: (i as f32) * sq_size + y_pad},
                        max: egui::Pos2{x: ((j as f32)+1.) * sq_size + x_pad, y: ((i as f32)+1.) * sq_size + y_pad},
                    };

                    painter.rect_filled(thisrect, 0.0, square_color);


                    match &self.piece_assets.get(&(square.color, square.piece)) {
                        Some(s) => s
                            .max_width(sq_size)
                            .paint_at(ui, thisrect),
                        _ => (),
                    };
                } 
            }
        });
    }
}
