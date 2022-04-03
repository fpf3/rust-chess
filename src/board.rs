use std::fmt;
//use std::str;

#[derive(Copy,Clone,PartialEq)]
enum Color {
    Empty,
    White,
    Black,
}

impl Default for Color {
    fn default() -> Self { Color::Empty}
}

#[derive(Copy,Clone,PartialEq)]
enum PieceType {
    Empty,
    Pawn,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Default for PieceType {
    fn default() -> Self { PieceType::Empty}
}

#[derive(Default,Copy,Clone)]
struct Square {
    color: Color,
    piece: PieceType,
}

struct Board {
    squares: [Square; 64],
}

impl Board {
    const piece_map: [char; 6] = ['.', 'P', 'N', 'B', 'Q', 'K'];

    fn print_board(&self)->String {
        let mut index: PieceType;
        let mut square: &Square;
        let mut color: Color;
        let mut board_string: String = "".to_string();
        for i in 0..8 {
            for j in 0..8{
                square = &self.squares[i*8+j];
                index = square.piece;
                color = square.color;
                if color == Color::White {
                    board_string.push(Board::piece_map[index as usize]);
                }
                else if color == Color::Black {
                    board_string.push(Board::piece_map[index as usize].to_lowercase().collect::<Vec<_>>()[0]);
                }
                else {
                    board_string.push(Board::piece_map[0]);
                }
            }
            board_string.push_str("\n");
        }
        return board_string;
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            squares: [Square::default(); 64]
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Board::print_board(self))
    }
}

fn main() {
    let mut board: Board = Board::default();
    println!("ahhh yes... chess.");
    println!("{}", board);

    board.squares[12] = Square { color: Color::Black, piece: PieceType::Queen };
    println!("{}", board);

}
