use std::fmt;
//use std::str;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const PIECE_MAP: [char; 7] = ['.', 'P', 'R', 'N', 'B', 'Q', 'K'];
macro_rules! CORRUPT_BOARD_PANIC_MSG{()=>("board hash tables corrupted, bailing...")}

#[derive(Copy,Clone,Eq,PartialEq)]
enum Color {
    White,
    Black,
}

impl Default for Color {
    fn default() -> Self { Color::White } // Used to have Color::Empty, but I think it makes more sense for that to live in PieceType...
}

#[derive(Copy,Clone,Eq,Hash,PartialEq)]
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

#[derive(Copy,Clone,Eq,PartialEq)]
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

#[derive(Default,Copy,Clone,Eq,PartialEq)]
struct Square {
    color: Color,
    piece: PieceType,
}

struct MoveOp {
    from: usize,
    to:   usize,
    set_enpassant: (bool, usize),
}

#[derive(Clone)]
struct Board {
    squares: Vec<Square>,
    shape: (usize, usize), // (height, width)
    piece_map: HashMap<PieceType, Vec<usize>>,
    to_play: Color,
    castling: (bool, bool, bool, bool), // KQkq
    en_passant: (bool,usize), // flag, coords behind pawn to be captured
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
        let height: usize = self.shape.0;
        let width: usize = self.shape.1;
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

    fn alg_to_index(&self, alg_notation: &str)->usize{
        let c_str = alg_notation.as_bytes();
        let file = (c_str[0] - 48) as usize;
        let rank = (c_str[1] - 48) as usize;
        
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

        new_board.populate_map();

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

    fn search_piece(&self, p: PieceType) -> Vec<usize>{
        self.squares.iter().enumerate().filter_map(|s| {
            if p == s.1.piece {
                Some(s.0 as usize)
            } else {
                None
            }
        }).collect::<Vec<_>>()
    }

    fn get_table(&self, p: PieceType) -> &Vec<usize>{
        match self.piece_map.get(&p){
            Some(p) => p,
            None => panic!(CORRUPT_BOARD_PANIC_MSG!()),
        }
    }
    
    fn get_mut_table(&mut self, p: PieceType) -> &mut Vec<usize>{
        match self.piece_map.get_mut(&p){
            Some(p) => p,
            None => panic!(CORRUPT_BOARD_PANIC_MSG!()),
        }
    }

    fn get_table_index(table: &Vec<usize>, val: usize) -> usize {
        match table.iter().position(|&r| r == val){
            Some(x) => x,
            None => panic!(CORRUPT_BOARD_PANIC_MSG!()),
        }
    }

    fn populate_map(&mut self) {
        self.piece_map = HashMap::from([
            (PieceType::King, self.search_piece(PieceType::King)),
            (PieceType::Queen, self.search_piece(PieceType::Queen)),
            (PieceType::Bishop, self.search_piece(PieceType::Bishop)),
            (PieceType::Knight, self.search_piece(PieceType::Knight)),
            (PieceType::Rook, self.search_piece(PieceType::Rook)),
            (PieceType::Pawn, self.search_piece(PieceType::Pawn)),
        ]);
    }

    fn apply_move(&mut self, moveop: MoveOp){
        let from_table = self.get_mut_table(self.squares[moveop.from].piece);

        let from_index = Self::get_table_index(from_table, moveop.from);
        
        from_table[from_index] = moveop.to;
    
        if self.squares[moveop.to].piece != PieceType::Empty { // remove a captured piece from the hash table
            let to_table = self.get_mut_table(self.squares[moveop.to].piece);

            let to_index = Self::get_table_index(to_table, moveop.to);

            to_table.remove(to_index);
        }

        self.squares[moveop.to] = self.squares[moveop.from];
        self.squares[moveop.from].piece = PieceType::Empty;
    }

    fn apply_move_nomut(&self, moveop: MoveOp) -> Self {
        let mut child: Self = self.clone();
        child.apply_move(moveop);

        child
    }

    fn get_sliding_squares(&self, piece: PieceType)->Vec<MoveOp> {
        let indices: &Vec<usize> = self.get_table(piece);
        let mut moves: Vec<MoveOp> = Vec::new();

        let height = self.shape.0;
        let width = self.shape.1;

        for &start_index in indices {
            let start_sq = self.squares[start_index];

            let mut index: i16 = 0;
            let mut eob_flag: bool = false;
            
            let mut target: Square;

            let mut incs: Vec<i16> = Vec::new();
            let mut newmove: MoveOp;
            let rook_incs: Vec<i16> = vec![8, -8, 1, -1];
            let bishop_incs: Vec<i16> = vec![9, 7, -7, -9];

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
                    let target_index = ((start_index as i16) + index) as usize;

                    if      (target_index < 0) 
                         || (target_index >= self.shape.0 * self.shape.1)
                         || eob_flag {
                             break;
                         }

                    if target_index % self.shape.1 == 0|| target_index % self.shape.1 == self.shape.1 - 1 {
                        eob_flag = true;
                    }
                    
                    target = self.squares[target_index];
                    
                    if target.color == start_sq.color {
                        break;
                    }

                    newmove = MoveOp {
                        from: start_index,
                        to: target_index,
                        set_enpassant: (false, 0),
                    };

                    if (target.color != start_sq.color) && (target.piece != PieceType::Empty) {
                        moves.push(newmove);
                        break;
                    }
                    index = 0;
                }
                index = 0;
            }
        }

        moves
    }

    /*
    fn get_knight_squares(&self, loc: (i16, i16))->Vec<MoveOp> {
        let start_index: i16 = loc.0 * 8 + loc.1;
        let start_sq = self.squares[start_index as usize];
        let mut target_sq: Square;
        let mut target_index: i16;
        let mut target_loc: (i16, i16) = (0,0);
        let mut moves: Vec<Move> = Vec::new();
        let mut index_horiz_shift: i16;
        let mut dist_closest_edge: i16;
        let mut newmove: Move;
        
        if self.squares[start_index as usize].piece == PieceType::Empty {
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
    */
    
    fn get_king_squares(&self, loc: (i16, i16))->Vec<MoveOp> {
        let indices = self.get_table(PieceType::King);
        let mut moves: Vec<MoveOp> = Vec::new();
        for &start_index in indices {
            let mut newmove: MoveOp;

            let start_sq = self.squares[start_index];
            let incs: Vec<i16> = vec![-9, -8, -7, -1, 1, 7, 8, 9];
        
            for inc in incs { // all king moves
                let target_index = ((start_index as i16) + inc) as usize;
                let target_loc: (i16, i16) = ((target_index as i16) >> 3, (target_index as i16) - ((target_index as i16) & 0x7ff8));
                let loc: (i16, i16) = ((start_index as i16) >> 3, (target_index as i16) - ((target_index as i16) & 0x7ff8));

                if ((target_loc.1 - loc.1).abs() > 1)
                || (target_index >= self.shape.0 * self.shape.1) {
                    continue;
                }

                let target_sq = self.squares[target_index];

                if target_sq.color == start_sq.color {
                    continue;
                }

                moves.push(MoveOp{
                    from: start_index,
                    to: target_index,
                    set_enpassant: (false, 0),
                });
            }
        }
        
        moves
    }

    /*
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
    */
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
