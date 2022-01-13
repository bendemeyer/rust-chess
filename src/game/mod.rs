use crate::{engine::{Engine, SearchResult, Perft}, rules::{Color, pieces::{Piece, movement::Move}, board::Board}};


pub struct Game {
    board: Board,
    move_history: Vec<Move>,
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
        }
    }

    pub fn search(&mut self, depth: u8) -> SearchResult {
        return Engine::do_search(self.board.clone(), depth);
    }

    pub fn threaded_search(&mut self, depth: u8, _threads: u8) -> SearchResult {
        return Engine::do_search(self.board.clone(), depth);
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.board.make_move(new_move);
        self.move_history.push(*new_move);
    }

    pub fn perft(&self, depth: u8) -> Perft {
        return Engine::do_perft(self.board, depth);
    }

    pub fn threaded_perft(&self, depth: u8, threads: u8) -> Perft {
        return Engine::do_threaded_perft(self.board, depth, threads);
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        return self.board.get_legal_moves();
    }

    pub fn get_piece_at(&self, square: u8) -> Option<Piece> {
        return self.board.position.piece_at(&square);
    }

    pub fn get_current_turn(&self) -> Color {
        return self.board.state.get_move_color();
    }

    pub fn serialize_board(&self) -> String {
        return self.board.to_fen();
    }
}