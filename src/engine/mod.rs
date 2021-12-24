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


fn get_best(a: i16, b: i16, color: Color) -> i16 {
    match color {
        Color::White => if a > b { a } else { b },
        Color::Black => if a < b { a } else { b }
    }
}


fn is_worse(new: i16, old: i16, color: Color) -> bool {
    match color {
        Color::White => new < old,
        Color::Black => new > old
    }
}


pub struct Engine {
    board: Board,
    depth: u8,
    moves: BTreeMap<i16, Move>,
}

impl Engine {
    pub fn new(board: Board, depth: u8) -> Engine {
        let mut engine = Engine {
            board: board,
            depth: depth,
            moves: BTreeMap::new(),
        };
        engine.search(
            match engine.board.state.get_move_color() { Color::White => i16::MIN, Color::Black => i16::MAX },
            match engine.board.state.get_move_color() { Color::White => i16::MAX, Color::Black => i16::MIN },
            0u8,
            engine.board.state.get_move_color());
        return engine;
    }

    pub fn suggest(&self) -> &Move {
        match self.board.state.get_move_color() {
            Color::White => self.moves.iter().next_back().unwrap().1,
            Color::Black => self.moves.iter().next().unwrap().1,
        }
    }

    pub fn do_perft(&mut self, depth: u8, perft: &mut Perft) {
        let old_depth = self.depth;
        self.depth = depth;
        println!("Starting Zobrist ID: {}", self.board.id);
        self.analyze(0, perft, self.selector());
        println!("Ending Zobrist ID: {}", self.board.id);
        self.depth = old_depth;
    }

    fn analyze(&mut self, depth: u8, perft: &mut Perft, selector: fn(Option<i16>, i16) -> Option<i16>) -> i16 {
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
            score = selector(score, self.analyze(depth + 1, perft, self.selector()));
            self.board.unmake_move(change);
        }
        return match score {
            Some(s) => s,
            None => evaluate_board(&self.board)
        }
    }

    fn search(&mut self, mut best_score: i16, worst_score: i16, depth: u8, color: Color) -> i16 {
        if depth >= self.depth {
            return evaluate_board(&self.board)
        }
        let moves = self.board.get_legal_moves();
        for m in moves {
            let change = self.board.make_move(&m);
            let score = self.search(worst_score, best_score, depth + 1, color.swap());
            self.board.unmake_move(change);
            if depth == 0 {
                self.moves.insert(score, m);
            }
            if is_worse(score, worst_score, color) { return worst_score; }
            best_score = get_best(score, best_score, color);
        }
        return best_score;
    }

    fn selector(&self) -> fn(Option<i16>, i16) -> Option<i16> {
        match self.board.state.get_move_color() {
            Color::White => max,
            Color::Black => min
        }
    }
}
