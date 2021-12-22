use fnv::FnvHashSet;

use crate::util::{ControlFlow, FnvIndexSet, FoldHelper, UnwrapsAll};

use crate::rules::board::squares::BoardSquare;

use super::{PieceType, Piece};


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

fn build_static_vector(square: u8, vector: &MovementVector) -> FnvIndexSet<u8> {
    return FnvIndexSet::from_iter(get_squares_for_vector(square, vector).into_iter());
}

fn collect_static_vectors(square: u8, vectors: &Vec<&MovementVector>) -> Vec<FnvIndexSet<u8>> {
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
    fn movement_rays(&self) -> &Vec<FnvIndexSet<u8>>;
    fn attacked_squares(&self) -> &FnvHashSet<u8>;
    fn can_move(&self, square: &u8) -> bool;
    fn can_capture(&self, square: &u8) -> bool;
}

pub enum MovementDetail {
    Piece(&'static PieceMovementDetail),
    Pawn(&'static PawnMovementDetail),
}

impl Movement for MovementDetail {
    fn movement_rays(&self) -> &Vec<FnvIndexSet<u8>> {
        match self {
            &MovementDetail::Piece(m) => &m.rays,
            &MovementDetail::Pawn(m) => &m.all_rays,
        }
    }
    fn attacked_squares(&self) -> &FnvHashSet<u8> {
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
    pub rays: Vec<FnvIndexSet<u8>>,
    pub all_squares: FnvHashSet<u8>,
}

impl PieceMovementDetail {
    pub fn from_static_vectors(vectors: Vec<FnvIndexSet<u8>>) -> PieceMovementDetail {
        PieceMovementDetail {
            rays: vectors.clone(),
            all_squares: vectors.clone().into_iter().flatten().collect(),
        }
    }
}

#[derive(Clone, Default)]
pub struct PawnMovementDetail {
    pub advancing_rays: Vec<FnvIndexSet<u8>>,
    pub attacking_rays: Vec<FnvIndexSet<u8>>,
    all_rays: Vec<FnvIndexSet<u8>>,
    pub advancing_squares: FnvHashSet<u8>,
    pub attacking_squares: FnvHashSet<u8>,
}

impl PawnMovementDetail {
    pub fn from_rays(adv: Vec<FnvIndexSet<u8>>, att: Vec<FnvIndexSet<u8>>) -> PawnMovementDetail {
        return PawnMovementDetail {
            advancing_rays: adv.clone(),
            attacking_rays: att.clone(),
            all_rays: [adv.clone(), att.clone()].concat(),
            advancing_squares: adv.clone().into_iter().flatten().collect(),
            attacking_squares: att.clone().into_iter().flatten().collect(),
        }
    }
}


pub struct PieceMovement {
    pub start_square: u8,
    pub end_square: u8,
}

pub trait HasMove {
    fn get_piece_movements(&self) -> Vec<PieceMovement>;

    fn get_capture(&self) -> Option<Piece>;
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

    fn get_capture(&self) -> Option<Piece> {
        match &self {
            &Move::NewGame(_m) => None,
            &Move::BasicMove(m) => m.get_capture(),
            &Move::Castle(m) => m.get_capture(),
            &Move::Promotion(m) => m.basic_move.get_capture(),
            &Move::TwoSquarePawnMove(m) => m.basic_move.get_capture(),
            &Move::EnPassant(m) => m.basic_move.get_capture(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NewGame {}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BasicMove {
    pub start: u8,
    pub end: u8,
    pub capture: Option<Piece>,
}

impl HasMove for BasicMove {
    fn get_piece_movements(&self) -> Vec<PieceMovement> {
        return [ PieceMovement { start_square: self.start, end_square: self.end } ].into_iter().collect();
    }

    fn get_capture(&self) -> Option<Piece> {
        return self.capture;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Castle {
    pub side: CastleType,
    pub king_start: u8,
    pub king_end: u8,
    pub rook_start: u8,
    pub rook_end: u8,
}

impl HasMove for Castle {
    fn get_piece_movements(&self) -> Vec<PieceMovement> {
        return [
            PieceMovement { start_square: self.king_start, end_square: self.king_end },
            PieceMovement { start_square: self.rook_start, end_square: self.rook_end },
        ].into_iter().collect()
    }

    fn get_capture(&self) -> Option<Piece> {
        return None
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Promotion {
    pub promote_to: PieceType,
    pub basic_move: BasicMove,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TwoSquarePawnMove {
    pub en_passant_target: u8,
    pub basic_move: BasicMove,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EnPassant {
    pub capture_square: u8,
    pub basic_move: BasicMove,
}