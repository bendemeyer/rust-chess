use fxhash::FxHashMap;

use crate::rules::Color;
use crate::rules::pieces::PieceType;
use crate::rules::pieces::movement::{Move, SlideDirection, PawnMovement};
use crate::rules::pieces::{Piece, movement::CastleType};

use super::bitboards::{get_bit_for_square, set_bit_at_square, unset_bit_at_square, get_diagonal_bitboard, get_ray_bitboard, BitboardSquares, get_knight_bitboard, get_pawn_bitboard, get_orthagonal_bitboard, ColorBoard, PieceTypeBoard, PieceBoard, BitboardPieceLocations};
use super::squares::BoardSquare;


#[derive(Clone, Default)]
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


#[derive(Copy, Clone)]
pub enum AttackOrPin {
    Attack(Attack),
    Pin(Pin),
}

#[derive(Copy, Clone)]
pub struct Attack {
    pub attacking_square: u8,
    pub attack_path: u64,
}

#[derive(Copy, Clone)]
pub struct Pin {
    pub pinning_square: u8,
    pub pinned_square: u8,
    pub pin_path: u64,
}


#[derive(Copy, Clone, Debug)]
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


pub struct PieceBoardGenerator {
    position: BoardPosition,
    pieces: Vec<Piece>,
    mask: u64,
}

impl Iterator for PieceBoardGenerator {
    type Item = PieceBoard;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pieces.pop() {
            None => None,
            Some(p) => Some(self.position.get_masked_piece_board(p.color, p.piece_type, self.mask))
        }
    }
}


#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct BoardPosition {
    white_pieces: u64,
    black_pieces: u64,
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
    kings: u64,
}

impl BoardPosition {
    pub fn from_piece_map(map: FxHashMap<u8, Piece>) -> Self {
        return map.into_iter().fold(Default::default(), |mut locs, (s, p)| {
            locs.insert_piece(s, p);
            locs
        });
    }

    pub fn find_king(&self, color: Color) -> u8 {
        return self.get_piece_locations(color, PieceType::King).trailing_zeros() as u8;
    }

    fn get_color_boards(&self) -> [ColorBoard; 2] {
        return [
            ColorBoard::from_board(self.white_pieces, Color::White),
            ColorBoard::from_board(self.black_pieces, Color::Black),
        ]
    }

    fn get_piece_type_boards(&self) -> [PieceTypeBoard; 6] {
        return [
            PieceTypeBoard::from_board(self.pawns,   PieceType::Pawn   ),
            PieceTypeBoard::from_board(self.knights, PieceType::Knight ),
            PieceTypeBoard::from_board(self.bishops, PieceType::Bishop ),
            PieceTypeBoard::from_board(self.rooks,   PieceType::Rook   ),
            PieceTypeBoard::from_board(self.queens,  PieceType::Queen  ),
            PieceTypeBoard::from_board(self.kings,   PieceType::King   ),
        ]
    }

    pub fn piece_at(&self, square: &u8) -> Option<Piece> {
        let square_bit = get_bit_for_square(*square);
        let color = match self.get_color_boards().into_iter().find(|cb| {
            return square_bit & cb.get_board() > 0;
        }) { Some(cb) => *cb.get_color(), None => return None };
        let piece_type = match self.get_piece_type_boards().into_iter().find(|pb| {
            return square_bit & pb.get_board() > 0;
        }) { Some(pb) => *pb.get_piece_type(), None => return None };
        return Some(Piece { color: color, piece_type: piece_type });
    }

    pub fn get_masked_piece_squares(&self, pieces: Vec<Piece>, mask: u64) -> BitboardPieceLocations<PieceBoardGenerator> {
        return BitboardPieceLocations::from_iter(PieceBoardGenerator {
            position: *self,
            mask: mask,
            pieces: pieces,
        });
    }

    pub fn get_all_masked_piece_squares_for_color(&self, color: Color, mask: u64) -> BitboardPieceLocations<PieceBoardGenerator> {
        return self.get_masked_piece_squares(Vec::from([
            Piece { color: color, piece_type: PieceType::Pawn   },
            Piece { color: color, piece_type: PieceType::Knight },
            Piece { color: color, piece_type: PieceType::Bishop },
            Piece { color: color, piece_type: PieceType::Rook   },
            Piece { color: color, piece_type: PieceType::Queen  },
            Piece { color: color, piece_type: PieceType::King   },
        ]), mask);
    }

    pub fn get_masked_piece_board(&self, color: Color, piece_type: PieceType, mask: u64) -> PieceBoard {
        return PieceBoard::from_board(
            self.get_piece_locations(color, piece_type) & mask,
            Piece { color: color, piece_type: piece_type });
    }

    pub fn get_piece_locations(&self, color: Color, piece_type: PieceType) -> u64 {
        let color_board = match color { Color::White => self.white_pieces, Color::Black => self.black_pieces };
        let piece_board = match piece_type {
            PieceType::Pawn => self.pawns,
            PieceType::Knight => self.knights,
            PieceType::Bishop => self.bishops,
            PieceType::Rook => self.rooks,
            PieceType::Queen => self.queens,
            PieceType::King => self.kings,
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
        self.insert_piece_into_boards(square, piece);
    }

    fn insert_piece_into_boards(&mut self, square:u8, piece: Piece) {
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
            PieceType::King   => self.kings   = set_bit_at_square(self.kings, square),
        }
    }

    fn remove_piece(&mut self, square: u8, piece: Piece) {
        self.remove_piece_from_boards(square, piece);
    }

    fn remove_piece_from_boards(&mut self, square: u8, piece: Piece) {
        match piece.color {
            Color::White => self.white_pieces = unset_bit_at_square(self.white_pieces, square),
            Color::Black => self.black_pieces = unset_bit_at_square(self.black_pieces, square),
        }
        match piece.piece_type {
            PieceType::Pawn   => self.pawns   = unset_bit_at_square(self.pawns, square),
            PieceType::Knight => self.knights = unset_bit_at_square(self.knights, square),
            PieceType::Bishop => self.bishops = unset_bit_at_square(self.bishops, square),
            PieceType::Rook   => self.rooks   = unset_bit_at_square(self.rooks, square),
            PieceType::Queen  => self.queens  = unset_bit_at_square(self.queens, square),
            PieceType::King   => self.kings   = unset_bit_at_square(self.kings, square),
        }
    }

    fn move_piece(&mut self, start: u8, end: u8, piece: Piece) {
        self.remove_piece(start, piece);
        self.insert_piece(end, piece);
    }

    pub fn apply_move(&mut self, new_move: &Move) {
        if let Some(capture) = new_move.get_capture() {
            self.remove_piece(capture.square, capture.get_piece());
        }
        if let Move::Promotion(p) = new_move {
            self.remove_piece(p.basic_move.start, p.basic_move.piece);
            self.insert_piece(p.basic_move.end, Piece { color: p.basic_move.piece.color, piece_type: p.promote_to });
        } else {
            for movement in new_move.get_piece_movements() {
                self.move_piece(movement.start_square, movement.end_square, movement.get_piece());
            }
        }
    }

    pub fn unapply_move(&mut self, old_move: &Move) {
        if let Some(capture) = old_move.get_capture() {
            self.insert_piece(capture.square, capture.get_piece());
        }
        if let Move::Promotion(p) = old_move {
            self.remove_piece(p.basic_move.end, Piece { color: p.basic_move.piece.color, piece_type: p.promote_to });
            self.insert_piece(p.basic_move.start, p.basic_move.piece);
        } else {
            for movement in old_move.get_piece_movements() {
                self.move_piece(movement.end_square, movement.start_square, movement.get_piece());
            }
        }
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