use std::{collections::BTreeMap, time::{Instant, Duration}};

use tabled::Tabled;

use crate::{engine::Engine, rules::{Color, pieces::{Piece, movement::Move}, board::Board}};


enum PerftType {
    Size,
    Captures,
    EnPassants,
    Castles,
    Promotions,
    Checks,
}


#[derive(Default)]
pub struct Perft {
    levels: Vec<LevelPerft>,
    pub cache_hits: u32,
    pub zobrist_start: u64,
    pub zobrist_end: u64,
    start: Option<Instant>,
    pub duration: Duration,
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
            PerftType::EnPassants => analysis_level.en_passants += 1,
            PerftType::Castles    => analysis_level.castles += 1,
            PerftType::Promotions => analysis_level.promotions += 1,
            PerftType::Checks     => analysis_level.checks += 1,
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

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn complete(&mut self) {
        match self.start {
            Some(i) => self.duration = i.elapsed(),
            None => ()
        }
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
    move_history: Vec<Move>,
    suggestions: BTreeMap<i16, Move>,
}

impl Game {
    pub fn new() -> Self {
        return Self::from_board(Board::from_starting_position());
    }

    pub fn from_fen(fen: &str) -> Self {
        return Self::from_board(Board::from_fen(fen));
    }

    fn from_board(board: Board) -> Self {
        return Self {
            board: board,
            move_history: Vec::new(),
            suggestions: Default::default(),
        }
    }

    pub fn search(&mut self, depth: u8) {
        self.suggestions = Engine::do_search(self.board.clone(), depth);
    }

    pub fn suggest_move(&self) -> &Move {
        return match self.board.state.get_move_color() {
            Color::White => self.suggestions.iter().next_back().unwrap().1,
            Color::Black => self.suggestions.iter().next().unwrap().1,
        }
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.board.make_move(new_move);
        self.move_history.push(*new_move);
        self.suggestions = Default::default()
    }

    pub fn do_perft(&mut self, depth: u8) -> Perft {
        let mut perft: Perft = Default::default();
        Engine::do_perft(self.board.clone(), depth, &mut perft);
        return perft;
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        return self.board.get_legal_moves();
    }

    pub fn get_piece_at(&self, square: u8) -> Option<&Piece> {
        return self.board.position.piece_at(&square);
    }

    pub fn get_current_turn(&self) -> Color {
        return self.board.state.get_move_color();
    }

    pub fn serialize_board(&self) -> String {
        return self.board.to_fen();
    }
}