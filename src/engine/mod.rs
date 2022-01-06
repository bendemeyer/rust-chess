use std::collections::BTreeMap;

use crate::{game::Perft, rules::{board::Board, pieces::movement::{Move, HasMove}, Color}, util::zobrist::ZobristHashMap};

use self::evaluation::evaluate_board;

pub mod evaluation;


struct SearchContext {
    board: Board,
    transpositions: ZobristHashMap<i16>,
    results: BTreeMap<i16, Move>,
    max_depth: u8,
    cache_hits: u64,
    calculated_nodes: u64,
}

impl SearchContext {
    pub fn get_best(&self, new: i16, old: i16) -> i16 {
        match self.board.state.get_move_color() {
            Color::White => if new > old { new } else { old },
            Color::Black => if new < old { new } else { old }
        }
    }

    pub fn is_worse(&self, new: i16, old: i16) -> bool {
        match self.board.state.get_move_color() {
            Color::White => new < old,
            Color::Black => new > old
        }
    }

    pub fn best_possible(&self) -> i16 {
        match self.board.state.get_move_color() {
            Color::White => i16::MAX,
            Color::Black => i16::MIN,
        }
    }

    pub fn worst_possible(&self) -> i16 {
        match self.board.state.get_move_color() {
            Color::White => i16::MIN,
            Color::Black => i16::MAX,
        }
    }
}


pub struct Engine;

impl Engine {

    pub fn do_perft(mut board: Board, depth: u8, perft: &mut Perft) {
        perft.zobrist_start = board.id;
        perft.start();
        Self::perft(&mut board, 0, depth, perft);
        perft.complete();
        perft.zobrist_end = board.id;
    }

    fn perft(board: &mut Board, depth: u8, max_depth: u8, perft: &mut Perft) {
        perft.increment_size(depth);
        if depth >= max_depth {
            return;
        }
        let moves = board.get_legal_moves();
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
            let change = board.make_move(&new_move);
            //if board.in_check() { perft.increment_checks(depth + 1); }
            Self::perft(board, depth + 1, max_depth, perft);
            board.unmake_move(change);
        }
    }

    pub fn do_search(board: Board, depth: u8) -> BTreeMap<i16, Move> {
        let mut ctx = SearchContext {
            board: board,
            transpositions: Default::default(),
            results: Default::default(),
            max_depth: depth,
            cache_hits: 0,
            calculated_nodes: 0,
        };
        Self::search(ctx.best_possible(), ctx.worst_possible(), 0, &mut ctx);
        println!("Seach had {} cache hits", ctx.cache_hits);
        println!("Search calculated {} nodes", ctx.calculated_nodes);
        return ctx.results;
    }

    fn search(mut best: i16, worst: i16, depth: u8, ctx: &mut SearchContext) -> i16 {
        if depth >= ctx.max_depth {
            ctx.calculated_nodes += 1;
            return evaluate_board(&ctx.board)
        }
        let moves = ctx.board.get_legal_moves();
        for m in moves {
            let change = ctx.board.make_move(&m);
            let score = match ctx.transpositions.get(&ctx.board.id) {
                Some(cached_score) => {
                    ctx.cache_hits += 1;
                    *cached_score
                },
                None => {
                    let calculated_score = Self::search(worst, best, depth + 1, ctx);
                    ctx.transpositions.insert(ctx.board.id, calculated_score);
                    calculated_score
                },
            };
            if depth == 0 {
                ctx.results.insert(score, m);
            }
            ctx.board.unmake_move(change);
            if ctx.is_worse(score, worst) { return worst; }
            best = ctx.get_best(score, best);
        }
        return best;
    }
}
