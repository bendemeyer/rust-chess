use std::{hash::{Hasher, BuildHasher}, collections::HashMap};

use lockfree::prelude::Map;

use crate::rules::{pieces::{PieceType, movement::{CastleType, Move}, Piece}, Color, board::{state::CastleRight, squares::get_square_from_col_and_row, positions::PieceLocation}};

use super::fen::FenBoardState;


pub type ZobristHashMap<T> = HashMap<u64, T, BuildZobristHasher>;

pub type ZobristLockfreeMap<T> = Map<u64, T, BuildZobristHasher>;


static TO_MOVE_BIT: u64       = 2u64.pow(63);
static BASE_OFFSET: u64       = 1876772766;
static SQUARE_MULTIPLIER: u64 = 10216516463056589;
static WHITE_OFFSET: u64      = 0;
static BLACK_OFFSET: u64      = SQUARE_MULTIPLIER * ((64 * 7) + 2);
static PAWN_OFFSET: u64       = SQUARE_MULTIPLIER * 64 * 0;
static KNIGHT_OFFSET: u64     = SQUARE_MULTIPLIER * 64 * 1;
static BISHOP_OFFSET: u64     = SQUARE_MULTIPLIER * 64 * 2;
static ROOK_OFFSET: u64       = SQUARE_MULTIPLIER * 64 * 3;
static QUEEN_OFFSET: u64      = SQUARE_MULTIPLIER * 64 * 4;
static KING_OFFSET: u64       = SQUARE_MULTIPLIER * 64 * 5;
static EN_PASSANT_OFFSET: u64 = SQUARE_MULTIPLIER * 64 * 6;
static CASTLE_OFFSET: u64     = SQUARE_MULTIPLIER * 64 * 7;
static KINGSIDE_OFFSET: u64   = 0;
static QUEENSIDE_OFFSET: u64  = SQUARE_MULTIPLIER;


pub enum BoardChange {
    BlackToMove,
    CastleRight(CastleRight),
    EnPassantTarget(u8),
    PieceLocation(PieceLocation),
}


pub struct ZobristHasher {
    pub state: u64,
}

impl Hasher for ZobristHasher {
    fn finish(&self) -> u64 {
        return self.state;
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("Tried to hash data other than a u64 with ZobrishHasher")
    }

    fn write_u64(&mut self, i: u64) {
        self.state = i;
    }
}

#[derive(Clone, Default)]
pub struct BuildZobristHasher;

impl BuildHasher for BuildZobristHasher {
    type Hasher = ZobristHasher;
    fn build_hasher(&self) -> Self::Hasher {
        return ZobristHasher { state: 0 }
    }
}


fn get_adjustment_for_change(change: BoardChange) -> u64 {
    return match change {
        BoardChange::BlackToMove => TO_MOVE_BIT,
        BoardChange::PieceLocation(loc) => {
            let color_offset = match loc.piece.color { Color::White => WHITE_OFFSET, Color::Black => BLACK_OFFSET };
            let piece_offset = match loc.piece.piece_type {
                PieceType::Pawn => PAWN_OFFSET,
                PieceType::Knight => KNIGHT_OFFSET,
                PieceType::Bishop => BISHOP_OFFSET,
                PieceType::Rook => ROOK_OFFSET,
                PieceType::Queen => QUEEN_OFFSET,
                PieceType::King => KING_OFFSET,
            };
            BASE_OFFSET + color_offset + piece_offset + (SQUARE_MULTIPLIER * loc.square as u64)
        },
        BoardChange::EnPassantTarget(square) => {
            BASE_OFFSET + EN_PASSANT_OFFSET + (square as u64 * SQUARE_MULTIPLIER)
        },
        BoardChange::CastleRight(rights) => {
            let color_offset = match rights.color { Color::White => WHITE_OFFSET, Color::Black => BLACK_OFFSET };
            let side_offset = match rights.side { CastleType::Kingside => KINGSIDE_OFFSET, CastleType::Queenside => QUEENSIDE_OFFSET};
            BASE_OFFSET + CASTLE_OFFSET + color_offset + side_offset
        },
    }
}


#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct ZobristId {
    state: u64,
}

impl ZobristId {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_changes(changes: Vec<BoardChange>) -> Self {
        let mut id = Self::new();
        for change in changes {
            id.update(change);
        }
        return id;
    }

    pub fn from_fen(state: &FenBoardState) -> Self {
        let mut changes: Vec<BoardChange> = Vec::new();
        if state.to_move == Color::Black { changes.push(BoardChange::BlackToMove) };
        if state.en_passant.is_some() { changes.push(BoardChange::EnPassantTarget(state.en_passant.unwrap().value())) };
        if state.castling.white_kingside  { changes.push(BoardChange::CastleRight(CastleRight { color: Color::White, side: CastleType::Kingside  })) };
        if state.castling.white_queenside { changes.push(BoardChange::CastleRight(CastleRight { color: Color::White, side: CastleType::Queenside })) };
        if state.castling.black_kingside  { changes.push(BoardChange::CastleRight(CastleRight { color: Color::Black, side: CastleType::Kingside  })) };
        if state.castling.black_queenside { changes.push(BoardChange::CastleRight(CastleRight { color: Color::Black, side: CastleType::Queenside })) };
        for (row_index, row) in state.board.iter().rev().enumerate() {
            for (col_index, square) in row.iter().enumerate() {
                match square {
                    Some(piece) => {
                        changes.push(BoardChange::PieceLocation(PieceLocation {
                            square: get_square_from_col_and_row(col_index as u8, row_index as u8),
                            piece: *piece,
                        }));
                    },
                    None => ()
                }
            }
        }
        return Self::from_changes(changes);
    }

    pub fn get_id(&self) -> u64 {
        self.state
    }

    fn update(&mut self, change: BoardChange) {
        self.state = self.state ^ get_adjustment_for_change(change);
    }

    pub fn update_turn(&mut self) {
        self.update(BoardChange::BlackToMove)
    }
    
    pub fn update_en_passant(&mut self, square: u8) {
        self.update(BoardChange::EnPassantTarget(square));
    }
    
    pub fn update_castle_right(&mut self, right: CastleRight) {
        self.update(BoardChange::CastleRight(right));
    }
    
    pub fn update_move(&mut self, new_move: &Move) {
        for movement in new_move.get_piece_movements() {
            self.update(BoardChange::PieceLocation(PieceLocation {
                square: movement.start_square,
                piece: movement.get_piece(),
            }));
            self.update(BoardChange::PieceLocation(PieceLocation {
                square: movement.end_square,
                piece: match new_move {
                    Move::Promotion(p) => Piece { color: movement.color, piece_type: p.promote_to },
                    _ => movement.get_piece(),
                },
            }));
        }
        if let Some(c) = new_move.get_capture() {
            self.update(BoardChange::PieceLocation(PieceLocation {
                square: c.square,
                piece: c.get_piece(),
            }));
        }
    }
}
