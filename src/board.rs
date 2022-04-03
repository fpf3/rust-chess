use std::fmt;
//use std::str;
use std::collections::HashMap;

#[derive(Copy,Clone,PartialEq)]
enum Color {
    Empty,
    White,
    Black,
}

impl Default for Color {
    fn default() -> Self { Color::Empty }
}

#[derive(Copy,Clone,PartialEq)]
enum PieceType {
    Empty,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Default for PieceType {
    fn default() -> Self { PieceType::Empty }
}


#[derive(Copy,Clone,PartialEq)]
enum GameResult {
    Active,
    Draw,
    WhiteTime,
    WhiteResign,
    WhiteCheckmate,
    BlackTime,
    BlackResign,
    BlackCheckmate,
}

impl Default for GameResult {
    fn default() -> Self { GameResult::Active }
}

#[derive(Default,Copy,Clone)]
struct Square {
    color: Color,
    piece: PieceType,
}

struct Board {
    squares: [Square; 64],
    to_play: Color,
    castling: (bool, bool, bool, bool), // KQkq
    en_passant: (bool,usize,usize),
    halfmove_clock: u16,
    fullmove_number: u16,
    result: GameResult,
}

impl Board {
    const PIECE_MAP: [char; 7] = ['.', 'P', 'R', 'N', 'B', 'Q', 'K'];
    const START_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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
                    board_string.push(Board::PIECE_MAP[index as usize]);
                }
                else if color == Color::Black {
                    board_string.push(Board::PIECE_MAP[index as usize].to_lowercase().collect::<Vec<_>>()[0]);
                }
                else {
                    board_string.push(Board::PIECE_MAP[0]);
                }
            }
            board_string.push_str("\n");
        }

        let statstr: String = format!("Move: {}, {} to play, {}", 
                                      self.fullmove_number,
                                      match self.to_play {
                                          Color::Empty=>"error",
                                          Color::White=>"White",
                                          Color::Black=>"Black",
                                      },
                                      match self.result {
                                          GameResult::Active=>"...",
                                          GameResult::Draw=>"draw",
                                          GameResult::WhiteTime=>"Black timed out, white is victorious.",
                                          GameResult::WhiteResign=>"Black resigned, white is victorious.",
                                          GameResult::WhiteCheckmate=>"Checkmate, white is victorious.",
                                          GameResult::BlackTime=>"White timed out, black is victorious.",
                                          GameResult::BlackResign=>"White resigned, black is victorious.",
                                          GameResult::BlackCheckmate=>"Checkmate, black is victorious.",
                                      });

        board_string.push_str(&statstr[..]);


        return board_string;
    }

    fn from_fen(fen_string: &str)->Result<Board, i16> {
        let fen_fields: Vec<&str> = fen_string.split(" ").collect();       
        let ranks= fen_fields[0].split("/");
        let toplay = fen_fields[1];
        let castling = fen_fields[2];
        let enpassant = fen_fields[3];
        let halfmove = fen_fields[4];
        let fullmove = fen_fields[5];

        let mut board_index: usize = 0;
        let mut new_board: Board = Board::default();

        let piececharmap = HashMap::from([
            ('P', PieceType::Pawn),
            ('R', PieceType::Rook),
            ('N', PieceType::Knight),
            ('B', PieceType::Bishop),
            ('Q', PieceType::Queen),
            ('K', PieceType::King),
        ]);

        // populate board
        for (i, rank) in ranks.enumerate() {
            for (j, c) in rank.chars().enumerate() {
                if c.is_digit(10) { // empty squares
                    for k in 0..c.to_digit(10).unwrap() {
                        new_board.squares[board_index] = Square::default();
                        board_index += 1;
                    }
                }
                else { // piece...
                    if c.is_uppercase() { // white piece...
                        new_board.squares[board_index] = Square { 
                            piece: match piececharmap.get(&c) {
                                Some(&piece) => piece,
                                None => PieceType::Empty,
                            },

                            color: Color::White,
                        };
                        //new_board.squares[board_index] = Square { piece: PieceType::Pawn, color: Color::White };
                        board_index += 1;
                    }
                    else { // black piece...
                        new_board.squares[board_index] = Square { 
                            piece: match piececharmap.get(&c.to_uppercase().collect::<Vec<_>>()[0]){
                                Some(&piece) => piece,
                                None => PieceType::Empty,
                            }, 

                            color: Color::Black };
                        board_index += 1;
                    }
                }
                
            }
        }

        // set board state
        if toplay == "w" {
            new_board.to_play = Color::White;
        }
        else {
            new_board.to_play = Color::Black;
        }
        
        if castling.contains("K") {
            new_board.castling.0 = true;
        }

        if castling.contains("Q") {
            new_board.castling.1 = true;
        }

        if castling.contains("k") {
            new_board.castling.2 = true;
        }

        if castling.contains("q") {
            new_board.castling.3 = true;
        }

        new_board.halfmove_clock = halfmove.parse::<u16>().unwrap();
        new_board.fullmove_number = fullmove.parse::<u16>().unwrap();
        new_board.result = GameResult::Active;


        return Ok(new_board);
    }

    fn fide_init()->Result<Board,i16> {
        Board::from_fen(Board::START_FEN)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            squares: [Square::default(); 64],
            to_play: Color::White,
            castling: (false, false, false, false),
            en_passant: (false, 0, 0),
            halfmove_clock: 0,
            fullmove_number: 0,
            result: GameResult::default(),
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

    board = Board::fide_init().unwrap();
    println!("board has been initialized from FEN string: {}\n", Board::START_FEN);
    println!("{}", board);

}
