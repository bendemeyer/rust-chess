#![allow(dead_code)]

use std::fmt::Display;

use interface::cli::Interface;
use rules::board::{BoardState, squares::BoardSquare};

use crate::rules::{Color, pieces::movement::CastleType};

#[macro_use] extern crate lazy_static;

mod engine;
mod game;
mod interface;
mod rules;
mod util;


struct RefTest {
    pub state: BoardState,
}

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "to_move: {}\n", self.get_move_color().value())?;
        write!(f, "castle_availability: {{\n")?;
        write!(f, "    white_kingside: {}\n", self.can_castle(Color::White, CastleType::Kingside))?;
        write!(f, "    white_queenside: {}\n", self.can_castle(Color::White, CastleType::Queenside))?;
        write!(f, "    black_kingside: {}\n", self.can_castle(Color::Black, CastleType::Kingside))?;
        write!(f, "    black_queenside: {}\n", self.can_castle(Color::Black, CastleType::Queenside))?;
        write!(f, "}}\n")?;
        write!(f, "en_passant_target: {}\n", match self.get_en_passant_target() { Some(s) => BoardSquare::from_value(s).get_notation_string(), None => String::from("none") })?;

        return Ok(());
    }
}

impl RefTest {
    fn new() -> RefTest {
        return RefTest {
            state: Default::default(),
        }
    }

    fn change(&mut self) {
        self.state.change_move_color();
        self.state.set_en_passant_target(20);
        self.state.disable_castle(Color::White, CastleType::Kingside);
        self.state.disable_castle(Color::White, CastleType::Queenside);
    }

    fn get_state(&self) -> &BoardState {
        return &self.state;
    }
}


fn main() {
    let mut interface = Interface::new();
    interface.init();
}
