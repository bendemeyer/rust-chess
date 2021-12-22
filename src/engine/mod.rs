use std::collections::BTreeMap;

use crate::rules::{board::Board, pieces::movement::Move, Color};

use self::evaluation::evaluate_board;

pub mod evaluation;


fn max(a: Option<i16>, b: i16) -> Option<i16> {
    match a {
        Some(s) => if s > b { Some(s) } else { Some(b) },
        None => Some(b)
    }
}

fn min(a: Option<i16>, b: i16) -> Option<i16> {
    match a {
        Some(s) => if s < b { Some(s) } else { Some(b) },
        None => Some(b)
    }
}


pub struct Engine {
    board: Board,
    depth: u8,
    moves: BTreeMap<i16, Move>,
    size: u32,
    size_by_depth: BTreeMap<u8, u32>,
}

impl Engine {
    pub fn new(board: Board, depth: u8) -> Engine {
        let mut engine = Engine {
            board: board,
            depth: depth,
            moves: BTreeMap::new(),
            size: 0,
            size_by_depth: BTreeMap::new(),
        };
        engine.analyze(0, engine.selector());
        return engine;
    }

    pub fn get_size(&self) -> &u32 {
        return &self.size;
    }

    pub fn get_sized_depth(&self) -> &BTreeMap<u8, u32> {
        return &self.size_by_depth;
    }

    pub fn suggest(&self) -> &Move {
        match self.board.state.get_move_color() {
            Color::White => self.moves.iter().next().unwrap().1,
            Color::Black => self.moves.iter().next_back().unwrap().1,
        }
    }

    fn analyze(&mut self, depth: u8, selector: fn(Option<i16>, i16) -> Option<i16>) -> i16 {
        self.size += 1;
        self.size_by_depth.insert(depth, self.size_by_depth.get(&depth).unwrap_or(&0u32) + 1);
        if depth > self.depth {
            return evaluate_board(&self.board);
        }
        let moves = self.board.get_legal_moves();
        let mut score: Option<i16> = None;
        for new_move in moves {
            let change = self.board.make_move(&new_move);
            score = selector(score, self.analyze(depth + 1, self.selector()));
            if depth == 0 {
                self.moves.insert(score.unwrap(), new_move);
            }
            self.board.unmake_move(change);
        }
        return match score {
            Some(s) => s,
            None => evaluate_board(&self.board)
        }
    } 

    fn selector(&self) -> fn(Option<i16>, i16) -> Option<i16> {
        match self.board.state.get_move_color() {
            Color::White => max,
            Color::Black => min
        }
    }
}
