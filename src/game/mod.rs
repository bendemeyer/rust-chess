use crate::{engine::Engine, rules::{Color, pieces::{Piece, movement::Move}}};


pub struct Game {
    engine: Engine,
    move_history: Vec<Move>,
}

impl Game {
    pub fn new(engine_depth: u8) -> Game {
        return Game {
            engine: Engine::new(engine_depth),
            move_history: Vec::new(),
        }
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.engine.make_move(new_move);
        self.move_history.push(new_move.clone());
    }

    pub fn get_legal_moves(&self) -> Vec<&Move> {
        return self.engine.get_moves();
    }

    pub fn suggest_moves(&self, n: u8) -> Vec<&Move> {
        return self.engine.suggest_moves(n);
    }

    pub fn suggest_move(&self) -> &Move {
        return self.suggest_moves(1).first().unwrap()
    }
 
    pub fn get_engine_size(&self) -> u64 {
        return self.engine.size();
    }

    pub fn get_piece_at(&self, square: u8) -> Option<&Piece> {
        return self.engine.piece_at(square);
    }

    pub fn get_current_turn(&self) -> Color {
        return self.engine.turn();
    }
}