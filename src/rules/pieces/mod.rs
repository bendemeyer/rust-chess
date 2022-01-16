pub mod movement;

use super::{Color, ColorIterator};


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
    pub fn iter() -> PieceTypeIterator {
        return PieceTypeIterator::new();
    }

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

pub struct PieceTypeIterator {
    state: Option<PieceType>,
}

impl PieceTypeIterator {
    pub fn new() -> Self {
        return Self { state: None }
    }
}

impl Iterator for PieceTypeIterator {
    type Item = PieceType;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.state {
            None => Some(PieceType::Pawn),
            Some(PieceType::Pawn)   => Some(PieceType::Knight),
            Some(PieceType::Knight) => Some(PieceType::Bishop),
            Some(PieceType::Bishop) => Some(PieceType::Rook),
            Some(PieceType::Rook)   => Some(PieceType::Queen),
            Some(PieceType::Queen)  => Some(PieceType::King),
            Some(PieceType::King)   => None,
        };
        self.state = next;
        return next;
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn iter() -> PieceIterator {
        return PieceIterator::new();
    }

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


pub struct PieceIterator {
    color_state: Option<Color>,
    color_iter: ColorIterator,
    type_state: Option<PieceType>,
    type_iter: PieceTypeIterator,
}

impl PieceIterator {
    pub fn new() -> Self {
        let mut citer = ColorIterator::new();
        let mut titer = PieceTypeIterator::new();
        return Self {
            color_state: citer.next(),
            color_iter: citer,
            type_state: titer.next(),
            type_iter: titer,
        }
    }
}

impl Iterator for PieceIterator {
    type Item = Piece;

    fn next(&mut self) -> Option<Self::Item> {
        match self.color_state {
            Some(color) => match self.type_state {
                Some(piece_type) => {
                    self.type_state = self.type_iter.next();
                    Some(Piece { color: color, piece_type: piece_type })
                },
                None => {
                    self.color_state = self.color_iter.next();
                    self.type_iter = PieceTypeIterator::new();
                    self.type_state = self.type_iter.next();
                    self.next()
                }
            }
            None => None
        }
    }
}