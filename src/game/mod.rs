use crate::{engine::Engine, rules::{Color, pieces::{Piece, movement::Move}, board::Board}};


pub struct Game {
    board: Board,
    engine: Engine,
    move_history: Vec<Move>,
    engine_depth: u8,
}

impl Game {
    pub fn new(engine_depth: u8) -> Game {
        let board = Board::from_starting_position();
        return Game {
            board: board.clone(),
            engine: Engine::new(board, engine_depth),
            move_history: Vec::new(),
            engine_depth: engine_depth,
        }
    }

    pub fn from_fen(fen: &str, engine_depth: u8) -> Game {
        let board = Board::from_fen(fen);
        return Game {
            board: board.clone(),
            engine: Engine::new(board, engine_depth),
            move_history: Vec::new(),
            engine_depth: engine_depth, 
        }
    }

    pub fn get_engine_size(&self) -> u32 {
        return *self.engine.get_size();
    }

    pub fn get_engine_depths(&self) -> Vec<u32> {
        self.engine.get_sized_depth().iter().map(|(_depth, size)| *size).collect()
    }

    pub fn suggest_move(&self) -> &Move {
        return self.engine.suggest();
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.board.make_move(new_move);
        self.move_history.push(*new_move);
        self.engine = Engine::new(self.board.clone(), self.engine_depth);
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