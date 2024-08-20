use std::fmt;
//use std::str;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const PIECE_MAP: [char; 7] = ['.', 'P', 'R', 'N', 'B', 'Q', 'K'];
macro_rules! CORRUPT_BOARD_PANIC_MSG{()=>("board hash tables corrupted, bailing...")}

#[derive(Copy,Clone,Eq,PartialEq,Hash,Default)]
pub enum Color {
    #[default] White,
               Black,
}

#[derive(Copy,Clone,Eq,Hash,PartialEq,Default)]
pub enum PieceType {
    #[default] Empty,
               Pawn,
               Rook,
               Knight,
               Bishop,
               Queen,
               King,
}

#[derive(Copy,Clone,Eq,PartialEq,Default)]
pub enum GameResult {
    #[default] Active,
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

#[derive(Default,Copy,Clone,Eq,PartialEq)]
pub struct Square {
    pub color: Color,
    pub piece: PieceType,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MoveOp {
    from: usize,
    to:   usize,
    is_enpassant: bool,
    is_castle: bool,
    set_enpassant: (bool, usize),
    promote: PieceType,
}

impl Default for MoveOp {
    fn default() -> Self {
        Self {
            from: 0,
            to: 0,
            is_enpassant: false,
            is_castle: false,
            set_enpassant: (false, 0),
            promote: PieceType::Empty,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub squares: Vec<Square>,
    pub shape: (usize, usize), // (height, width)
    pub piece_map: HashMap<PieceType, Vec<usize>>,
    pub to_play: Color,
    pub castling: ((bool, bool), (bool, bool)), // KQkq
    pub en_passant: (bool,usize), // flag, coords behind pawn to be captured
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
    pub result: GameResult,
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
                                        ((false, false), (false, false)) => "----",
                                        ((false, false), (false, true))  => "---q",
                                        ((false, false), (true,  false)) => "--k-",
                                        ((false, false), (true,  true))  => "--kq",
                                        ((false, true),  (false, false)) => "-Q--",
                                        ((false, true),  (false, true))  => "-Q-q",
                                        ((false, true),  (true,  false)) => "-Qk-",
                                        ((false, true),  (true,  true))  => "-Qkq",
                                        ((true,  false), (false, false)) => "K---",
                                        ((true,  false), (false, true))  => "K--q",
                                        ((true,  false), (true,  false)) => "K-k-",
                                        ((true,  false), (true,  true))  => "K-kq",
                                        ((true,  true),  (false, false)) => "KQ--",
                                        ((true,  true),  (false, true))  => "KQ-q",
                                        ((true,  true),  (true,  false)) => "KQk-",
                                        ((true,  true),  (true,  true))  => "KQkq",
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

    pub fn from_fen(fen_string: &str)->Result<Board, i16> {
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
            new_board.castling.0.0 = true;
        }

        if castling.contains('Q') {
            new_board.castling.0.1 = true;
        }

        if castling.contains('k') {
            new_board.castling.1.0 = true;
        }

        if castling.contains('q') {
            new_board.castling.1.1 = true;
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

    fn get_table(&self, p: PieceType) -> Vec<usize>{
        match self.piece_map.get(&p){
            Some(l) => l.clone(),
            None => panic!(CORRUPT_BOARD_PANIC_MSG!()),
        }
    }

    fn get_table_colored(&self, p: PieceType, c: Color) -> Vec<usize> {
        self.get_table(p).into_iter().filter(|&m| self.squares[m].color == c).collect()
    }
    
    fn get_mut_table(&mut self, p: PieceType) -> &mut Vec<usize>{
        match self.piece_map.get_mut(&p){
            Some(p) => p,
            None => panic!(CORRUPT_BOARD_PANIC_MSG!()), 
        }
    }

    /* TODO! This is hard... We need to filter, but preserve the mutable reference.
    fn get_mut_table_colored(&self, p: PieceType) -> &Vec<usize> {
        self.get_mut_table(p).into_iter().filter(|&m| self.squares[m].color == self.to_play).collect()
    }  */

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

        let mut capture: bool = false;
        
        from_table[from_index] = moveop.to;
    
        if self.squares[moveop.to].piece != PieceType::Empty { // remove a captured piece from the hash table
            capture = true;
            let to_table = self.get_mut_table(self.squares[moveop.to].piece);

            let to_index = Self::get_table_index(to_table, moveop.to);

            to_table.remove(to_index);
        }

        // deal with en passant...
        if moveop.is_enpassant {
            capture = true;
            let backwards_dir: i16 = match self.squares[from_index].color {
                Color::White =>  1,
                Color::Black => -1,
            };

            let target_pawn_index = (moveop.to as i16 + backwards_dir * self.shape.1 as i16) as usize;

            let to_table = self.get_mut_table(PieceType::Pawn);
            let to_index = Self::get_table_index(to_table, target_pawn_index);

            to_table.remove(to_index);
        }

        if moveop.set_enpassant.0 {
            self.en_passant = (true, moveop.set_enpassant.1);
        } else {
            self.en_passant = (false, 0);
        }

        // deal with castling...
        if self.squares[from_index].piece == PieceType::Rook {
            let castle: &mut (bool, bool) = match self.squares[from_index].color {
                Color::White => &mut self.castling.0,
                Color::Black => &mut self.castling.1,
            };

            if castle.0 && (from_index % self.shape.1 == self.shape.1 - 1){ // king side
                castle.0 = false;
            } else if castle.1 && (from_index % self.shape.1 == 0) { // queen side
                castle.1 = false;
            }

        } else if self.squares[from_index].piece == PieceType::King {
            if moveop.is_castle {
                // Create a secondary move that isn't a castle, but moves the rook to where it needs to go
                let castle_from_index: usize;
                let castle_to_index: usize;

                if (moveop.from as i16) - (moveop.to as i16) > 0 { // queen side
                    castle_from_index = moveop.from - 4;
                    castle_to_index = moveop.to + 1;
                } else { // king side
                    castle_from_index = moveop.from + 3;
                    castle_to_index = moveop.to - 1;
                }

                self.apply_move(MoveOp {
                    from: castle_from_index,
                    to: castle_to_index,
                    ..Default::default()
                })
            }
            
            if self.squares[from_index].color == Color::White {
                self.castling.0 = (false, false);
            } else {
                self.castling.1 = (false, false);
            }
        }

        // deal with 50 move rule...
        if capture || self.squares[from_index].piece == PieceType::Pawn {
            self.halfmove_clock = 50;
        } else {
            self.halfmove_clock -= 1;
        }

        if self.halfmove_clock == 0 {
            self.result = GameResult::Draw50Moves;
        }

        self.squares[moveop.to] = self.squares[moveop.from];
        self.squares[moveop.from].piece = PieceType::Empty;

        self.to_play = match self.to_play {
            Color::Black => Color::White,
            Color::White => Color::Black,
        };

        if self.to_play == Color::White {
            self.fullmove_number += 1;
        }
    }

    pub fn apply_move_nomut(&self, moveop: MoveOp) -> Self {
        let mut child: Self = self.clone();
        child.apply_move(moveop);

        child
    }

    fn get_sliding_moves_single(&self, piece: PieceType, start_index: usize)->Vec<MoveOp> {
        let start_sq = self.squares[start_index];
        let mut moves: Vec<MoveOp> = Vec::new();

        let mut index: i16 = 0;
        
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
            let mut eob_flag: bool = false;
            loop {
                index += inc;
                let target_index = ((start_index as i16) + index) as usize;

                if target_index >= self.shape.0 * self.shape.1 || eob_flag {
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
                    ..Default::default()
                };

                if (target.color != start_sq.color) && (target.piece != PieceType::Empty) {
                    moves.push(newmove);
                    break;
                }
                index = 0;
            }
            index = 0;
        }

        moves
    }
    
    fn get_sliding_moves(&self, piece: PieceType)->Vec<MoveOp> {
        let indices: Vec<usize> = self.get_table_colored(piece, self.to_play);
        let mut moves: Vec<MoveOp> = Vec::new();


        for start_index in indices {
            moves.append(&mut self.get_sliding_moves_single(piece, start_index));
        }

        moves
    }

    fn get_knight_moves_single(&self, start_index: usize)->Vec<MoveOp> {
        let mut moves: Vec<MoveOp> = Vec::new();
        let start_sq = self.squares[start_index as usize];
        let mut target_sq: Square;
        let mut index_horiz_shift: i16;
        let mut dist_closest_edge: i16;
        let incs: Vec<i16> = vec![-10, -6, -17, -15, 6, 10, 16, 17];
        let loc = ((start_index as i16) >> 3, (start_index as i16) - ((start_index as i16) & 0x7ff8));
    
        for inc in incs { // all knight moves
            let target_index = ((start_index as i16) + inc) as usize;
            let target_loc = ((target_index as i16) >> 3, (target_index as i16) - ((target_index as i16) & 0x7ff8));
            index_horiz_shift = target_loc.1 - loc.1;

            if loc.1 < 4 {
                dist_closest_edge = loc.1;
            } else {
                dist_closest_edge = 8 - loc.1;
            }
            
            if target_index >= self.shape.0 * self.shape.1
            || index_horiz_shift.abs() > dist_closest_edge {
                continue;
            }

            target_sq = self.squares[target_index as usize];

            if target_sq.color == start_sq.color {
                continue;
            }

            moves.push(MoveOp {
                from: start_index,
                to: target_index,
                ..Default::default()
            });
        }

        moves
    }

    fn get_knight_moves(&self)->Vec<MoveOp> {
        let indices = self.get_table_colored(PieceType::Knight, self.to_play);
        let mut moves: Vec<MoveOp> = Vec::new();

        for start_index in indices {
            moves.append(&mut self.get_knight_moves_single(start_index));
        }
        
        moves
    }
    
    fn get_king_moves(&self)->Vec<MoveOp> {
        let indices = self.get_table_colored(PieceType::King, self.to_play);
        let mut moves: Vec<MoveOp> = Vec::new();
        for start_index in indices {
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
                    ..Default::default()
                });
            }
        }
        
        moves
    }

    fn get_pawn_moves_single(&self, start_index: usize, c: Color)->Vec<MoveOp> {
        let mut moves: Vec<MoveOp> = Vec::new();

        let direction: i16 = match c {
            Color::White => -1,
            Color::Black =>  1,
        };

        let advance1: usize = start_index + (direction * self.shape.1 as i16) as usize;
        
        if self.squares[advance1].piece == PieceType::Empty {
            moves.push(MoveOp {
                from: start_index,
                to: advance1,
                ..Default::default()
            });

            let advance2: usize = start_index + (2 * direction * self.shape.1 as i16) as usize;

            if self.squares[advance2].piece == PieceType::Empty {
                moves.push(MoveOp {
                    from: start_index,
                    to: advance2,
                    set_enpassant: (true, advance1),
                    ..Default::default()
                });
            }
        }

        let mut attack_indices: Vec<usize> = Vec::new();

        if start_index % self.shape.1 != 0 {
            attack_indices.push(start_index + (direction * self.shape.1 as i16) as usize - 1);
        }

        if start_index % self.shape.1 != self.shape.1 - 1 {
            attack_indices.push(start_index + (direction * self.shape.1 as i16) as usize + 1);
        }

        for index in attack_indices {
            if self.squares[index].piece != PieceType::Empty && self.squares[index].color != c{
                moves.push(MoveOp {
                    from: start_index,
                    to: index,
                    ..Default::default()
                });
            } 

            if self.en_passant.0 && index == self.en_passant.1 {
                moves.push(MoveOp{
                    from: start_index,
                    to: index,
                    is_enpassant: true,
                    ..Default::default()
                })
            }
        }

        moves
    }

    fn get_pawn_moves(&self)->Vec<MoveOp> {
        let indices = self.get_table_colored(PieceType::Pawn, self.to_play);
        let mut moves: Vec<MoveOp> = Vec::new();
        for start_index in indices {
            moves.append(&mut self.get_pawn_moves_single(start_index, self.to_play));
        }

        moves
    }


    fn get_all_moves(&self) -> Vec<MoveOp> {
        let mut moves: Vec<MoveOp> = Vec::new();
        moves.extend(self.get_king_moves());
        moves.extend(self.get_sliding_moves(PieceType::Queen));
        moves.extend(self.get_sliding_moves(PieceType::Bishop));
        moves.extend(self.get_sliding_moves(PieceType::Rook));
        moves.extend(self.get_knight_moves());

        moves
    }

    fn get_legal_moves(&self) -> Vec<MoveOp> {
        let candidates = self.get_all_moves();
        let mut moves: Vec<MoveOp> = Vec::new();
        for m in &candidates {
            let newboard = self.apply_move_nomut(*m);
            let kingloc = newboard.get_table_colored(PieceType::King, self.to_play)[0];
            if !newboard.get_all_moves().into_iter().map(|m| m.to).any(|i| i == kingloc){
                moves.push(*m);
            }
        }

        moves
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            squares: vec![Square::default(); 64],
            shape: (8, 8),
            piece_map: HashMap::new(),
            to_play: Color::White,
            castling: ((false, false), (false, false)),
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

#[cfg(test)]
mod tests {

    use crate::board::*;
    #[test]
    fn board_test() {
        let mut board: Board = Board::default();
        println!("ahhh yes... chess.");
        println!("{}\n", board);

        board = Board::from_fen(START_FEN).unwrap();
        println!("board has been initialized from FEN string: {}\n", crate::board::START_FEN);
        println!("{}\n", board);

        // Play the ruy...
        board.apply_move(MoveOp{from: 52, to: 36, is_enpassant: false, is_castle: false, set_enpassant: (false, 0), promote: PieceType::Empty});
        board.apply_move(MoveOp{from: 12, to: 28, is_enpassant: false, is_castle: false, set_enpassant: (false, 0), promote: PieceType::Empty});
        board.apply_move(MoveOp{from: 62, to: 45, is_enpassant: false, is_castle: false, set_enpassant: (false, 0), promote: PieceType::Empty});
        board.apply_move(MoveOp{from: 1, to: 18, is_enpassant: false, is_castle: false, set_enpassant: (false, 0), promote: PieceType::Empty});
        board.apply_move(MoveOp{from: 61, to: 25, is_enpassant: false, is_castle: false, set_enpassant: (false, 0), promote: PieceType::Empty});

        println!("{}", board);
    }
}
