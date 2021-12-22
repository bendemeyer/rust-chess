pub mod squares;

use fnv::{FnvHashMap, FnvHashSet};

use crate::util::FnvIndexSet;
use crate::util::errors::InputError;
use crate::util::fen::{FenBoardState, Castling, STARTING_POSITION, get_notation_for_piece};

use self::squares::{BoardSquare, square_in_row, BoardRow, get_col_and_row_from_square, get_square_from_col_and_row};

use super::Color;
use super::pieces::{Piece, PieceType, PIECE_MOVE_VECTOR_MAP, UNMOVED_WHITE_PAWN_ADVANCING_VECTORS, MOVED_WHITE_PAWN_ADVANCING_VECTORS, UNMOVED_BLACK_PAWN_ADVANCING_VECTORS, MOVED_BLACK_PAWN_ADVANCING_VECTORS, WHITE_PAWN_ATTACKING_VECTORS, BLACK_PAWN_ATTACKING_VECTORS};
use super::pieces::movement::{BasicMove, Castle, CastleType, EnPassant, HasMove, Move, Movement, MovementDetail, MovementVector, PawnMovementDetail, PieceMovementDetail, Promotion, TwoSquarePawnMove, build_movement_detail, build_pawn_movement_detail};


lazy_static! {
    pub static ref BOARD_PIECE_MOVES: FnvHashMap<u8, FnvHashMap<PieceType, PieceMovementDetail>> = FnvHashMap::from_iter((0u8..=63u8).into_iter().map(|square| {
        (square, FnvHashMap::from_iter(PIECE_MOVE_VECTOR_MAP.iter().map(|(ptype, vectors)| {
            (*ptype, build_movement_detail(square, vectors))
        })))
    }));

    static ref ALL_PIECE_MOVES: FnvHashMap<u8, PieceMovementDetail> = BOARD_PIECE_MOVES.iter().map(|(square, piece_map)| {
        (*square, PieceMovementDetail::from_static_vectors([
            piece_map.get(&PieceType::Queen).unwrap().rays.clone(),
            piece_map.get(&PieceType::Knight).unwrap().rays.clone(),
        ].into_iter().flatten().collect()))
    }).collect();

    static ref BOARD_PAWN_MOVES: FnvHashMap<u8, FnvHashMap<Color, PawnMovementDetail>> = FnvHashMap::from_iter((0u8..=63u8).into_iter().map(|square| {
        (square, FnvHashMap::from_iter([Color::White, Color::Black].into_iter().map(|color| {
            (color, match pawn_square_is_invalid(square) {
                true => PawnMovementDetail::default(),
                false => build_pawn_movement_detail(
                    square,
                    &get_pawn_vectors(color, square, false),
                    &get_pawn_vectors(color, square, true),
                )
            })
        })))
    }));

    static ref CASTLING_MOVES: FnvHashMap<Color, FnvHashMap<CastleType, CastlingSquares>> = FnvHashMap::from_iter([
        (Color::White, FnvHashMap::from_iter([
            (CastleType::Kingside, CastlingSquares::from_color_and_type(Color::White, CastleType::Kingside)),
            (CastleType::Queenside, CastlingSquares::from_color_and_type(Color::White, CastleType::Queenside))
        ].into_iter())),
        (Color::Black, FnvHashMap::from_iter([
            (CastleType::Kingside, CastlingSquares::from_color_and_type(Color::Black, CastleType::Kingside)),
            (CastleType::Queenside, CastlingSquares::from_color_and_type(Color::Black, CastleType::Queenside))
        ].into_iter())),
    ].into_iter());
}


fn pawn_square_is_invalid(square: u8) -> bool {
    square_in_row(&square, BoardRow::Row1) || square_in_row(&square, BoardRow::Row8)
}

fn pawn_square_is_starting(color: Color, square: u8) -> bool {
    return match color {
        Color::White => square_in_row(&square, BoardRow::Row2),
        Color::Black => square_in_row(&square, BoardRow::Row7),
    }
}

fn pawn_square_is_fourth_rank(color: Color, square: u8) -> bool {
    return match color {
        Color::White => square_in_row(&square, BoardRow::Row4),
        Color::Black => square_in_row(&square, BoardRow::Row5),
    }
}

fn pawn_square_is_promotion(color: Color, square: u8) -> bool {
    return match color {
        Color::White => square_in_row(&square, BoardRow::Row8),
        Color::Black => square_in_row(&square, BoardRow::Row1),
    }
}

fn get_capture_square_for_en_passant(start: u8, end: u8) -> u8 {
    let [_start_col, start_row] = get_col_and_row_from_square(start);
    let [end_col, _end_row] = get_col_and_row_from_square(end);
    return get_square_from_col_and_row(end_col, start_row)
}

fn get_en_passant_target_for_two_square_first_move(color: Color, square: u8) -> u8 {
    let [col, row] = get_col_and_row_from_square(square);
    let direction: i8 = match color { Color::White => -1, Color::Black => 1 };
    return get_square_from_col_and_row(col, (row as i8 + direction) as u8)
}

fn get_pawn_vectors(color: Color, square: u8, attacking: bool) -> Vec<&'static MovementVector> {
    if pawn_square_is_invalid(square) {
        return Vec::new();
    }
    let is_start = pawn_square_is_starting(color, square);
    return match attacking {
        true => match color {
            Color::White => WHITE_PAWN_ATTACKING_VECTORS.iter().collect(),
            Color::Black => BLACK_PAWN_ATTACKING_VECTORS.iter().collect(),
        },
        false => match color {
            Color::White => (match is_start { true => UNMOVED_WHITE_PAWN_ADVANCING_VECTORS.iter(), false => MOVED_WHITE_PAWN_ADVANCING_VECTORS.iter() }).collect(),
            Color::Black => (match is_start { true => UNMOVED_BLACK_PAWN_ADVANCING_VECTORS.iter(), false => MOVED_BLACK_PAWN_ADVANCING_VECTORS.iter() }).collect(),
        }
    }
}


fn piece_map_from_fen_board(board: [[Option<(Color, PieceType)>; 8]; 8]) -> FnvHashMap<u8, Piece> {
    let mut piece_map: FnvHashMap<u8, Piece> = Default::default();
    board.into_iter().rev().enumerate().for_each(|(row_index, row)| {
        row.into_iter().enumerate().for_each(|(col_index, option)| {
            match option {
                None => (),
                Some((color, piece)) => {
                    piece_map.insert((col_index + (row_index * 8)) as u8, Piece { color: color, piece_type: piece });
                    ();
                }
            }
        })
    });
    return piece_map;
}

fn print_fen_board(board: [[Option<(Color, PieceType)>; 8]; 8]) {
    board.iter().for_each(|row| {
        println!("{}", row.iter().map(|square| {
            match square {
                Some((c, p)) => get_notation_for_piece(*c, *p).to_string(),
                None => String::from("-")
            }
        }).collect::<Vec<String>>().join(" "))
    })
}

pub fn fen_board_from_piece_map(piece_map: &FnvHashMap<u8, Piece>) -> [[Option<(Color, PieceType)>; 8]; 8] {
    let mut board: [[Option<(Color, PieceType)>; 8]; 8] = Default::default();
    (0u8..=7u8).rev().enumerate().for_each(|(row, index )| {
        (0u8..=7u8).for_each(|col| {
            board[index as usize][col as usize] = match piece_map.get(&(col + ((row as u8) * 8))) { Some(p) => Some((p.color, p.piece_type)), None => None }
        })
    });
    return board;
}

fn board_from_fen_state(state: FenBoardState) -> Board {
    let piece_map = piece_map_from_fen_board(state.board);
    return Board {
        piece_locations: BoardLocations::from_piece_map(piece_map.clone()),
        state: BoardState {
            to_move: state.to_move,
            en_passant_target: match state.en_passant { Some(s) => Some(s.value()), None => None },
            halfmove_clock: state.halfmove_timer,
            move_number: state.move_number,
            castle_availability: BoardCastles {
                white_kingside: state.castling.white_kingside,
                white_queenside: state.castling.white_queenside,
                black_kingside: state.castling.black_kingside,
                black_queenside: state.castling.black_queenside,
            }
        },
    }
}

fn fen_state_from_board(board: &Board) -> FenBoardState {
    return FenBoardState {
        board: fen_board_from_piece_map(board.get_piece_map()),
        to_move: board.state.to_move,
        castling: Castling {
            white_kingside: board.state.castle_availability.white_kingside,
            white_queenside: board.state.castle_availability.white_queenside,
            black_kingside: board.state.castle_availability.black_kingside,
            black_queenside: board.state.castle_availability.black_queenside,
        },
        en_passant: match board.state.en_passant_target { Some(n) => Some(BoardSquare::from_value(n)), None => None },
        halfmove_timer: board.state.halfmove_clock,
        move_number: board.state.move_number,
    }
}

fn parse_checks_and_pins(checks_and_pins: Vec<CheckOrPin>) -> (Vec<Check>, Vec<Pin>) {
    checks_and_pins.into_iter().fold((Vec::new(), Vec::new()), |(mut checks, mut pins), check_or_pin| {
        match check_or_pin {
            CheckOrPin::Check(c) => checks.push(c),
            CheckOrPin::Pin(p) => pins.push(p)
        }
        (checks, pins)
    })
}

fn get_moves(square: &u8, piece: &Piece) -> MovementDetail {
    let error_msg = format!(
        "Error getting movement detail for {} {} on {}",
        piece.color.value(),
        piece.piece_type.value(),
        BoardSquare::from_value(*square).get_notation_string()
    );
    return match piece.piece_type {
        PieceType::Pawn => {
            MovementDetail::Pawn(&BOARD_PAWN_MOVES.get(square).expect(&error_msg).get(&piece.color).expect(&error_msg))
        },
        _ => {
            MovementDetail::Piece(&BOARD_PIECE_MOVES.get(square).expect(&error_msg).get(&piece.piece_type).expect(&error_msg))
        }
    }
}

fn get_all_moves(square: &u8) -> MovementDetail {
    return MovementDetail::Piece(ALL_PIECE_MOVES.get(square).unwrap());
}

fn create_promotions(start: u8, end: u8, capture: Option<Piece>) -> Vec<Move> {
    return [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight].into_iter().fold(Vec::new(), |mut moves, ptype| {
        moves.push(Move::Promotion(Promotion { basic_move: BasicMove { start: start, end: end, capture }, promote_to: ptype }));
        moves
    });
}

fn get_castle_details(color: Color, castle_type: CastleType) -> &'static CastlingSquares {
    return CASTLING_MOVES.get(&color).unwrap().get(&castle_type).unwrap()
}

fn get_castle_for_rook_position(square: u8) -> Option<(Color, CastleType)> {
    return match square {
        0  => Some((Color::White, CastleType::Queenside)),
        7  => Some((Color::White, CastleType::Kingside)),
        56 => Some((Color::Black, CastleType::Queenside)),
        63 => Some((Color::Black, CastleType::Kingside)),
        _  => None
    }
}


enum CheckOrPin {
    Check(Check),
    Pin(Pin),
}

pub struct Check {
    pub checking_square: u8,
    pub blocking_squares: FnvIndexSet<u8>,
}

pub struct Pin {
    pub pinned_square: u8,
    pub pinning_square: u8,
    pub pinning_path: FnvIndexSet<u8>,
}


#[derive(Clone, Debug)]
pub struct CastlingSquares {
    pub king_start: u8,
    pub king_end: u8,
    pub rook_start: u8,
    pub rook_end: u8,
    pub transit_squares: Vec<u8>,
    pub king_transit_squares: Vec<u8>
}

impl CastlingSquares {
    pub fn from_color_and_type(color: Color, ctype: CastleType) -> CastlingSquares {
        return match (color, ctype) {
            (Color::White, CastleType::Kingside) => {
                 CastlingSquares {
                     king_start: BoardSquare::E1.value(), king_end: BoardSquare::G1.value(),
                     rook_start: BoardSquare::H1.value(), rook_end: BoardSquare::F1.value(),
                     transit_squares: Vec::from([BoardSquare::F1.value(), BoardSquare::G1.value()]),
                     king_transit_squares: Vec::from([BoardSquare::F1.value()])
                 }
            },
            (Color::White, CastleType::Queenside) => {
                CastlingSquares {
                    king_start: BoardSquare::E1.value(), king_end: BoardSquare::C1.value(),
                    rook_start: BoardSquare::A1.value(), rook_end: BoardSquare::D1.value(),
                    transit_squares: Vec::from([BoardSquare::D1.value(), BoardSquare::C1.value(), BoardSquare::B1.value()]),
                    king_transit_squares: Vec::from([BoardSquare::D1.value()])
                }
            },
            (Color::Black, CastleType::Kingside) => {
                CastlingSquares {
                    king_start: BoardSquare::E8.value(), king_end: BoardSquare::G8.value(),
                    rook_start: BoardSquare::H8.value(), rook_end: BoardSquare::F8.value(),
                    transit_squares: Vec::from([BoardSquare::F8.value(), BoardSquare::G8.value()]),
                    king_transit_squares: Vec::from([BoardSquare::F8.value()])
                }
            },
            (Color::Black, CastleType::Queenside) => {
                CastlingSquares {
                    king_start: BoardSquare::E8.value(), king_end: BoardSquare::C8.value(),
                    rook_start: BoardSquare::A8.value(), rook_end: BoardSquare::D8.value(),
                    transit_squares: Vec::from([BoardSquare::D8.value(), BoardSquare::C8.value(), BoardSquare::B8.value()]),
                    king_transit_squares: Vec::from([BoardSquare::D8.value()])
                }
            }
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub struct BoardCastles {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl Default for BoardCastles {
    fn default() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}


#[derive(Copy, Clone, Default)]
pub struct BoardState {
    to_move: Color,
    castle_availability: BoardCastles,
    en_passant_target: Option<u8>,
    move_number: u8,
    halfmove_clock: u8,
}

impl BoardState {
    pub fn reset_halfmove_clock(&mut self) {
        self.halfmove_clock = 0;
    }

    pub fn increment_halfmove_clock(&mut self) {
        self.halfmove_clock += 1;
    }

    pub fn increment_move_number(&mut self) {
        self.move_number += 1;
    }

    pub fn get_move_color(&self) -> Color {
        return self.to_move
    }

    pub fn change_move_color(&mut self) {
        self.to_move = self.to_move.swap();
    }

    pub fn clear_en_passant_target(&mut self) {
        self.en_passant_target = None;
    }

    pub fn set_en_passant_target(&mut self, square: u8) {
        self.en_passant_target = Some(square);
    }

    pub fn get_en_passant_target(&self) -> Option<u8> {
        return self.en_passant_target
    }

    pub fn can_castle(&self, color: Color, side: CastleType) -> bool {
        match (color, side) {
            (Color::White, CastleType::Kingside) => self.castle_availability.white_kingside,
            (Color::White, CastleType::Queenside) => self.castle_availability.white_queenside,
            (Color::Black, CastleType::Kingside) => self.castle_availability.black_kingside,
            (Color::Black, CastleType::Queenside) => self.castle_availability.black_queenside,
        }
    }

    pub fn disable_castle(&mut self, color: Color, side: CastleType) {
        match (color, side) {
            (Color::White, CastleType::Kingside) => self.castle_availability.white_kingside = false,
            (Color::White, CastleType::Queenside) => self.castle_availability.white_queenside = false,
            (Color::Black, CastleType::Kingside) => self.castle_availability.black_kingside = false,
            (Color::Black, CastleType::Queenside) => self.castle_availability.black_queenside = false,
        }
    }
}


#[derive(Clone, Default)]
pub struct BoardLocations {
    piece_map: FnvHashMap<u8, Piece>,
    all_white_pieces: FnvIndexSet<u8>,
    white_king: u8,
    all_black_pieces: FnvIndexSet<u8>,
    black_king: u8,
}

impl BoardLocations {
    pub fn from_piece_map(map: FnvHashMap<u8, Piece>) -> Self {
        let mut locs: Self = map.clone().into_iter().fold(Default::default(), |mut locs, (s, p)| {
            match (p.color, p.piece_type) {
                (Color::White, PieceType::King)   => { locs.all_white_pieces.insert(s); locs.white_king = s; },
                (Color::White, _)                 => { locs.all_white_pieces.insert(s); },
                (Color::Black, PieceType::King)   => { locs.all_black_pieces.insert(s); locs.black_king = s; },
                (Color::Black, _)                 => { locs.all_black_pieces.insert(s); },
            }
            locs
        });
        locs.piece_map = map.clone();
        return locs;
    }

    pub fn get_piece_map(&self) -> &FnvHashMap<u8, Piece> {
        return &self.piece_map;
    }

    pub fn find_king(&self, color: Color) -> u8 {
        return match color { Color::White => self.white_king, Color::Black => self.black_king }
    }

    pub fn piece_at(&self, square: &u8) -> Option<&Piece> {
        return self.piece_map.get(square);
    }

    pub fn all_pieces(&self, color: Color) -> &FnvIndexSet<u8> {
        match color { Color::White => &self.all_white_pieces, Color::Black => &self.all_black_pieces }
    }

    fn insert_piece(&mut self, square: u8, piece: Piece) {
        match piece.color {
            Color::White => self.all_white_pieces.insert(square),
            Color::Black => self.all_black_pieces.insert(square)
        };
        self.piece_map.insert(square, piece);
    }

    fn try_capture_piece(&mut self, square: u8) -> Option<Piece> {
        match self.piece_map.remove(&square) {
            Some(p) => match p.color {
                Color::White => { self.all_white_pieces.remove(&square); Some(p) },
                Color::Black => { self.all_black_pieces.remove(&square); Some(p) },
            }
            None => None
        }
    }

    fn move_piece(&mut self, start: u8, end: u8) -> Result<&Piece, InputError> {
        match self.piece_map.remove(&start) {
            None => return Err(InputError::new(&format!("No piece to move at square {}", start))),
            Some(p) => {
                self.piece_map.insert(end, p);
                match p.color {
                    Color::White => {
                        self.all_white_pieces.remove(&start);
                        self.all_white_pieces.insert(end);
                        if p.piece_type == PieceType::King { self.white_king = end };
                    },
                    Color::Black => {
                        self.all_black_pieces.remove(&start);
                        self.all_black_pieces.insert(end);
                        if p.piece_type == PieceType::King { self.black_king = end };
                    },
                }
            }
        };
        return Ok(self.piece_at(&end).unwrap())
    }

    fn change_piece_type(&mut self, square: u8, piece_type: PieceType) -> Result<&Piece, InputError> {
        match self.piece_map.get_mut(&square) {
            None => Err(InputError::new(&format!("No piece at square {}", square))),
            Some(p) => {
                p.piece_type = piece_type;
                Ok(p)
            }
        }
    }

    pub fn apply_move(&mut self, new_move: &Move) -> Result<(&Piece, Option<Piece>), InputError> {
        match new_move {
            Move::TwoSquarePawnMove(t) => {
                match self.move_piece(t.basic_move.start, t.basic_move.end) {
                    Err(e) => Err(e),
                    Ok(p) => Ok((p, None))
                }
            }
            Move::BasicMove(b) => {
                let captured_piece = self.try_capture_piece(b.end);
                match self.move_piece(b.start, b.end) {
                    Err(e) => Err(e),
                    Ok(p) => Ok((p, captured_piece))
                }
            },
            Move::Castle(c) => {
                match self.move_piece(c.rook_start, c.rook_end) {
                    Err(e) => return Err(e),
                    _ => ()
                };
                match self.move_piece(c.king_start, c.king_end) {
                    Err(e) => Err(e),
                    Ok(k) => Ok((k, None))
                }
            },
            Move::EnPassant(e) => {
                let captured_piece = self.try_capture_piece(e.capture_square);
                match self.move_piece(e.basic_move.start, e.basic_move.end) {
                    Err(e) => Err(e),
                    Ok(p) => Ok((p, captured_piece))
                }
            },
            Move::Promotion(p) => {
                let captured_piece = self.try_capture_piece(p.basic_move.end);
                match self.move_piece(p.basic_move.start, p.basic_move.end) {
                    Err(e) => Err(e),
                    Ok(_) => {
                        match self.change_piece_type(p.basic_move.end, p.promote_to) {
                            Err(e) => Err(e),
                            Ok(p) => Ok((p, captured_piece))
                        }
                    }
                }
            },
            Move::NewGame(_) => Err(InputError::new("Cannot apply move 'NewGame'"))
        }
    }

    pub fn unapply_move(&mut self, old_move: &Move) -> Result<(), InputError> {
        match old_move {
            Move::EnPassant(e) => {
                match old_move.get_capture() {
                    Some(p) => self.insert_piece(e.capture_square, p),
                    None => return Err(InputError::new("En Passant missing captured piece"))
                };
                match self.move_piece(e.basic_move.end, e.basic_move.start) {
                    Err(e) => return Err(e),
                    Ok(_) => ()
                }
            },
            Move::Promotion(p) => {
                self.change_piece_type(p.basic_move.end, PieceType::Pawn)?;
                match self.move_piece(p.basic_move.end, p.basic_move.start) {
                    Err(e) => return Err(e),
                    Ok(_) => ()
                }
            },
            _ => {
                for movement in old_move.get_piece_movements() {
                    self.move_piece(movement.end_square, movement.start_square)?;
                    match old_move.get_capture() {
                        Some(p) => self.insert_piece(movement.end_square, p),
                        None => ()
                    }
                }
            }
        }
        return Ok(());
    }
}


pub struct ReversibleBoardChange {
    pub move_made: Move,
    pub prior_state: BoardState,
}


#[derive(Clone)]
pub struct Board {
    pub piece_locations: BoardLocations,
    pub state: BoardState,
}

impl Board {
    pub fn from_starting_position() -> Board {
        return Self::from_fen(STARTING_POSITION);
    }

    pub fn from_fen(fen: &str) -> Board {
        let parsed_fen = FenBoardState::from_fen(fen);
        return board_from_fen_state(parsed_fen);
    }

    pub fn to_fen(&self) -> String {
        return fen_state_from_board(self).to_fen();
    }

    pub fn get_piece_map(&self) -> &FnvHashMap<u8, Piece> {
        return self.piece_locations.get_piece_map();
    }

    pub fn get_state(&self) -> &BoardState {
        return &self.state;
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let king_square = self.find_king(self.state.to_move);
        let mut moves: Vec<Move> = Vec::new();
        let (checks, pins) = parse_checks_and_pins(self.get_checks_and_pins(&king_square, self.state.to_move));
        let pinned_squares: FnvHashSet<u8> = pins.iter().map(|p| p.pinned_square).collect();
        if checks.len() > 1 { return self.get_legal_king_moves() }
        if !checks.is_empty() { return self.get_legal_moves_from_check(checks.first().unwrap(), &pinned_squares) }
        for pin in pins {
            moves.append(&mut (self.get_legal_moves_for_pinned_piece(&pin)))
        }
        for square in self.get_pieces_to_move() {
            if pinned_squares.contains(square) { continue };
            if *square == self.find_king(self.state.to_move) {
                moves.append(&mut (self.get_legal_king_moves()))
            } else {
                moves.append(&mut (self.get_moves_for_piece(square)))
            }
        }
        moves.append(&mut (self.get_castle_moves(self.state.to_move)));
        return moves;
    }

    pub fn make_move(&mut self, new_move: &Move) -> ReversibleBoardChange {
        let result = ReversibleBoardChange {
            move_made: *new_move,
            prior_state: *self.get_state(),
        };
        self.state.increment_halfmove_clock();
        self.state.clear_en_passant_target();
        match self.piece_locations.apply_move(new_move) {
            Err(e) => panic!("{}", e.msg),
            Ok((piece, cap)) => {
                if piece.piece_type == PieceType::Pawn { self.state.reset_halfmove_clock() }
                match cap { Some(_) => self.state.reset_halfmove_clock(), None => () }
            }
        }
        self.update_castle_availability(new_move);
        
        match new_move {
            Move::TwoSquarePawnMove(m) => self.state.set_en_passant_target(m.en_passant_target),
            Move::Promotion(_) => self.state.reset_halfmove_clock(),
            _ => ()
        }
        if self.state.get_move_color() == Color::Black { self.state.increment_move_number() }
        self.state.change_move_color();

        return result;
    }

    pub fn unmake_move(&mut self, change: ReversibleBoardChange) {
        self.state = change.prior_state;
        match self.piece_locations.unapply_move(&change.move_made) {
            Err(e) => panic!("{}", e.msg),
            Ok(_) => ()
        }
    }

    pub fn get_piece_squares(&self) -> Vec<(u8, &Piece)> {
        return self.piece_locations.all_pieces(Color::White).iter().chain(self.piece_locations.all_pieces(Color::Black).iter()).fold(Vec::new(), |mut v, square| {
            v.push((*square, self.piece_locations.piece_at(square).unwrap()));
            v
        })
    }

    fn update_castle_availability(&mut self, new_move: &Move) {
        for m in new_move.get_piece_movements() {
            if m.start_square == BoardSquare::E1.value() || m.end_square == BoardSquare::E1.value() {
                self.state.disable_castle(Color::White, CastleType::Kingside);
                self.state.disable_castle(Color::White, CastleType::Queenside);
            }
            if m.start_square == BoardSquare::E8.value() || m.end_square == BoardSquare::E8.value() {
                self.state.disable_castle(Color::Black, CastleType::Kingside);
                self.state.disable_castle(Color::Black, CastleType::Queenside);
            }
            if m.start_square == BoardSquare::H1.value() || m.end_square == BoardSquare::H1.value() {
                self.state.disable_castle(Color::White, CastleType::Kingside);
            }
            if m.start_square == BoardSquare::A1.value() || m.end_square == BoardSquare::A1.value() {
                self.state.disable_castle(Color::White, CastleType::Queenside);
            }
            if m.start_square == BoardSquare::H8.value() || m.end_square == BoardSquare::H8.value() {
                self.state.disable_castle(Color::Black, CastleType::Kingside);
            }
            if m.start_square == BoardSquare::A8.value() || m.end_square == BoardSquare::A8.value() {
                self.state.disable_castle(Color::Black, CastleType::Queenside);
            }
        }
    }

    fn get_moves_for_piece(&self, square: &u8) -> Vec<Move> {
        let piece = self.piece_locations.piece_at(square).unwrap();
        let movement = get_moves(square, piece);
        return movement.movement_rays().iter().fold(Vec::new(), |mut moves, ray| {
            for move_square in ray {
                match self.piece_locations.piece_at(move_square) {
                    Some(p) if p.color == self.state.to_move => break,
                    Some(p) => {
                        if movement.can_capture(move_square) { 
                            if piece.piece_type == PieceType::Pawn && pawn_square_is_promotion(piece.color, *square) {
                                moves.append(&mut create_promotions(*square, *move_square, Some(*p)))
                            } else {
                                moves.push(Move::BasicMove(BasicMove { start: *square, end: *move_square, capture: Some(*p) }))
                            }
                        }
                        break;
                    },
                    None => {
                        if Some(*move_square) == self.state.en_passant_target && piece.piece_type == PieceType::Pawn && movement.can_capture(move_square) {
                            let capture_square = get_capture_square_for_en_passant(*square, *move_square);
                            if !self.en_passant_is_pin(self.state.to_move, *square, capture_square, self.state.get_en_passant_target().unwrap()) {
                                moves.push(Move::EnPassant(EnPassant {
                                    basic_move: BasicMove { start: *square, end: *move_square, capture: self.piece_locations.piece_at(&capture_square).map(|p| *p) },
                                    capture_square: capture_square
                                }));
                            }
                            break
                        } else if movement.can_move(move_square) {
                            if piece.piece_type == PieceType::Pawn && pawn_square_is_promotion(piece.color, *move_square) {
                                moves.append(&mut create_promotions(*square, *move_square, None))
                            } else if piece.piece_type == PieceType::Pawn && pawn_square_is_starting(piece.color, *square) && pawn_square_is_fourth_rank(piece.color, *move_square) {
                                moves.push(Move::TwoSquarePawnMove(TwoSquarePawnMove {
                                    basic_move: BasicMove { start: *square, end: *move_square, capture: None },
                                    en_passant_target: get_en_passant_target_for_two_square_first_move(piece.color, *move_square)
                                }));
                            } else {
                                moves.push(Move::BasicMove(BasicMove { start: *square, end: *move_square, capture: None }));
                            }
                        }
                    }
                }
            }
            moves
        })
    }

    fn en_passant_is_pin(&self, king_color: Color, start_square: u8, capture_square: u8, target_square: u8) -> bool {
        let king_square = self.find_king(king_color);
        let attacking_movement = get_moves(&king_square, &Piece { color: king_color.swap(), piece_type: PieceType::Queen });
        if !attacking_movement.attacked_squares().contains(&king_square) { return false };
        for ray in attacking_movement.movement_rays() {
            if !ray.contains(&king_square) { continue };
            for square in ray {
                if square == &target_square { return false };
                if square == &start_square || square == &capture_square { continue };
                match self.piece_locations.piece_at(square) {
                    Some(p) if p.color != king_color => {
                        match get_moves(square, p).attacked_squares().contains(&king_square) {
                            true => return true,
                            false => return false,
                        }
                    },
                    Some(_p) => return false,
                    None => ()
                }
            }
        }
        return false;
    }

    fn get_legal_moves_for_pinned_piece(&self, pin: &Pin) -> Vec<Move> {
        let pinned_piece = self.piece_locations.piece_at(&pin.pinned_square).unwrap();
        let movement = get_moves(&pin.pinned_square, pinned_piece);
        let mut moves: Vec<Move> = Vec::new();
        if movement.can_capture(&pin.pinning_square) {
            moves.push(Move::BasicMove(BasicMove { start: pin.pinned_square, end: pin.pinning_square, capture: self.piece_locations.piece_at(&pin.pinning_square).map(|p| *p) }))
        }
        return pin.pinning_path.iter().fold(moves, |mut acc, square| {
            if movement.can_move(square) {
                acc.push(Move::BasicMove(BasicMove { start: pin.pinned_square, end: *square, capture: None }))
            }
            acc
        });
    }

    fn get_legal_moves_from_check(&self, check: &Check, pinned_squares: &FnvHashSet<u8>) -> Vec<Move> {
        let mut moves = self.get_legal_king_moves();
        for square in self.get_pieces_to_move() {
            if pinned_squares.contains(square) { continue }
            let piece = self.piece_locations.piece_at(square).expect(&format!("Failed looking for a {} piece on {}", self.state.to_move.value(), BoardSquare::from_value(*square).get_notation_string()));
            if piece.piece_type == PieceType::King { continue }
            let movement = get_moves(square, piece);
            for block in &check.blocking_squares {
                if movement.can_move(block) {
                    moves.push(Move::BasicMove(BasicMove { start: *square, end: *block, capture: None }))
                }
            }
            if movement.can_capture(&check.checking_square) {
                moves.push(Move::BasicMove(BasicMove { start: *square, end: check.checking_square, capture: self.piece_locations.piece_at(&check.checking_square).map(|p| *p) }))
            }
        }
        return moves;
    }

    fn get_legal_king_moves(&self) -> Vec<Move> {
        let king_location = self.find_king(self.state.to_move);
        let king = self.piece_locations.piece_at(&king_location).unwrap();
        return get_moves(&king_location, king).attacked_squares().iter().fold(Vec::new(), |mut moves, square| {
            match self.piece_locations.piece_at(square) {
                Some(p) if p.color == self.state.to_move => (),
                Some(p) => {
                    if !self.is_check(square, self.state.to_move) {
                        moves.push(Move::BasicMove(BasicMove { start: king_location, end: *square, capture: Some(*p) }));
                    }
                },
                None => {
                    if !self.is_check(square, self.state.to_move) {
                        moves.push(Move::BasicMove(BasicMove { start: king_location, end: *square, capture: None }));
                    }
                }
            }
            moves
        })
    }

    fn get_castle_moves(&self, color: Color) -> Vec<Move> {
        [self.get_castle(color, CastleType::Kingside), self.get_castle(color, CastleType::Queenside)].into_iter().filter_map(|opt| {
            match opt { Some(m) => Some(m), None => None }
        }).collect()
    }

    fn get_castle(&self, color: Color, side: CastleType) -> Option<Move> {
        if !self.state.can_castle(color, side) { return None };
        let detail = get_castle_details(color, side);
        for square in &detail.transit_squares {
            match self.piece_locations.piece_at(square) { Some(_) => return None, None => () }
        }
        for square in &detail.king_transit_squares {
            if self.is_check(square, color) { return None }
        }
        if self.is_check(&detail.king_end, color) { return None };
        return Some(Move::Castle(Castle {
            side: side,
            king_start: detail.king_start,
            king_end: detail.king_end,
            rook_start: detail.rook_start,
            rook_end: detail.rook_end,
        }))
    }

    fn find_king(&self, color: Color) -> u8 {
        return match color {
            Color::White => self.piece_locations.find_king(Color::White),
            Color::Black => self.piece_locations.find_king(Color::Black),
        }
    }

    fn get_pieces_to_move(&self) -> &FnvIndexSet<u8> {
        return match self.state.get_move_color() {
            Color::White => &self.piece_locations.all_pieces(Color::White),
            Color::Black => &self.piece_locations.all_pieces(Color::Black),
        }
    }

    fn is_check(&self, king_square: &u8, king_color: Color) -> bool {
        for path in get_all_moves(&king_square).movement_rays() {
            match self.get_check_for_path(king_square, king_color, path) {
                Some(_) => return true,
                None => ()
            }
        }
        return false;
    }

    fn get_checks_and_pins(&self, king_square: &u8, king_color: Color) -> Vec<CheckOrPin> {
        let mut checks_and_pins: Vec<CheckOrPin> = Vec::new();
        for path in get_all_moves(&king_square).movement_rays() {
            match self.get_check_or_pin_for_path(king_square, king_color, path) {
                Some(x) => checks_and_pins.push(x),
                None => ()
            }
        }
        return checks_and_pins;
    }

    fn get_check_for_path(&self, king_square: &u8, king_color: Color, path: &FnvIndexSet<u8>) -> Option<Check> {
        let mut checking_path: FnvIndexSet<u8> = Default::default();
        for square in path {
            match self.piece_locations.piece_at(square) {
                Some(p) if p.color != king_color => {
                    if get_moves(&square, p).attacked_squares().contains(king_square) {
                        return Some(Check { checking_square: *square, blocking_squares: checking_path })
                    } else {
                        return None
                    }
                },
                Some(p) if p.color == king_color && p.piece_type == PieceType::King => { checking_path.insert(*square); },
                Some(_p) => return None,
                None => { checking_path.insert(*square); }
            }
        }
        return None
    }

    fn get_check_or_pin_for_path(&self, king_square: &u8, king_color: Color, path: &FnvIndexSet<u8>) -> Option<CheckOrPin> {
        let mut pinned_piece: Option<u8> = None;
        let mut current_path: FnvIndexSet<u8> = Default::default();
        for square in path {
            match self.piece_locations.piece_at(square) {
                Some(p) if p.color != king_color => {
                    if !get_moves(square, p).attacked_squares().contains(king_square) { return None };
                    match pinned_piece {
                        Some(s) => {
                            return Some(CheckOrPin::Pin(Pin{ pinned_square: s, pinning_square: *square, pinning_path: current_path }));
                        },
                        None => {
                            return Some(CheckOrPin::Check(Check { checking_square: *square, blocking_squares: current_path }));
                        }
                    }
                },
                Some(_p) => {
                    match pinned_piece {
                        Some(_) => return None,
                        None => pinned_piece = Some(*square)
                    }
                },
                None =>  { current_path.insert(*square); }
            }
        }
        return None;
    }
}