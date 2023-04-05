use std::fmt;
//use std::str;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const PIECE_MAP: [char; 7] = ['.', 'P', 'R', 'N', 'B', 'Q', 'K'];

#[derive(Copy,Clone,PartialEq)]
enum Color {
    White,
    Black,
}

impl Default for Color {
    fn default() -> Self { Color::White } // Used to have Color::Empty, but I think it makes more sense for that to live in PieceType...
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
    DrawAgreement,
    DrawThreefold,
    Draw50Moves,
    DrawInsufficientMaterial,
    DrawTimeoutInsufficientMaterial,
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

struct Move {
    from: (i16, i16),
    to:   (i16, i16),
    set_enpassant: (bool, i16, i16),
}

struct Board {
    squares: Vec<Square>,
    shape: (u16, u16), // (height, width)
    piece_map: HashMap<PieceType, Vec<u16>>,
    to_play: Color,
    castling: (bool, bool, bool, bool), // KQkq
    en_passant: (bool,u16), // flag, coords behind pawn to be captured
    halfmove_clock: u16,
    fullmove_number: u16,
    result: GameResult,
}

impl Board {
    fn print_board(&self)->String {
        let mut index: PieceType;
        let mut square: &Square;
        let mut color: Color;
        let mut board_string: String = "".to_string();
        let height: usize = self.shape.0 as usize;
        let width: usize = self.shape.1 as usize;
        for i in 0..height {
            for j in 0..width{
                square = &self.squares[i*width+j];
                index = square.piece;
                color = square.color;
                if color == Color::White {
                    board_string.push(PIECE_MAP[index as usize]);
                }
                else if color == Color::Black {
                    board_string.push(PIECE_MAP[index as usize].to_lowercase().collect::<Vec<_>>()[0]);
                }
                else {
                    board_string.push(PIECE_MAP[0]);
                }
            }
            board_string.push('\n');
        }

        let statstr: String = format!("Move: {}, {} to play, {}, Castling: {}", 
                                      self.fullmove_number,
                                      match self.to_play {
                                          Color::White=>"White",
                                          Color::Black=>"Black",
                                      },
                                      match self.result {
                                          GameResult::Active=>"...",
                                          GameResult::DrawAgreement=>"Draw by mutual agreement",
                                          GameResult::DrawThreefold=>"Three-fold repetition - draw.",
                                          GameResult::Draw50Moves=>"50 moves w/o capture or pawn move - draw.",
                                          GameResult::DrawInsufficientMaterial=>"Insufficient material - draw.",
                                          GameResult::DrawTimeoutInsufficientMaterial=>"Timeout & insufficient material - draw.",
                                          GameResult::WhiteTime=>"Black timed out, white is victorious.",
                                          GameResult::WhiteResign=>"Black resigned, white is victorious.",
                                          GameResult::WhiteCheckmate=>"Checkmate, white is victorious.",
                                          GameResult::BlackTime=>"White timed out, black is victorious.",
                                          GameResult::BlackResign=>"White resigned, black is victorious.",
                                          GameResult::BlackCheckmate=>"Checkmate, black is victorious.",
                                      },
                                      match self.castling {
                                        (false, false, false, false)    => "----",
                                        (false, false, false, true)     => "---q",
                                        (false, false, true, false)     => "--k-",
                                        (false, false, true, true)      => "--kq",
                                        (false, true, false, false)     => "-Q--",
                                        (false, true, false, true)      => "-Q-q",
                                        (false, true, true, false)      => "-Qk-",
                                        (false, true, true, true)       => "-Qkq",
                                        (true, false, false, false)     => "K---",
                                        (true, false, false, true)      => "K--q",
                                        (true, false, true, false)      => "K-k-",
                                        (true, false, true, true)       => "K-kq",
                                        (true, true, false, false)      => "KQ--",
                                        (true, true, false, true)       => "KQ-q",
                                        (true, true, true, false)       => "KQk-",
                                        (true, true, true, true)        => "KQkq",
                                      }
                                    );

        board_string.push_str(&statstr[..]);
        
        board_string
    }

    fn alg_to_index(&self, alg_notation: &str)->u16{
        let c_str = alg_notation.as_bytes();
        let file: u16 = (c_str[0] - 48) as u16;
        let rank: u16 = (c_str[1] - 48) as u16;
        
        rank*self.shape.1 + file
    }

    fn from_fen(fen_string: &str)->Result<Board, i16> {
        lazy_static!{
            static ref FEN_EXP: Regex = Regex::new(r"^((?:[rnbqkpRNBQKP1-8]+/?){8})\s+([wb])\s+([KQkq\-]+)\s+([\-a-h1-8]+)\s+(\d)\s+(\d)").unwrap();
        }

        let fen_fields = match FEN_EXP.captures_iter(fen_string).next() {
            Some(x) => x,
            None => return Err(1),
        };

        let ranks= fen_fields[1].split('/');
        let toplay = &fen_fields[2];
        let castling = &fen_fields[3];
        let en_passant = &fen_fields[4];
        let halfmove = &fen_fields[5];
        let fullmove = &fen_fields[6];

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
        for (_, rank) in ranks.enumerate() {
            for (_, c) in rank.chars().enumerate() {
                if c.is_numeric() { // empty squares
                    for _ in 0..c.to_digit(10).unwrap() {
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
        
        if castling.contains('K') {
            new_board.castling.0 = true;
        }

        if castling.contains('Q') {
            new_board.castling.1 = true;
        }

        if castling.contains('k') {
            new_board.castling.2 = true;
        }

        if castling.contains('q') {
            new_board.castling.3 = true;
        }

        new_board.halfmove_clock = halfmove.parse::<u16>().unwrap();
        new_board.fullmove_number = fullmove.parse::<u16>().unwrap();

        if en_passant != "-" {
            new_board.en_passant = (true, new_board.alg_to_index(en_passant));
        }

        new_board.result = GameResult::Active;


        Ok(new_board)
    }

}

impl Default for Board {
    fn default() -> Self {
        Board {
            squares: vec![Square::default(); 64],
            shape: (8, 8),
            piece_map: HashMap::new(),
            to_play: Color::White,
            castling: (false, false, false, false),
            en_passant: (false, 0),
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

pub fn board_test() {
    let mut board: Board = Board::default();
    println!("ahhh yes... chess.");
    println!("{}", board);

    board = Board::from_fen(START_FEN).unwrap();
    println!("board has been initialized from FEN string: {}\n", START_FEN);
    println!("{}", board);
}
