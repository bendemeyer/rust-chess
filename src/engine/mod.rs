use std::{sync::Mutex, time::{Duration, Instant}, cmp::Ordering};

use crate::{rules::{board::Board, pieces::movement::{Move, NullMove}, Color}, util::{zobrist::ZobristHashMap}};

use self::{scores::{best_score, is_better}, evaluation::evaluate_board};

pub mod evaluation;
pub mod scores;


impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.relative_capture_value(), other.relative_capture_value()) {
            (Some(val), Some(other_val)) => {
                if val > other_val { Ordering::Greater }
                else if val < other_val { Ordering::Less }
                else { Ordering::Equal }
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}


pub struct SearchResult {
    color: Color,
    best_move: Move,
    best_move_score: i16,
    pub calculated_nodes: u32,
    pub cache_hits: u32,
    start_time: Option<Instant>,
    pub search_time: Duration,
}

impl SearchResult {
    pub fn from_color(color: Color) -> Self {
        Self {
            color: color,
            best_move: Move::NullMove(NullMove {}),
            best_move_score: best_score(color.swap()),
            calculated_nodes: 0,
            cache_hits: 0,
            start_time: None,
            search_time: Default::default(),
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn complete(&mut self) {
        if let Some(start) = self.start_time {
            self.search_time = start.elapsed();
        }
    }

    pub fn process_move(&mut self, new_move: Move, score: i16) {
        if is_better(score, self.best_move_score, self.color) {
            self.best_move_score = score;
            self.best_move = new_move;
        }
    }

    pub fn get_score(&self) -> i16 {
        return self.best_move_score;
    }

    pub fn get_move(&self) -> &Move {
        return &self.best_move;
    }
}


struct SearchContext {
    board: Board,
    calculated_nodes: u32,
    cache_hits: u32,
}

impl SearchContext {
    pub fn get_best(&self, new: i16, old: i16) -> i16 {
        match self.board.state.get_move_color() {
            Color::White => if new > old { new } else { old },
            Color::Black => if new < old { new } else { old }
        }
    }

    pub fn is_better(&self, new: i16, old: i16) -> bool {
        match self.board.state.get_move_color() {
            Color::White => new > old,
            Color::Black => new < old
        }
    }
}


pub struct Engine;

impl Engine {

    pub fn do_search(board: Board, depth: u8) -> SearchResult {
        let transposition_table: Mutex<ZobristHashMap<i16>> = Mutex::new(Default::default());
        let mut result = SearchResult::from_color(board.state.get_move_color());
        result.start();
        let mut moves = board.get_legal_moves();
        moves.sort();
        for m in moves.into_iter().rev() {
            let mut updated_board = board.clone();
            updated_board.make_move(&m);

            let mut ctx = SearchContext {
                board: updated_board,
                cache_hits: 0,
                calculated_nodes: 0,
            };
            let score = Self::search(
                best_score(board.state.get_move_color()),
                result.get_score(),
                depth - 1,
                &transposition_table,
                &mut ctx);
            result.process_move(m, score);
            result.calculated_nodes += ctx.calculated_nodes;
            result.cache_hits += ctx.cache_hits;
        }
        result.complete();
        return result;
    }

    fn search(mut best_forcible: i16, opponent_best_forcible: i16, depth: u8, transpositions: &Mutex<ZobristHashMap<i16>>, ctx: &mut SearchContext) -> i16 {
        if depth <= 0 {
            ctx.calculated_nodes += 1;
            return evaluate_board(&ctx.board)
        }
        let mut moves = ctx.board.get_legal_moves();
        moves.sort();
        for m in moves.into_iter().rev() {
            let change = ctx.board.make_move(&m);
            let cache_hit: Option<i16>;
            {
                let map = transpositions.lock().unwrap();
                cache_hit = map.get(&ctx.board.zobrist.get_id()).map(|s| *s);
            }
            let score = match cache_hit {
                Some(cached_score) => {
                    ctx.cache_hits += 1;
                    cached_score
                },
                None => {
                    let calculated_score =  Self::search(opponent_best_forcible, best_forcible, depth - 1, transpositions, ctx);
                    {
                        let mut map = transpositions.lock().unwrap();
                        map.insert(ctx.board.zobrist.get_id(), calculated_score);
                    }
                    calculated_score
                },
            };
            ctx.board.unmake_move(change);
            if ctx.is_better(score, opponent_best_forcible) { return opponent_best_forcible; }
            best_forcible = ctx.get_best(score, best_forcible);
        }
        return best_forcible;
    }
}
