pub mod movement;

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
            Self::Pawn   => ' ',
            Self::Knight => 'N',
            Self::Bishop => 'B',
            Self::Rook   => 'R',
            Self::Queen  => 'Q',
            Self::King   => 'K',
        }
    }

    pub fn name(&self) -> &str {
        return match self {
            Self::Pawn   => "pawn",
            Self::Knight => "knight",
            Self::Bishop => "bishop",
            Self::Rook   => "rook",
            Self::Queen  => "queen",
            Self::King   => "king",
        }
    }

    pub fn value(&self) -> u8 {
        return match self {
            Self::Pawn   => 1,
            Self::Knight => 3,
            Self::Bishop => 3,
            Self::Rook   => 5,
            Self::Queen  => 9,
            Self::King   => 100,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn material_score(&self) -> i16 {
        self.piece_type.value() as i16 * 100i16 * (match self.color {
            Color::White => 1,
            Color::Black => -1,
        })
    }

    pub fn relative_value(&self, other: Piece) -> i16 {
        return self.piece_type.value() as i16 - other.piece_type.value() as i16;
    }
}