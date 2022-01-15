use crate::{engine::{Engine, SearchResult}, rules::{Color, pieces::{Piece, movement::Move}, board::Board}, util::zobrist::ZobristHashMap};


#[derive(Copy, Clone)]
pub struct Turn {
    board: Board,
    move_played: Move,
}


#[derive(Clone)]
pub struct GameHistory {
    current_board: Board,
    turn_history: Vec<Turn>,
    repetitions: ZobristHashMap<Vec<Board>>,
}

impl GameHistory {
    pub fn new(board: Board) -> Self {
        return Self {
            current_board: board,
            turn_history: Vec::new(),
            repetitions: Default::default(),
        }
    }

    pub fn take_turn(&mut self, mov: &Move) {
        let turn = Turn { board: self.current_board, move_played: *mov };
        self.add_repetition(turn.board.id, turn.board);
        self.turn_history.push(turn);
        self.current_board.make_move(mov);
    }

    pub fn untake_turn(&mut self) {
        let last_turn = self.turn_history.pop().unwrap();
        self.repetitions.get_mut(&last_turn.board.id).unwrap().pop();
    }

    fn add_repetition(&mut self, hash: u64, board: Board) {
        match self.repetitions.get_mut(&hash) {
            Some(boards) => { boards.push(board); },
            None => { self.repetitions.insert(hash, Vec::from([board])); },
        };
    }

    fn count_fuzzy_repetitions(&self, hash: u64) -> u8 {
        return self.repetitions.get(&hash).unwrap_or(&Vec::new()).len() as u8;
    }

    fn count_exact_repetitions(&self, hash: u64) -> u8 {
        let boards = match self.repetitions.get(&hash) {
            Some(b) => b,
            None => return 0u8,
        };
        let mut identical_board_sets: Vec<Vec<&Board>> = Vec::new();
        for board in boards {
            for set in identical_board_sets.iter_mut() {
                if board.repeats(set.iter().last().unwrap()) { set.push(board) }
                continue;
            }
            identical_board_sets.push(Vec::from([board]));
        }
        return identical_board_sets.iter().fold(0u8, |max, set| {
            if set.len() as u8 > max { set.len() as u8 } else { max }
        });
    }

    fn repeats_x_or_more(&self, hash: u64, x: u8) -> bool {
        if self.count_fuzzy_repetitions(hash) >= x {
            return self.count_exact_repetitions(hash) >= x;
        } else {
            return false;
        }
    }

    pub fn has_threefold_repetition(&self, hash: u64) -> bool {
        return self.repeats_x_or_more(hash, 3)
    }

    pub fn has_fivefold_repetition(&self, hash: u64) -> bool {
        return self.repeats_x_or_more(hash, 5)
    }
}


pub struct Game {
    board: Board,
    history: GameHistory,
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
            history: GameHistory::new(board),
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
        self.history.take_turn(new_move);
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

    pub fn get_board(&self) -> &Board {
        return &self.board;
    }

    pub fn serialize_board(&self) -> String {
        return self.board.to_fen();
    }
}