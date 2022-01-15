use crate::rules::{Color, pieces::movement::{CastleType, Move}};

use super::{bitboards::get_bit_for_square, positions::{Attack, Pin, BoardPosition}};


#[derive(Clone)]
pub struct ApplyableBoardChange {
    pub new_move: Move,
    pub checks: Vec<Attack>,
    pub absolute_pins: Vec<Pin>,
    pub pinned_pieces: u64,
    pub responses: Vec<ApplyableBoardChange>,
    pub new_zobrist_id: u64,
    pub new_position: BoardPosition,
    pub new_state: BoardState,
}


#[derive(Copy, Clone)]
pub struct ReversibleBoardChange {
    pub prior_zobrist_id: u64,
    pub prior_position: BoardPosition,
    pub prior_state: BoardState,
    
}


#[derive(Copy, Clone)]
pub struct CastleRight {
    pub color: Color,
    pub side: CastleType,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoardCastles {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl BoardCastles {
    pub fn revoke_right(&mut self, right: &CastleRight) {
        match (right.color, right.side) {
            (Color::White, CastleType::Kingside)  => self.white_kingside  = false,
            (Color::White, CastleType::Queenside) => self.white_queenside = false,
            (Color::Black, CastleType::Kingside)  => self.black_kingside  = false,
            (Color::Black, CastleType::Queenside) => self.black_queenside = false,
        }
    }

    pub fn unrevoke_right(&mut self, right: &CastleRight) {
        match (right.color, right.side) {
            (Color::White, CastleType::Kingside)  => self.white_kingside  = true,
            (Color::White, CastleType::Queenside) => self.white_queenside = true,
            (Color::Black, CastleType::Kingside)  => self.black_kingside  = true,
            (Color::Black, CastleType::Queenside) => self.black_queenside = true,
        }
    }

    pub fn can_castle(&self, right: CastleRight) -> bool {
        match (right.color, right.side) {
            (Color::White, CastleType::Kingside)  => self.white_kingside,
            (Color::White, CastleType::Queenside) => self.white_queenside,
            (Color::Black, CastleType::Kingside)  => self.black_kingside,
            (Color::Black, CastleType::Queenside) => self.black_queenside,
        }
    }
}

impl Default for BoardCastles {
    fn default() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}


#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BoardState {
    pub to_move: Color,
    pub castle_rights: BoardCastles,
    pub en_passant_target: u64,
    pub move_number: u8,
    pub halfmove_clock: u8,
}

impl BoardState {
    pub fn reset_halfmove_clock(&mut self) {
        self.halfmove_clock = 0;
    }

    pub fn increment_halfmove_clock(&mut self) {
        self.halfmove_clock += 1;
    }

    pub fn increment_move_number(&mut self) {
        self.move_number += 1;
    }

    pub fn get_move_color(&self) -> Color {
        return self.to_move
    }

    pub fn change_move_color(&mut self) {
        self.to_move = self.to_move.swap();
    }

    pub fn clear_en_passant_target(&mut self) -> Option<u8> {
        let old_target = self.en_passant_target;
        self.en_passant_target = 0u64;
        return match old_target {
            0 => None,
            x => Some(x.trailing_zeros() as u8)
        }
    }

    pub fn set_en_passant_target(&mut self, square: u8) {
        self.en_passant_target = get_bit_for_square(square)
    }

    pub fn get_en_passant_target(&self) -> Option<u8> {
        return match self.en_passant_target {
            0 => None,
            x => Some(x.trailing_zeros() as u8)
        }
    }

    pub fn can_castle(&self, castle: &CastleRight) -> bool {
        match (castle.color, castle.side) {
            (Color::White, CastleType::Kingside) => self.castle_rights.white_kingside,
            (Color::White, CastleType::Queenside) => self.castle_rights.white_queenside,
            (Color::Black, CastleType::Kingside) => self.castle_rights.black_kingside,
            (Color::Black, CastleType::Queenside) => self.castle_rights.black_queenside,
        }
    }

    pub fn revoke_castle_right(&mut self, castle: &CastleRight) {
        self.castle_rights.revoke_right(castle);
    }

    pub fn return_castle_right(&mut self, castle: &CastleRight) {
        self.castle_rights.unrevoke_right(castle);
    }
}