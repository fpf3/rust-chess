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
    squares: [Square; 64],
    to_play: Color,
    castling: (bool, bool, bool, bool), // KQkq
    en_passant: (bool,usize,usize), // flag, coords behind pawn to be captured
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
                square = &self.squares[(i*8+j) as usize];
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
                                      });

        board_string.push_str(&statstr[..]);


        return board_string;
    }

    fn get_sliding_squares(&self, loc: (i16, i16), piece: PieceType)->Vec<Move> {
        let start_index: i16 = loc.0 * 8 + loc.1;
        let start_sq = self.squares[start_index as usize];

        let mut moves: Vec<Move> = Vec::new();
        let mut index: i16 = 0;
        let mut eob_flag: bool = false;
        
        let mut target_loc: (i16, i16);
        let mut target_index: i16;
        let mut target: Square;

        let mut incs: Vec<i16> = Vec::new();
        let mut newmove: Move;
        let rook_incs: Vec<i16> = vec![8, -8, 1, -1];
        let bishop_incs: Vec<i16> = vec![9, 7, -7, -9];
        
        if self.squares[start_index as usize].color == Color::Empty {
            return moves;
        }

        if piece == PieceType::Rook{
            incs.extend(&rook_incs);
        }
        else if piece == PieceType::Bishop {
            incs.extend(&bishop_incs);
        }
        else if piece == PieceType::Queen {
            incs.extend(&rook_incs);
            incs.extend(&bishop_incs);
        }

        for inc in incs{ // down, up, left, right
            eob_flag = false;
            loop {
                index += inc;
                target_index = start_index + index;

                if      (start_index + index < 0) 
                     || (start_index + index >= 64)
                     || eob_flag {
                         break;
                     }

                if (start_index + index) & 7 == 0 || (start_index + index & 7 == 7) { // mod 8 hack
                    eob_flag = true;
                }
                
                target_loc = (target_index >> 3, target_index - (target_index & 0x7ff8));
                target = self.squares[(target_index) as usize];

                newmove = Move {
                    from: loc,
                    to: target_loc,
                    set_enpassant: (false, 0, 0),
                };

                if target.color == start_sq.color {
                    break;
                }
                else if (target.color != start_sq.color) && (target.color != Color::Empty) {
                    moves.push(newmove);
                    break;
                }
                moves.push(newmove);
            }
            index = 0;
        }

        return moves
    }

    fn get_rook_squares(&self, loc: (i16, i16))->Vec<Move> {
        return self.get_sliding_squares(loc, PieceType::Rook);
    }

    fn get_bishop_squares(&self, loc: (i16, i16))->Vec<Move>  {
        return self.get_sliding_squares(loc, PieceType::Bishop);
    }
    
    fn get_queen_squares(&self, loc: (i16, i16))->Vec<Move> {
        return self.get_sliding_squares(loc, PieceType::Queen);
    }

    fn get_knight_squares(&self, loc: (i16, i16))->Vec<Move> {
        let start_index: i16 = loc.0 * 8 + loc.1;
        let start_sq = self.squares[start_index as usize];
        let mut target_sq: Square;
        let mut target_index: i16;
        let mut target_loc: (i16, i16) = (0,0);
        let mut moves: Vec<Move> = Vec::new();
        let mut index_horiz_shift: i16;
        let mut dist_closest_edge: i16;
        let mut newmove: Move;
        
        if self.squares[start_index as usize].color == Color::Empty {
            return moves;
        }
    
        for inc in [-10, -6, -17, -15, 6, 10, 16, 17] { // all knight moves
            target_index = start_index + inc;
            target_loc = (target_index >> 3, target_index - (target_index & 0x7ff8));
            index_horiz_shift = target_loc.1 - loc.1;

            if (loc.1 < 4) {
                dist_closest_edge = loc.1;
            } else {
                dist_closest_edge = 8 - loc.1;
            }
            
            if (target_index < 0)
            || (target_index >= 64)
            || (index_horiz_shift.abs() > dist_closest_edge) {
                continue;
            }

            target_sq = self.squares[target_index as usize];

            if target_sq.color == start_sq.color {
                continue;
            }

            moves.push(Move {
                from: loc,
                to: target_loc,
                set_enpassant: (false, 0, 0),
            });
        }
        
        return moves;
    }
    
    fn get_king_squares(&self, loc: (i16, i16))->Vec<Move> {
        let start_index: i16 = loc.0 * 8 + loc.1;
        let start_sq = self.squares[start_index as usize];
        let mut target_sq: Square;
        let mut target_index: i16;
        let mut target_loc: (i16, i16) = (0,0);
        let mut moves: Vec<Move> = Vec::new();
        let mut index_horiz_shift: i16;
        let mut dist_closest_edge: i16;
        let mut newmove: Move;

        if self.squares[start_index as usize].color == Color::Empty {
            return moves;
        }
    
        for inc in [-9,-8,-7,-1,1,7,8,9] { // all king moves
            target_index = start_index + inc;
            target_loc = (target_index >> 3, target_index - (target_index & 0x7ff8));

            if ((target_loc.1 - loc.1).abs() > 1)
            || (target_index < 0)
            || (target_index >= 64) {
                continue;
            }

            target_sq = self.squares[target_index as usize];

            if target_sq.color == start_sq.color {
                continue;
            }

            moves.push(Move{
                from: loc,
                to: target_loc,
                set_enpassant: (false, 0, 0),
            });
        }
        
        return moves;
    }

    fn get_pawn_squares(&self, loc: (i16, i16))->Vec<Move> {
        let start_index: i16 = loc.0 * 8 + loc.1;
        let start_sq = self.squares[start_index as usize];
        let mut target_sq: Square;
        let mut target_index: i16;
        let mut target_loc: (i16, i16) = (0,0);
        let mut moves: Vec<Move> = Vec::new();
        let mut index_horiz_shift: i16;
        let mut dist_closest_edge: i16;
        let mut newmove: Move;
        let mut double_advance: bool = false;
        let mut pass_enpassant: bool = false;

        let direction: i16 = match self.squares[start_index as usize].color{
            Color::White => -1,
            Color::Black => 1,
            Color::Empty => 0,
        };

        if !direction {
            return moves;
        }

        if (self.squares[start_index as usize].color == Color::White && loc.1 == 6)
        || (self.squares[start_index as usize].color == Color::Black && loc.1 == 1) {
            double_advance = true;
        }
        
        target_index = start_index + 8 * direction;
        if target_index < 64 && target_index >= 0 && self.squares[target_index as usize].color == Color::Empty{ // pawn can move forward
            moves.push(Move{
                from: start_index,
                to: target_index,
                set_enpassant: (false, 0, 0),
            });

            target_index += 8*direction;
            if double_advance && self.squares[target_index as usize].color == Color::Empty { // We can double advance
                if (start_index % 8) != 7{
                    if self.board.squares[target_index+1].color != Color::Empty && self.board.squares[target_index+1] != self.board.squares[start_index].color {
                        moves.push(Move{
                            from: start_index,
                            to: target_index,
                            set_enpassant: (true, (target_index-8*direction)%8, (target_index-8*direction)/8),
                        });
                    }
                } else if start_index % 8 != 0 {
                    if self.board.squares[target_index-1].color != Color::Empty && self.board.squares[target_index-1] != self.board.squares[start_index].color {
                        moves.push(Move{
                            from: start_index,
                            to: target_index,
                            set_enpassant: (true, (target_index-8*direction)%8, (target_index-8*direction)/8),
                        });
                } else {
                    moves.push(Move{
                        from: start_index,
                        to: target_index,
                        set_enpassant: (false, 0, 0),
                    });
                }
            }
        }

        moves
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

    board = Board::from_fen("r1n1kn1r/bP1bqpP1/NB1nq1BN/Qq2n1qQ/1N4N1/2Q2Q2/3RR3/3K4 w - - 0 1").unwrap();
    println!("board has been initialized from FEN string: {}\n", Board::START_FEN);
    println!("{}", board);
}
