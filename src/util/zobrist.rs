use std::{hash::{Hasher, BuildHasher}, collections::HashMap};

use fnv::FnvHashMap;
use rand::{SeedableRng, RngCore};
use rand_pcg::Mcg128Xsl64;

use crate::rules::{pieces::{PieceType, movement::{CastleType, Move, HasMove}}, board::CastleRight, Color};

pub type ZobristHashMap<T> = HashMap<u64, T, BuildZobristHasher>;


static ZOBRIST_SEED: [u8; 16] = [
    240, 222,  77,  56,
    169, 194, 104, 138,
    212, 109,  14, 241,
    158,  91, 205,  73,
];


static TO_MOVE_KEY_OFFSET: u16    = 0;
static WHITE_KEY_OFFSET: u16      = 10000;
static BLACK_KEY_OFFSET: u16      = 20000;
static CASTLE_KEY_OFFSET: u16     = 0;
static KINGSIDE_KEY_OFFSET: u16   = 0;
static QUEENSIDE_KEY_OFFSET: u16  = 1;
static PAWN_KEY_OFFSET: u16       = 100;
static KNIGHT_KEY_OFFSET: u16     = 200;
static BISHOP_KEY_OFFSET: u16     = 300;
static ROOK_KEY_OFFSET: u16       = 400;
static QUEEN_KEY_OFFSET: u16      = 500;
static KING_KEY_OFFSET: u16       = 600;
static EN_PASSANT_KEY_OFFSET: u16 = 1000;


lazy_static! {
    static ref ZOBRIST_RANDOMS: FnvHashMap<u16, u64> = generate_zobrist_randoms();
}


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

#[derive(Default)]
pub struct BuildZobristHasher;

impl BuildHasher for BuildZobristHasher {
    type Hasher = ZobristHasher;
    fn build_hasher(&self) -> Self::Hasher {
        return ZobristHasher { state: 0 }
    }
}


fn get_key_for_change(change: BoardChange) -> u16 {
    return match change {
        BoardChange::BlackToMove => TO_MOVE_KEY_OFFSET,
        BoardChange::PieceLocation(piece_at) => {
            let color_offset = match piece_at.color { Color::White => WHITE_KEY_OFFSET, Color::Black => BLACK_KEY_OFFSET };
            let piece_offset = match piece_at.piece_type {
                PieceType::Pawn => PAWN_KEY_OFFSET,
                PieceType::Knight => KNIGHT_KEY_OFFSET,
                PieceType::Bishop => BISHOP_KEY_OFFSET,
                PieceType::Rook => ROOK_KEY_OFFSET,
                PieceType::Queen => QUEEN_KEY_OFFSET,
                PieceType::King => KING_KEY_OFFSET,
            };
            color_offset + piece_offset + piece_at.square as u16
        },
        BoardChange::CastleRight(rights) => {
            let color_offset = match rights.color { Color::White => WHITE_KEY_OFFSET, Color::Black => BLACK_KEY_OFFSET };
            let side_offset = match rights.side { CastleType::Kingside => KINGSIDE_KEY_OFFSET, CastleType::Queenside => QUEENSIDE_KEY_OFFSET};
            color_offset + side_offset + CASTLE_KEY_OFFSET
        },
        BoardChange::EnPassantTarget(square) => EN_PASSANT_KEY_OFFSET + square as u16
    }
}


fn generate_zobrist_randoms() -> FnvHashMap<u16, u64> {
    let mut rng: Mcg128Xsl64 = Mcg128Xsl64::from_seed(ZOBRIST_SEED);
    let mut hashes: FnvHashMap<u16, u64> = Default::default();

    let black_to_move_key = get_key_for_change(BoardChange::BlackToMove);
    hashes.insert(black_to_move_key, rng.next_u64());

    [Color::White, Color::Black].into_iter().for_each(|color| {
        [CastleType::Kingside, CastleType::Queenside].into_iter().for_each(|side| {
            let castle_key = get_key_for_change(BoardChange::CastleRight(CastleRight { color: color, side: side }));
            hashes.insert(castle_key, rng.next_u64());
        })
    });

    (0..=63).for_each(|square| {
        let en_passant_key = get_key_for_change(BoardChange::EnPassantTarget(square));
        hashes.insert(en_passant_key, rng.next_u64());
        [Color::White, Color::Black].into_iter().for_each(|color| {
            [PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King].into_iter().for_each(|piece_type| {
                let piece_key = get_key_for_change(BoardChange::PieceLocation(PieceLocation { square: square, color: color, piece_type: piece_type }));
                hashes.insert(piece_key, rng.next_u64());
            })
        });
    });

    return hashes;
}


fn get_random_from_change(change: BoardChange) -> u64 {
    return ZOBRIST_RANDOMS.get(&get_key_for_change(change)).map(|r| *r).unwrap();
}


pub fn zobrist_init(changes: Vec<BoardChange>) -> u64 {
    let mut state = 0;
    for change in changes {
        state = zobrist_update_in(0, change);
    }
    return state;
}


pub fn zobrist_update_in(hash: u64, change: BoardChange) -> u64 {
    return hash ^ get_random_from_change(change);
}


pub fn zobrist_update_out(hash: u64, change: BoardChange) -> u64 {
    return get_random_from_change(change) ^ hash;
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
