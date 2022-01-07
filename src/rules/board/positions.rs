use fxhash::FxHashMap;

use crate::rules::Color;
use crate::rules::pieces::PieceType;
use crate::rules::pieces::movement::{Move, SlideDirection, PawnMovement};
use crate::rules::pieces::{Piece, movement::CastleType};
use crate::util::errors::InputError;

use super::bitboards::{get_bit_for_square, set_bit_at_square, unset_bit_at_square, get_diagonal_bitboard, get_ray_bitboard, BitboardSquares, get_knight_bitboard, get_pawn_bitboard, get_orthagonal_bitboard};
use super::squares::BoardSquare;


#[derive(Default)]
pub struct AttacksAndPins {
    pub target: u8,
    pub attacks: Vec<Attack>,
    pub attackers: u64,
    pub pins: Vec<Pin>,
    pub pinners: u64,
    pub pinned: u64,
}

impl AttacksAndPins {
    pub fn ingest(&mut self, item: AttackOrPin) {
        match item {
            AttackOrPin::Attack(a) => {
                self.attackers |= get_bit_for_square(a.attacking_square);
                self.attacks.push(a);
            }
            AttackOrPin::Pin(p) => {
                self.pinners |= get_bit_for_square(p.pinning_square);
                self.pinned |= get_bit_for_square(p.pinned_square);
                self.pins.push(p);

            }
        }
    }
}


pub enum AttackOrPin {
    Attack(Attack),
    Pin(Pin),
}


pub struct Attack {
    pub attacking_square: u8,
    pub attack_path: u64,
}

pub struct Pin {
    pub pinning_square: u8,
    pub pinned_square: u8,
    pub pin_path: u64,
}


#[derive(Clone, Debug)]
pub struct CastlingSquares {
    pub king_start: u8,
    pub king_end: u8,
    pub rook_start: u8,
    pub rook_end: u8,
    pub transit_squares: u64,
    pub king_transit_squares: u64
}

impl CastlingSquares {
    pub fn from_color_and_type(color: Color, ctype: CastleType) -> CastlingSquares {
        return match (color, ctype) {
            (Color::White, CastleType::Kingside) => {
                 CastlingSquares {
                     king_start: BoardSquare::E1.value(), king_end: BoardSquare::G1.value(),
                     rook_start: BoardSquare::H1.value(), rook_end: BoardSquare::F1.value(),
                     transit_squares: get_bit_for_square(BoardSquare::F1.value()) | get_bit_for_square(BoardSquare::G1.value()),
                     king_transit_squares: get_bit_for_square(BoardSquare::F1.value()),
                 }
            },
            (Color::White, CastleType::Queenside) => {
                CastlingSquares {
                    king_start: BoardSquare::E1.value(), king_end: BoardSquare::C1.value(),
                    rook_start: BoardSquare::A1.value(), rook_end: BoardSquare::D1.value(),
                    transit_squares: get_bit_for_square(BoardSquare::D1.value()) | get_bit_for_square(BoardSquare::C1.value()) | get_bit_for_square(BoardSquare::B1.value()),
                    king_transit_squares: get_bit_for_square(BoardSquare::D1.value()),
                }
            },
            (Color::Black, CastleType::Kingside) => {
                CastlingSquares {
                    king_start: BoardSquare::E8.value(), king_end: BoardSquare::G8.value(),
                    rook_start: BoardSquare::H8.value(), rook_end: BoardSquare::F8.value(),
                    transit_squares: get_bit_for_square(BoardSquare::F8.value()) | get_bit_for_square(BoardSquare::G8.value()),
                    king_transit_squares: get_bit_for_square(BoardSquare::F8.value()),
                }
            },
            (Color::Black, CastleType::Queenside) => {
                CastlingSquares {
                    king_start: BoardSquare::E8.value(), king_end: BoardSquare::C8.value(),
                    rook_start: BoardSquare::A8.value(), rook_end: BoardSquare::D8.value(),
                    transit_squares: get_bit_for_square(BoardSquare::D8.value()) | get_bit_for_square(BoardSquare::C8.value()) | get_bit_for_square(BoardSquare::B8.value()),
                    king_transit_squares: get_bit_for_square(BoardSquare::D8.value()),
                }
            }
        }
    }
}


#[derive(Clone, Default)]
pub struct BoardPositions {
    pub piece_map: FxHashMap<u8, Piece>,
    white_pieces: u64,
    white_king: u64,
    black_pieces: u64,
    black_king: u64,
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
}

impl BoardPositions {
    pub fn from_piece_map(map: FxHashMap<u8, Piece>) -> Self {
        return map.into_iter().fold(Default::default(), |mut locs, (s, p)| {
            locs.insert_piece(s, p);
            locs
        });
    }

    pub fn get_piece_map(&self) -> &FxHashMap<u8, Piece> {
        return &self.piece_map;
    }

    pub fn find_king(&self, color: Color) -> u8 {
        return match color {
            Color::White => self.white_king.trailing_zeros() as u8,
            Color::Black => self.black_king.trailing_zeros() as u8
        }
    }

    pub fn piece_at(&self, square: &u8) -> Option<&Piece> {
        return self.piece_map.get(square);
    }

    pub fn get_piece_locations(&self, color: Color, piece_type: PieceType) -> u64 {
        let color_board = match color { Color::White => self.white_pieces, Color::Black => self.black_pieces };
        let piece_board = match piece_type {
            PieceType::Pawn => self.pawns,
            PieceType::Knight => self.knights,
            PieceType::Bishop => self.bishops,
            PieceType::Rook => self.rooks,
            PieceType::Queen => self.queens,
            PieceType::King => {
                match color {
                    Color::White => return self.white_king,
                    Color::Black => return self.black_king,
                }
            }
        };
        return color_board & piece_board;
    }

    pub fn get_diagonal_slider_locations(&self, color: Color) -> u64 {
        let color_board = match color { Color::White => self.white_pieces, Color::Black => self.black_pieces };
        return color_board & (self.bishops | self.queens);
    }

    pub fn get_orthagonal_slider_locations(&self, color: Color) -> u64 {
        let color_board = match color { Color::White => self.white_pieces, Color::Black => self.black_pieces };
        return color_board & (self.rooks | self.queens);
    }

    pub fn get_all_piece_locations(&self, color: Color) -> u64 {
        return match color { Color::White => self.white_pieces, Color::Black => self.black_pieces };
    }

    fn insert_piece(&mut self, square: u8, piece: Piece) {
        match piece.color {
            Color::White => self.white_pieces = set_bit_at_square(self.white_pieces, square),
            Color::Black => self.black_pieces = set_bit_at_square(self.black_pieces, square),
        }
        match piece.piece_type {
            PieceType::Pawn   => self.pawns   = set_bit_at_square(self.pawns, square),
            PieceType::Knight => self.knights = set_bit_at_square(self.knights, square),
            PieceType::Bishop => self.bishops = set_bit_at_square(self.bishops, square),
            PieceType::Rook   => self.rooks   = set_bit_at_square(self.rooks, square),
            PieceType::Queen  => self.queens  = set_bit_at_square(self.queens, square),
            PieceType::King   => {
                match piece.color {
                    Color::White => self.white_king = get_bit_for_square(square),
                    Color::Black => self.black_king = get_bit_for_square(square),
                }
            }
        }
        self.piece_map.insert(square, piece);
    }

    fn try_remove_piece(&mut self, square: u8) -> Option<Piece> {
        match self.piece_map.remove(&square) {
            Some(p) => {
                match p.color {
                    Color::White => self.white_pieces = unset_bit_at_square(self.white_pieces, square),
                    Color::Black => self.black_pieces = unset_bit_at_square(self.black_pieces, square),
                }
                match p.piece_type {
                    PieceType::Pawn   => self.pawns   = unset_bit_at_square(self.pawns, square),
                    PieceType::Knight => self.knights = unset_bit_at_square(self.knights, square),
                    PieceType::Bishop => self.bishops = unset_bit_at_square(self.bishops, square),
                    PieceType::Rook   => self.rooks   = unset_bit_at_square(self.rooks, square),
                    PieceType::Queen  => self.queens  = unset_bit_at_square(self.queens, square),
                    PieceType::King   => (),
                }
                Some(p)
            }
            None => None
        }
    }

    fn move_piece(&mut self, start: u8, end: u8) -> Result<&Piece, InputError> {
        match self.try_remove_piece(start) {
            None => return Err(InputError::new(&format!("No piece to move at square {}", start))),
            Some(p) => self.insert_piece(end, p)
        }
        return Ok(self.piece_at(&end).unwrap())
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
                let captured_piece = self.try_remove_piece(b.end);
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
                let captured_piece = self.try_remove_piece(e.capture_square);
                match self.move_piece(e.basic_move.start, e.basic_move.end) {
                    Err(e) => Err(e),
                    Ok(p) => Ok((p, captured_piece))
                }
            },
            Move::Promotion(p) => {
                let captured_piece = self.try_remove_piece(p.basic_move.end);
                match self.try_remove_piece(p.basic_move.start) {
                    None => return Err(InputError::new(&format!("No piece to move at square {}", p.basic_move.start))),
                    Some(piece) => {
                        let promoted_piece = Piece { color: piece.color, piece_type: p.promote_to };
                        self.insert_piece(p.basic_move.end, promoted_piece);
                        Ok((self.piece_at(&p.basic_move.end).unwrap(), captured_piece))
                    }
                }
            },
            Move::NewGame(_) => Err(InputError::new("Cannot apply move 'NewGame'"))
        }
    }

    pub fn unapply_move(&mut self, old_move: &Move) -> Result<(), InputError> {
        match old_move {
            Move::EnPassant(e) => {
                self.move_piece(e.basic_move.end, e.basic_move.start)?;
                match old_move.get_capture() {
                    Some(capture) => self.insert_piece(e.capture_square, capture.get_piece()),
                    None => return Err(InputError::new("En Passant missing captured piece"))
                };
            },
            Move::Promotion(p) => {
                match self.try_remove_piece(p.basic_move.end) {
                    None => return Err(InputError::new(&format!("No piece to unmove at square {}", p.basic_move.end))),
                    Some(piece) => {
                        let unpromoted_piece = Piece { color: piece.color, piece_type: PieceType::Pawn };
                        self.insert_piece(p.basic_move.start, unpromoted_piece);
                    }
                };
                match old_move.get_capture() {
                    Some(capture) => self.insert_piece(p.basic_move.end, capture.get_piece()),
                    None => ()
                }
            },
            _ => {
                for movement in old_move.get_piece_movements() {
                    self.move_piece(movement.end_square, movement.start_square)?;
                    match old_move.get_capture() {
                        Some(capture) => self.insert_piece(movement.end_square, capture.get_piece()),
                        None => ()
                    }
                }
            }
        }
        return Ok(());
    }

    pub fn get_attacks_and_pins(&self, target: u8, color: Color) -> AttacksAndPins {
        let attacking_color = color.swap();
        let mut result: AttacksAndPins = Default::default();
        result.target = target;
        let diagonal_attackers = self.get_diagonal_slider_locations(attacking_color);
        match get_diagonal_bitboard(target) & diagonal_attackers {
            0 => (),
            _ => {
                SlideDirection::diagonals().into_iter().for_each(|dir| {
                    match self.get_sliding_attack_or_pin(target, dir, color, diagonal_attackers) {
                        None => (),
                        Some(ap) => result.ingest(ap),
                    }
                })
            }
        }
        let orthagonal_attackers = self.get_orthagonal_slider_locations(attacking_color);
        match get_orthagonal_bitboard(target) & orthagonal_attackers {
            0 => (),
            _ => {
                SlideDirection::orthagonals().into_iter().for_each(|dir| {
                    match self.get_sliding_attack_or_pin(target, dir, color, orthagonal_attackers) {
                        None => (),
                        Some(ap) => result.ingest(ap),
                    }
                })
            }
        }
        BitboardSquares::from_board(get_knight_bitboard(target) & self.get_piece_locations(attacking_color, PieceType::Knight)).for_each(|s| {
            result.attacks.push(Attack { attacking_square: s, attack_path: 0u64 })
        });
        let pawn_attacks = match color { Color::White => PawnMovement::WhiteAttack, Color::Black => PawnMovement::BlackAttack };
        BitboardSquares::from_board(get_pawn_bitboard(target, pawn_attacks) & self.get_piece_locations(attacking_color, PieceType::Pawn)).for_each(|s| {
            result.attacks.push(Attack { attacking_square: s, attack_path: 0u64 })
        });
        return result;
    }

    fn get_sliding_attack_or_pin(&self, target: u8, dir: SlideDirection, color: Color, attackers: u64) -> Option<AttackOrPin> {
        let ray = get_ray_bitboard(target, dir);
        let friendlies = self.get_all_piece_locations(color);
        let enemies = self.get_all_piece_locations(color.swap());
        let all_pieces = friendlies | enemies;
        let blocks = ray & all_pieces;
        if blocks == 0 { return None };
        let first_block = match dir.is_positive() {
            true => blocks.trailing_zeros() as u8,
            false => 63 - blocks.leading_zeros() as u8,
        };
        let blocker_bit = get_bit_for_square(first_block);
        if blocker_bit & enemies != 0 && blocker_bit & attackers == 0 {
            return None;
        }
        let blocked_squares = get_ray_bitboard(first_block, dir);
        if blocker_bit & attackers != 0 {
            return Some(AttackOrPin::Attack(Attack {
                attacking_square: first_block,
                attack_path: ray ^ (blocker_bit | blocked_squares)
            }))
        }
        let next_blocks = blocked_squares & all_pieces;
        if next_blocks == 0 { return None };
        let second_block = match dir.is_positive() {
            true => next_blocks.trailing_zeros() as u8,
            false => 63 - next_blocks.leading_zeros() as u8,
        };
        let second_block_bit = get_bit_for_square(second_block);
        if second_block_bit & attackers == 0 { return None };
        let path_mask = blocker_bit | second_block_bit | get_ray_bitboard(second_block, dir);
        return Some(AttackOrPin::Pin(Pin {
            pinning_square: second_block,
            pinned_square: first_block,
            pin_path: ray ^ path_mask,
        }))
    }

    pub fn en_passant_is_illegal(&self, color: Color, start: u8, end: u8, capture: u8) -> bool {
        let king_square = self.find_king(color);
        let attacking_color = color.swap();
        let friendlies = self.get_all_piece_locations(color);
        let enemies = self.get_all_piece_locations(color.swap());
        let all_pieces = friendlies | enemies;

        let start_bit = get_bit_for_square(start);
        let end_bit = get_bit_for_square(end);
        let capture_bit = get_bit_for_square(capture);

        let diagonal_attackers = self.get_diagonal_slider_locations(attacking_color);
        let diagonal_bitboard = get_diagonal_bitboard(king_square);
        if diagonal_bitboard & capture_bit != 0 && diagonal_bitboard & diagonal_attackers != 0 {
            for dir in SlideDirection::diagonals() {
                let ray = get_ray_bitboard(king_square, dir);
                if ray & capture_bit == 0 { continue };
                if ray & diagonal_attackers == 0 { break };
                let mut blocks = ray & all_pieces;
                if blocks == 0 { continue };
                loop {
                    let next_block = get_bit_for_square(if dir.is_positive() { blocks.trailing_zeros() as u8 } else { 63 - blocks.leading_zeros() as u8 });
                    if next_block == capture_bit {
                        blocks &= !next_block;
                        continue;
                    }
                    if next_block & diagonal_attackers == 0 { return false };
                    return true;
                }
            }
        }
        let orthagonal_attackers = self.get_orthagonal_slider_locations(attacking_color);
        let orthagonal_bitboard = get_orthagonal_bitboard(king_square);
        if orthagonal_bitboard & capture_bit != 0 && orthagonal_bitboard & orthagonal_attackers != 0 {
            for dir in SlideDirection::orthagonals() {
                let ray = get_ray_bitboard(king_square, dir);
                if ray & capture_bit == 0 { continue };
                if ray & orthagonal_attackers == 0 { break };
                let mut blocks = ray & all_pieces;
                if blocks == 0 { continue }
                loop {
                    let next_block = get_bit_for_square(if dir.is_positive() { blocks.trailing_zeros() as u8 } else { 63 - blocks.leading_zeros() as u8 });
                    if next_block == capture_bit || next_block == start_bit {
                        blocks &= !next_block;
                        continue;
                    };
                    if next_block & orthagonal_attackers == 0 { return false };
                    if ray & end_bit != 0 { return false };
                    return true;
                }
            }
        }
        return false;
    }

    pub fn is_check(&self, king_square: u8, king_color: Color) -> bool {
        let attacking_color = king_color.swap();
        let current_king_location = get_bit_for_square(self.find_king(king_color));
        let friendlies = self.get_all_piece_locations(king_color);
        let enemies = self.get_all_piece_locations(attacking_color);
        let all_pieces = (friendlies | enemies) & !current_king_location;

        if self.get_piece_locations(attacking_color, PieceType::Knight) & get_knight_bitboard(king_square) != 0 {
            return true;
        }

        let pawn_attacks = match king_color { Color::White => PawnMovement::WhiteAttack, Color::Black => PawnMovement::BlackAttack };
        if  self.get_piece_locations(attacking_color, PieceType::Pawn) & get_pawn_bitboard(king_square, pawn_attacks) != 0 {
            return true;
        }

        let diagonal_attackers = self.get_diagonal_slider_locations(attacking_color);
        let diagonal_bitboard = get_diagonal_bitboard(king_square);
        if diagonal_attackers & diagonal_bitboard != 0 {
            for dir in SlideDirection::diagonals() {
                let ray = get_ray_bitboard(king_square, dir);
                let blocks = ray & all_pieces;
                if blocks == 0 { continue };
                let potential_attacker = get_bit_for_square(if dir.is_positive() { blocks.trailing_zeros() as u8 } else { 63 - blocks.leading_zeros() as u8 });
                if potential_attacker & diagonal_attackers != 0 { return true };
            }
        }

        let orthagonal_attackers = self.get_orthagonal_slider_locations(attacking_color);
        let orthagonal_bitboard = get_orthagonal_bitboard(king_square);
        if orthagonal_attackers & orthagonal_bitboard != 0 {
            for dir in SlideDirection::orthagonals() {
                let ray = get_ray_bitboard(king_square, dir);
                let blocks = ray & all_pieces;
                if blocks == 0 { continue };
                let potential_attacker = get_bit_for_square(if dir.is_positive() { blocks.trailing_zeros() as u8 } else { 63 - blocks.leading_zeros() as u8 });
                if potential_attacker & orthagonal_attackers != 0 { return true };
            }
        }

        return false;
    }
}