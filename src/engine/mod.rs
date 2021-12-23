use std::collections::BTreeMap;

use crate::{rules::{board::Board, pieces::movement::{Move, HasMove}, Color}, game::Perft};

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
}

impl Engine {
    pub fn new(board: Board, depth: u8, perft: &mut Perft) -> Engine {
        let mut engine = Engine {
            board: board,
            depth: depth,
            moves: BTreeMap::new(),
        };
        engine.analyze(0, engine.selector(), perft);
        return engine;
    }

    pub fn suggest(&self) -> &Move {
        match self.board.state.get_move_color() {
            Color::White => self.moves.iter().next_back().unwrap().1,
            Color::Black => self.moves.iter().next().unwrap().1,
        }
    }

    fn analyze(&mut self, depth: u8, selector: fn(Option<i16>, i16) -> Option<i16>, perft: &mut Perft) -> i16 {
        perft.increment_size(depth);
        if depth >= self.depth {
            return evaluate_board(&self.board);
        }
        let moves = self.board.get_legal_moves();
        let mut score: Option<i16> = None;
        for new_move in moves {
            match new_move.get_capture() {
                Some(_) => perft.increment_captures(depth + 1),
                None => ()
            }
            match new_move {
                Move::EnPassant(_) => perft.increment_en_passants(depth + 1),
                Move::Promotion(_) => perft.increment_promotions(depth + 1),
                Move::Castle(_) => perft.increment_castles(depth + 1),
                _ => (),
            }
            let change = self.board.make_move(&new_move);
            if self.board.in_check() { perft.increment_checks(depth + 1); }
            score = selector(score, self.analyze(depth + 1, self.selector(), perft));
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
