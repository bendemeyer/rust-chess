use std::{hash::{Hasher, BuildHasher}, collections::HashMap};

use crate::rules::{pieces::{PieceType, movement::{CastleType, Move, HasMove}}, board::CastleRight, Color};


pub type ZobristHashMap<T> = HashMap<u64, T, BuildZobristHasher>;


static TO_MOVE_BIT: u64       = 2u64.pow(63);
static BASE_OFFSET: u64       = 9884226941274182611;
static WHITE_OFFSET: u64      = 0;
static BLACK_OFFSET: u64      = 9324229921666102;
static SQUARE_MULTIPLIER: u64 = 18648459843332204;
static PAWN_OFFSET: u64       = 0;
static KNIGHT_OFFSET: u64     = SQUARE_MULTIPLIER * 64;
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

pub struct PieceLocation {
    pub color: Color,
    pub piece_type: PieceType,
    pub square: u8,
}


pub struct ZobristHasher {
    pub state: u64,
}

impl Hasher for ZobristHasher {
    fn finish(&self) -> u64 {
        return self.state;
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("Hopefully this isn't used!")
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
        BoardChange::PieceLocation(piece_at) => {
            let color_offset = match piece_at.color { Color::White => WHITE_OFFSET, Color::Black => BLACK_OFFSET };
            let piece_offset = match piece_at.piece_type {
                PieceType::Pawn => PAWN_OFFSET,
                PieceType::Knight => KNIGHT_OFFSET,
                PieceType::Bishop => BISHOP_OFFSET,
                PieceType::Rook => ROOK_OFFSET,
                PieceType::Queen => QUEEN_OFFSET,
                PieceType::King => KING_OFFSET,
            };
            BASE_OFFSET + color_offset + piece_offset + (SQUARE_MULTIPLIER * piece_at.square as u64)
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


pub fn zobrist_init(changes: Vec<BoardChange>) -> u64 {
    let mut state = 0;
    for change in changes {
        state = zobrist_update_in(0, change);
    }
    return state;
}


pub fn zobrist_update_in(hash: u64, change: BoardChange) -> u64 {
    return hash ^ get_adjustment_for_change(change);
}


pub fn zobrist_update_out(hash: u64, change: BoardChange) -> u64 {
    return get_adjustment_for_change(change) ^ hash;
}


pub fn zobrist_update_apply_move(hash: u64, new_move: &Move) -> u64 {
    let mut state = hash;
    for movement in new_move.get_piece_movements() {
        state = zobrist_update_out(state, BoardChange::PieceLocation(PieceLocation {
            color: movement.color,
            piece_type: movement.piece_type,
            square: movement.start_square
        }));
        state = zobrist_update_in(state, BoardChange::PieceLocation(PieceLocation {
            color: movement.color,
            piece_type: movement.piece_type,
            square: movement.end_square
        }));
    }
    match new_move.get_capture() {
        Some(c) => {
            state = zobrist_update_out(state, BoardChange::PieceLocation(PieceLocation {
                color: c.color,
                piece_type: c.piece_type,
                square: c.square
            }));
        },
        None => ()
    }
    return state;
}

pub fn zobrist_update_unapply_move(hash: u64, new_move: &Move) -> u64 {
    let mut state = hash;
    for movement in new_move.get_piece_movements() {
        state = zobrist_update_in(state, BoardChange::PieceLocation(PieceLocation {
            color: movement.color,
            piece_type: movement.piece_type,
            square: movement.start_square
        }));
        state = zobrist_update_out(state, BoardChange::PieceLocation(PieceLocation {
            color: movement.color,
            piece_type: movement.piece_type,
            square: movement.end_square
        }));
    }
    match new_move.get_capture() {
        Some(c) => {
            state = zobrist_update_in(state, BoardChange::PieceLocation(PieceLocation {
                color: c.color,
                piece_type: c.piece_type,
                square: c.square
            }));
        },
        None => ()
    }
    return state;
}


pub fn zobrist_update_turn(hash: u64, color: Color) -> u64 {
    return match color {
        Color::Black => zobrist_update_in(hash, BoardChange::BlackToMove),
        Color::White => zobrist_update_out(hash, BoardChange::BlackToMove),
    }
}


pub fn zobrist_update_add_en_passant_target(hash: u64, square: u8) -> u64 {
    return zobrist_update_in(hash, BoardChange::EnPassantTarget(square));
}

pub fn zobrist_update_remove_en_passant_target(hash: u64, square: u8) -> u64 {
    return zobrist_update_out(hash, BoardChange::EnPassantTarget(square));
}


pub fn zobrist_update_gain_castle_right(hash: u64, color: Color, side: CastleType) -> u64 {
    return zobrist_update_in(hash, BoardChange::CastleRight(CastleRight { color: color, side: side }))
}

pub fn zobrist_update_lose_castle_right(hash: u64, color: Color, side: CastleType) -> u64 {
    return zobrist_update_out(hash, BoardChange::CastleRight(CastleRight { color: color, side: side }))
}
