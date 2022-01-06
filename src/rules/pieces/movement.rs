use fxhash::FxHashSet;

use crate::rules::Color;
use crate::util::{ControlFlow, FxIndexSet, FoldHelper, UnwrapsAll};

use crate::rules::board::squares::{BoardSquare, ROW_2, ROW_7};

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
            SlideDirection::North     => 0,
            SlideDirection::NorthEast => 64,
            SlideDirection::East      => 128,
            SlideDirection::SouthEast => 192,
            SlideDirection::South     => 256,
            SlideDirection::SouthWest => 320,
            SlideDirection::West      => 384,
            SlideDirection::NorthWest => 448,
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
            PawnMovement::WhiteAdvance => 0,
            PawnMovement::WhiteAttack  => 64,
            PawnMovement::BlackAdvance => 128,
            PawnMovement::BlackAttack  => 192,
        }
    }

    pub fn get_max_distance(&self, square: u8) -> u8 {
        return match self {
            PawnMovement::WhiteAdvance => if ROW_2.contains(&square) { 2u8 } else { 1u8 },
            PawnMovement::WhiteAttack  => 1u8,
            PawnMovement::BlackAdvance => if ROW_7.contains(&square) { 2u8 } else { 1u8 },
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


pub fn get_sliding_bitboard(square: u8, direction: SlideDirection) -> u64 {
    let (col_shift, row_shift) = direction.get_direction();
    let mut active_bits: FxHashSet<u8> = Default::default();
    let mut current_square = square;
    loop {
        match BoardSquare::from_value(current_square).apply_movement(col_shift, row_shift) {
            Err(_) => break,
            Ok(new_square) => {
                active_bits.insert(new_square.value());
                current_square = new_square.value();
            },
        }
    }
    let bit_string = (0u8..=63u8).fold(String::new(), |mut bits, bit| {
        if active_bits.contains(&bit) {
            bits.push('1')
        } else {
            bits.push('0')
        }
        bits
    });
    return u64::from_str_radix(&bit_string, 2).unwrap();
}


fn get_squares_for_vector(square: u8, vector: &MovementVector) -> Vec<u8> {
    return (0u8..vector.max_dist).try_fold(FoldHelper::init(Vec::new(), square), |fh, _| {
        match BoardSquare::from_value(fh.data).apply_movement(vector.col_shift, vector.row_shift) {
            Err(_) => ControlFlow::Break(fh),
            Ok(new_square) => {
                ControlFlow::Continue(FoldHelper::init([fh.accumulator, Vec::from([new_square.value()])].concat(), new_square.value()))
            }
        }
    }).unwrap_all().get_result();
}

fn build_static_vector(square: u8, vector: &MovementVector) -> FxIndexSet<u8> {
    return FxIndexSet::from_iter(get_squares_for_vector(square, vector).into_iter());
}

fn collect_static_vectors(square: u8, vectors: &Vec<&MovementVector>) -> Vec<FxIndexSet<u8>> {
    return vectors.iter().map(|vector| build_static_vector(square, vector)).collect();
}

pub fn build_movement_detail(square: u8, vectors: &Vec<&MovementVector>) -> PieceMovementDetail {
    return PieceMovementDetail::from_static_vectors(collect_static_vectors(square, vectors));
}

pub fn build_pawn_movement_detail(square: u8, adv_vecs: &Vec<&MovementVector>, att_vecs: &Vec<&MovementVector>) -> PawnMovementDetail {
    return PawnMovementDetail::from_rays(
        collect_static_vectors(square, adv_vecs),
        collect_static_vectors(square, att_vecs),
    );
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


pub struct MovementVector {
    pub col_shift: i8,
    pub row_shift: i8,
    pub max_dist: u8,
}


pub trait Movement {
    fn movement_rays(&self) -> &Vec<FxIndexSet<u8>>;
    fn attacked_squares(&self) -> &FxHashSet<u8>;
    fn can_move(&self, square: &u8) -> bool;
    fn can_capture(&self, square: &u8) -> bool;
}

pub enum MovementDetail {
    Piece(&'static PieceMovementDetail),
    Pawn(&'static PawnMovementDetail),
}

impl Movement for MovementDetail {
    fn movement_rays(&self) -> &Vec<FxIndexSet<u8>> {
        match self {
            &MovementDetail::Piece(m) => &m.rays,
            &MovementDetail::Pawn(m) => &m.all_rays,
        }
    }
    fn attacked_squares(&self) -> &FxHashSet<u8> {
        match self {
            &MovementDetail::Piece(m) => &m.all_squares,
            &MovementDetail::Pawn(m) => &m.attacking_squares,
        }
    }
    fn can_move(&self, square: &u8) -> bool {
        return match self {
            &MovementDetail::Piece(m) => m.all_squares.contains(square),
            &MovementDetail::Pawn(m) => m.advancing_squares.contains(square)
        }
    }
    fn can_capture(&self, square: &u8) -> bool {
        return match self {
            &MovementDetail::Piece(m) => m.all_squares.contains(square),
            &MovementDetail::Pawn(m) => m.attacking_squares.contains(square)
        }
    }
    
}

#[derive(Clone)]
pub struct PieceMovementDetail {
    pub rays: Vec<FxIndexSet<u8>>,
    pub all_squares: FxHashSet<u8>,
}

impl PieceMovementDetail {
    pub fn from_static_vectors(vectors: Vec<FxIndexSet<u8>>) -> PieceMovementDetail {
        PieceMovementDetail {
            rays: vectors.clone(),
            all_squares: vectors.clone().into_iter().flatten().collect(),
        }
    }
}

#[derive(Clone, Default)]
pub struct PawnMovementDetail {
    pub advancing_rays: Vec<FxIndexSet<u8>>,
    pub attacking_rays: Vec<FxIndexSet<u8>>,
    all_rays: Vec<FxIndexSet<u8>>,
    pub advancing_squares: FxHashSet<u8>,
    pub attacking_squares: FxHashSet<u8>,
}

impl PawnMovementDetail {
    pub fn from_rays(adv: Vec<FxIndexSet<u8>>, att: Vec<FxIndexSet<u8>>) -> PawnMovementDetail {
        return PawnMovementDetail {
            advancing_rays: adv.clone(),
            attacking_rays: att.clone(),
            all_rays: [adv.clone(), att.clone()].concat(),
            advancing_squares: adv.clone().into_iter().flatten().collect(),
            attacking_squares: att.clone().into_iter().flatten().collect(),
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

pub trait HasMove {
    fn get_piece_movements(&self) -> Vec<PieceMovement>;

    fn get_capture(&self) -> Option<Capture>;
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

impl HasMove for Move {
    fn get_piece_movements(&self) -> Vec<PieceMovement> {
        match &self {
            &Move::NewGame(_m) => Vec::new(),
            &Move::BasicMove(m) => m.get_piece_movements(),
            &Move::Castle(m) => m.get_piece_movements(),
            &Move::Promotion(m) => m.basic_move.get_piece_movements(),
            &Move::TwoSquarePawnMove(m) => m.basic_move.get_piece_movements(),
            &Move::EnPassant(m) => m.basic_move.get_piece_movements(),
        }
    }

    fn get_capture(&self) -> Option<Capture> {
        match &self {
            &Move::NewGame(_m) => None,
            &Move::BasicMove(m) => m.get_capture(),
            &Move::Castle(m) => m.get_capture(),
            &Move::Promotion(m) => m.basic_move.get_capture(),
            &Move::TwoSquarePawnMove(m) => m.basic_move.get_capture(),
            &Move::EnPassant(m) => m.get_capture(),
        }
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