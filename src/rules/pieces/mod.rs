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