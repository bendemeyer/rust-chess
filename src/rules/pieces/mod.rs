pub mod movement;

use fxhash::FxHashMap;
use movement::MovementVector;
use super::Color;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn get_notation(&self) -> char {
        return match self {
            &Self::Pawn   => ' ',
            &Self::Knight => 'N',
            &Self::Bishop => 'B',
            &Self::Rook   => 'R',
            &Self::King   => 'K',
            &Self::Queen  => 'Q',
        }
    }

    pub fn value(&self) -> &str {
        return match self {
            &Self::Pawn   => "pawn",
            &Self::Knight => "knight",
            &Self::Bishop => "bishop",
            &Self::Rook   => "rook",
            &Self::King   => "king",
            &Self::Queen  => "queen",
        }
    }
}


pub static UNMOVED_WHITE_PAWN_ADVANCING_VECTORS: [MovementVector; 1] = [
    MovementVector { col_shift: 0, row_shift: 1, max_dist: 2 },
];

pub static MOVED_WHITE_PAWN_ADVANCING_VECTORS: [MovementVector; 1] = [
    MovementVector { col_shift: 0, row_shift: 1, max_dist: 1 },
];

pub static WHITE_PAWN_ATTACKING_VECTORS: [MovementVector; 2] = [
    MovementVector { col_shift: 1, row_shift: 1, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: 1, max_dist: 1 },
];

pub static UNMOVED_BLACK_PAWN_ADVANCING_VECTORS: [MovementVector; 1] = [
    MovementVector { col_shift: 0, row_shift: -1, max_dist: 2 },
];

pub static MOVED_BLACK_PAWN_ADVANCING_VECTORS: [MovementVector; 1] = [
    MovementVector { col_shift: 0, row_shift: -1, max_dist: 1 },
];

pub static BLACK_PAWN_ATTACKING_VECTORS: [MovementVector; 2] = [
    MovementVector { col_shift: 1, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: -1, max_dist: 1 },
];

pub static KNIGHT_MOVE_VECTORS: [MovementVector; 8] = [
    MovementVector { col_shift: 1, row_shift: 2, max_dist: 1 },
    MovementVector { col_shift: 2, row_shift: 1, max_dist: 1 },
    MovementVector { col_shift: 2, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: 1, row_shift: -2, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: -2, max_dist: 1 },
    MovementVector { col_shift: -2, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: -2, row_shift: 1, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: 2, max_dist: 1 },
];

pub static BISHOP_MOVE_VECTORS: [MovementVector; 4] = [
    MovementVector { col_shift: 1, row_shift: 1, max_dist: 7 },
    MovementVector { col_shift: 1, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: 1, max_dist: 7 },
];

pub static ROOK_MOVE_VECTORS: [MovementVector; 4] = [
    MovementVector { col_shift: 0, row_shift: 1, max_dist: 7 },
    MovementVector { col_shift: 1, row_shift: 0, max_dist: 7 },
    MovementVector { col_shift: 0, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: 0, max_dist: 7 },
];

pub static QUEEN_MOVE_VECTORS: [MovementVector; 8] = [
    MovementVector { col_shift: 0, row_shift: 1, max_dist: 7 },
    MovementVector { col_shift: 1, row_shift: 1, max_dist: 7 },
    MovementVector { col_shift: 1, row_shift: 0, max_dist: 7 },
    MovementVector { col_shift: 1, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: 0, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: -1, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: 0, max_dist: 7 },
    MovementVector { col_shift: -1, row_shift: 1, max_dist: 7 },
];

pub static KING_MOVE_VECTORS: [MovementVector; 8] = [
    MovementVector { col_shift: 0, row_shift: 1, max_dist: 1 },
    MovementVector { col_shift: 1, row_shift: 1, max_dist: 1 },
    MovementVector { col_shift: 1, row_shift: 0, max_dist: 1 },
    MovementVector { col_shift: 1, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: 0, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: -1, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: 0, max_dist: 1 },
    MovementVector { col_shift: -1, row_shift: 1, max_dist: 1 },
];


lazy_static! {
    pub static ref PIECE_MOVE_VECTOR_MAP: FxHashMap<PieceType, Vec<&'static MovementVector>> = FxHashMap::from_iter([
        (PieceType::Knight, Vec::from_iter(KNIGHT_MOVE_VECTORS.iter())),
        (PieceType::Bishop, Vec::from_iter(BISHOP_MOVE_VECTORS.iter())),
        (PieceType::Rook, Vec::from_iter(ROOK_MOVE_VECTORS.iter())),
        (PieceType::Queen, Vec::from_iter(QUEEN_MOVE_VECTORS.iter())),
        (PieceType::King, Vec::from_iter(KING_MOVE_VECTORS.iter())),
    ].into_iter());
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn material_score(&self) -> i16 {
        return (match self.piece_type {
            PieceType::Pawn   => 100,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook   => 500,
            PieceType::Queen  => 900,
            PieceType::King   => 0,
        }) * (match self.color {
            Color::White => 1,
            Color::Black => -1,
        })
    }
}