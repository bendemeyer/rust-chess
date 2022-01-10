use crate::rules::Color;

use crate::rules::board::squares::is_second_rank;

use super::{PieceType, Piece};


#[derive(Clone, Copy)]
pub enum SlideDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl SlideDirection {
    pub fn diagonals() -> [SlideDirection; 4] {
        [
            SlideDirection::NorthEast,
            SlideDirection::SouthEast,
            SlideDirection::SouthWest,
            SlideDirection::NorthWest,
        ]
    }

    pub fn orthagonals() -> [SlideDirection; 4] {
        [
            SlideDirection::North,
            SlideDirection::East,
            SlideDirection::South,
            SlideDirection::West,
        ]
    }

    pub fn all_directions() -> [SlideDirection; 8] {
        [
            SlideDirection::North,
            SlideDirection::NorthEast,
            SlideDirection::East,
            SlideDirection::SouthEast,
            SlideDirection::South,
            SlideDirection::SouthWest,
            SlideDirection::West,
            SlideDirection::NorthWest,
        ]
    }

    pub fn get_direction(&self) -> (i8, i8) {
        return match self {
            SlideDirection::North     => (0, 1),
            SlideDirection::NorthEast => (1, 1),
            SlideDirection::East      => (1, 0),
            SlideDirection::SouthEast => (1, -1),
            SlideDirection::South     => (0, -1),
            SlideDirection::SouthWest => (-1, -1),
            SlideDirection::West      => (-1, 0),
            SlideDirection::NorthWest => (-1, 1),
        }
    }

    pub fn get_hash_offset(&self) -> u16 {
        return match self {
            SlideDirection::North     => 64 * 0,
            SlideDirection::NorthEast => 64 * 1,
            SlideDirection::East      => 64 * 2,
            SlideDirection::SouthEast => 64 * 3,
            SlideDirection::South     => 64 * 4,
            SlideDirection::SouthWest => 64 * 5,
            SlideDirection::West      => 64 * 6,
            SlideDirection::NorthWest => 64 * 7,
        }
    }

    pub fn is_positive(&self) -> bool {
        return match self {
            SlideDirection::North     => true,
            SlideDirection::NorthEast => true,
            SlideDirection::East      => true,
            SlideDirection::SouthEast => false,
            SlideDirection::South     => false,
            SlideDirection::SouthWest => false,
            SlideDirection::West      => false,
            SlideDirection::NorthWest => true,
        }
    }
}


#[derive(Clone, Copy)]
pub enum PawnMovement {
    WhiteAdvance,
    WhiteAttack,
    BlackAdvance,
    BlackAttack,
}

impl PawnMovement {
    pub fn get_movements(&self) -> Vec<(i8, i8)> {
        return match self {
            PawnMovement::WhiteAdvance => Vec::from([(0, 1)]),
            PawnMovement::WhiteAttack  => Vec::from([(1, 1), (-1, 1)]),
            PawnMovement::BlackAdvance => Vec::from([(0, -1)]),
            PawnMovement::BlackAttack  => Vec::from([(1, -1), (-1, -1)]),
        }
    }

    pub fn get_hash_offset(&self) -> u8 {
        return match self {
            PawnMovement::WhiteAdvance => 64 * 0,
            PawnMovement::WhiteAttack  => 64 * 1,
            PawnMovement::BlackAdvance => 64 * 2,
            PawnMovement::BlackAttack  => 64 * 3,
        }
    }

    pub fn get_max_distance(&self, square: u8) -> u8 {
        return match self {
            PawnMovement::WhiteAdvance => if is_second_rank(square, Color::White) { 2u8 } else { 1u8 },
            PawnMovement::WhiteAttack  => 1u8,
            PawnMovement::BlackAdvance => if is_second_rank(square, Color::Black) { 2u8 } else { 1u8 },
            PawnMovement::BlackAttack  => 1u8,
        }
    }

    pub fn is_positive(&self) -> bool {
        return match self {
            PawnMovement::WhiteAdvance => true,
            PawnMovement::WhiteAttack  => true,
            PawnMovement::BlackAdvance => false,
            PawnMovement::BlackAttack  => false,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CastleType {
    Kingside,
    Queenside,
}

impl CastleType {
    fn get_notation(&self) -> String {
        return match self {
            &Self::Kingside => String::from("O-O"),
            &Self::Queenside => String::from("O-O-O"),
        }
    }

    pub fn value(&self) -> &str {
        return match self {
            &Self::Kingside => "kingside",
            &Self::Queenside => "queenside",
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PieceMovement {
    pub color: Color,
    pub piece_type: PieceType,
    pub start_square: u8,
    pub end_square: u8,
}

impl PieceMovement {
    pub fn get_piece(&self) -> Piece {
        return Piece { color: self.color, piece_type: self.piece_type }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Capture {
    pub color: Color,
    pub piece_type: PieceType,
    pub square: u8,
}

impl Capture {
    pub fn get_piece(&self) -> Piece {
        return Piece { color: self.color, piece_type: self.piece_type }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Move {
    NewGame(NewGame),
    BasicMove(BasicMove),
    Castle(Castle),
    Promotion(Promotion),
    TwoSquarePawnMove(TwoSquarePawnMove),
    EnPassant(EnPassant),
}

impl Move {
    pub fn get_piece_movements(&self) -> Vec<PieceMovement> {
        match self {
            Move::NewGame(_m) => Vec::new(),
            Move::BasicMove(m) => m.get_piece_movements(),
            Move::Castle(m) => m.get_piece_movements(),
            Move::Promotion(m) => m.basic_move.get_piece_movements(),
            Move::TwoSquarePawnMove(m) => m.basic_move.get_piece_movements(),
            Move::EnPassant(m) => m.basic_move.get_piece_movements(),
        }
    }

    pub fn get_capture(&self) -> Option<Capture> {
        match self {
            Move::NewGame(_m) => None,
            Move::BasicMove(m) => m.get_capture(),
            Move::Castle(m) => m.get_capture(),
            Move::Promotion(m) => m.basic_move.get_capture(),
            Move::TwoSquarePawnMove(m) => m.basic_move.get_capture(),
            Move::EnPassant(m) => m.get_capture(),
        }
    }

    pub fn relative_capture_value(&self) -> Option<i16> {
        self.get_capture().map(|cap| {
            self.get_piece_movements()[0].get_piece().relative_value(cap.get_piece())
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NewGame {}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BasicMove {
    pub piece: Piece,
    pub start: u8,
    pub end: u8,
    pub capture: Option<Piece>,
}

impl BasicMove {
    fn get_piece_movements(&self) -> Vec<PieceMovement> {
        return [ PieceMovement {
            color: self.piece.color,
            piece_type: self.piece.piece_type,
            start_square: self.start,
            end_square: self.end,
        } ].into_iter().collect();
    }

    fn get_capture(&self) -> Option<Capture> {
        return self.capture.map(|p| {
            Capture { color: p.color, piece_type: p.piece_type, square: self.end }
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Castle {
    pub color: Color,
    pub side: CastleType,
    pub king_start: u8,
    pub king_end: u8,
    pub rook_start: u8,
    pub rook_end: u8,
}

impl Castle {
    fn get_piece_movements(&self) -> Vec<PieceMovement> {
        return [
            PieceMovement { color: self.color, piece_type: PieceType::King, start_square: self.king_start, end_square: self.king_end },
            PieceMovement { color: self.color, piece_type: PieceType::Rook, start_square: self.rook_start, end_square: self.rook_end },
        ].into_iter().collect()
    }

    fn get_capture(&self) -> Option<Capture> {
        return None
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Promotion {
    pub basic_move: BasicMove,
    pub promote_to: PieceType,
}

impl Promotion {
    pub fn get_all_from_basic_move(base: &BasicMove) -> Vec<Promotion> {
        return [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight].into_iter().map(|ptype| {
            Promotion { basic_move: *base, promote_to: ptype }
        }).collect();
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TwoSquarePawnMove {
    pub basic_move: BasicMove,
    pub en_passant_target: u8,
}

impl TwoSquarePawnMove {
    pub fn from_basic_move(base: &BasicMove, en_passant_target: u8) -> TwoSquarePawnMove {
        TwoSquarePawnMove { basic_move: *base, en_passant_target: en_passant_target }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EnPassant {
    pub basic_move: BasicMove,
    pub capture_square: u8,
}

impl EnPassant {
    pub fn from_basic_move(base: &BasicMove, capture_square: u8) -> EnPassant {
        EnPassant { basic_move: *base, capture_square: capture_square }
    }

    fn get_capture(&self) -> Option<Capture> {
        return self.basic_move.capture.map(|p| {
            Capture { color: p.color, piece_type: p.piece_type, square: self.capture_square }
        })
    }
}