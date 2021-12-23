use tabled::Tabled;

use crate::{engine::Engine, rules::{Color, pieces::{Piece, movement::Move}, board::Board}};


enum PerftType {
    Size,
    Captures,
    Checks,
    EnPassants,
    Promotions,
    Castles,
}


#[derive(Default)]
pub struct Perft {
    levels: Vec<LevelPerft>,
}

impl Perft {
    fn create_and_increment(&mut self, level: u8, analysis_type: PerftType) {
        while self.levels.len() <= level as usize {
            self.levels.push(Default::default());
        }
        let mut analysis_level = self.levels.iter_mut().nth(level as usize).unwrap();
        match analysis_type {
            PerftType::Size       => analysis_level.size += 1,
            PerftType::Captures   => analysis_level.captures += 1,
            PerftType::Checks     => analysis_level.checks += 1,
            PerftType::EnPassants => analysis_level.en_passants += 1,
            PerftType::Promotions => analysis_level.promotions += 1,
            PerftType::Castles    => analysis_level.castles += 1,
        };
    }

    pub fn increment_size(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::Size);
    }

    pub fn increment_captures(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::Captures);
    }

    pub fn increment_checks(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::Checks);
    }

    pub fn increment_en_passants(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::EnPassants);
    }

    pub fn increment_promotions(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::Promotions);
    }

    pub fn increment_castles(&mut self, level: u8) {
        self.create_and_increment(level, PerftType::Castles);
    }

    pub fn get_analysis(&self) -> Vec<&LevelPerft> {
        return self.levels.iter().collect();
    }
}


#[derive(Default, Tabled)]
pub struct LevelPerft {
    pub size: u32,
    pub captures: u32,
    pub checks: u32,
    pub en_passants: u32,
    pub promotions: u32,
    pub castles: u32,
}


pub struct Game {
    board: Board,
    engine: Engine,
    move_history: Vec<Move>,
    engine_depth: u8,
    perft: Perft,
}

impl Game {
    pub fn new(engine_depth: u8) -> Self {
        return Self::from_board(Board::from_starting_position(), engine_depth);
    }

    pub fn from_fen(fen: &str, engine_depth: u8) -> Self {
        return Self::from_board(Board::from_fen(fen), engine_depth);
    }

    fn from_board(board: Board, engine_depth: u8) -> Self {
        let mut perft: Perft = Default::default();
        let engine = Engine::new(board.clone(), engine_depth, &mut perft);
        return Self {
            board: board,
            engine: engine,
            move_history: Vec::new(),
            engine_depth: engine_depth,
            perft: perft,
        }
    }

    pub fn get_perft(&self) -> Vec<&LevelPerft> {
        return self.perft.get_analysis();
    }

    pub fn suggest_move(&self) -> &Move {
        return self.engine.suggest();
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.board.make_move(new_move);
        self.move_history.push(*new_move);
        let mut perft: Perft = Default::default();
        self.engine = Engine::new(self.board.clone(), self.engine_depth, &mut perft);
        self.perft = perft;
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        return self.board.get_legal_moves();
    }

    pub fn get_piece_at(&self, square: u8) -> Option<&Piece> {
        return self.board.piece_locations.piece_at(&square);
    }

    pub fn get_current_turn(&self) -> Color {
        return self.board.state.get_move_color();
    }

    pub fn serialize_board(&self) -> String {
        return self.board.to_fen();
    }
}