use std::{sync::{Mutex, Arc}, time::{Duration, Instant}, cmp::Ordering};

use crossbeam_channel::unbounded;
use tabled::Tabled;

use crate::{rules::{board::Board, pieces::movement::{Move, NewGame}, Color}, util::{zobrist::ZobristHashMap, concurrency::{ThreadPool, Job}}};

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

    pub fn merge(&mut self, other: &Self) {
        while self.levels.len() < other.levels.len() {
            self.levels.push(Default::default())
        }
        self.levels.iter_mut().enumerate().for_each(|(i, lp)| {
            lp.merge(other.levels.iter().nth(i).unwrap_or(&Default::default()));
        });
    }
}


#[derive(Default, Tabled)]
pub struct LevelPerft {
    pub size: u32,
    pub captures: u32,
    pub en_passants: u32,
    pub castles: u32,
    pub promotions: u32,
    pub checks: u32,
}

impl LevelPerft {
    pub fn merge(&mut self, other: &Self) {
        self.size        += other.size;
        self.captures    += other.captures;
        self.en_passants += other.en_passants;
        self.castles     += other.castles;
        self.promotions  += other.promotions;
        self.checks      += other.checks;
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
            best_move: Move::NewGame(NewGame {}),
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


struct ThreadSearchResult {
    initial_move: Move,
    score: i16,
    calculated_nodes: u32,
    cache_hits: u32,
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

    pub fn do_threaded_perft(board: Board, depth: u8, threads: u8) -> Perft {
        let mut thread_pool = ThreadPool::new();
        thread_pool.init(threads);
        let passable_pool = Arc::new(thread_pool);
        let result = Self::threaded_perft(board, 0, depth, Arc::clone(&passable_pool));
        Arc::try_unwrap(passable_pool).unwrap_or_else(|_| panic!("Failed joining threads")).join();
        return result;
    }

    pub fn do_perft(mut board: Board, depth: u8) -> Perft {
        return Self::perft(&mut board, 0, depth);
    }

    fn threaded_perft(board: Board, depth: u8, max_depth: u8, thread_pool: Arc<ThreadPool<Perft>>) -> Perft {
        let mut perft: Perft = Default::default();
        perft.increment_size(depth);
        if board.in_check() {
            perft.increment_checks(depth);
        }
        if depth >= max_depth {
            return perft;
        }
        let moves = board.get_legal_moves();
        let (tx, rx) = unbounded();
        for mov in moves {
            match mov.get_capture() {
                Some(_) => perft.increment_captures(depth + 1),
                None => ()
            }
            match mov {
                Move::EnPassant(_) => perft.increment_en_passants(depth + 1),
                Move::Promotion(_) => perft.increment_promotions(depth + 1),
                Move::Castle(_) => perft.increment_castles(depth + 1),
                _ => (),
            }
            let local_pool = Arc::clone(&thread_pool);
            let mut thread_board = board;
            thread_board.make_move(&mov);
            thread_pool.enqueue(Job {
                task: Box::new(move || {
                    Self::threaded_perft(thread_board, depth + 1, max_depth, local_pool)
                }),
                comm: tx.clone()
            });
        }
        drop(tx);
        while let Ok(result) = rx.recv() {
            perft.merge(&result);
        }
        return perft;
    }

    fn perft(board: &mut Board, depth: u8, max_depth: u8) -> Perft {
        let mut perft: Perft = Default::default();
        perft.increment_size(depth);
        if depth >= max_depth {
            return perft;
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
            if board.in_check() { perft.increment_checks(depth + 1); }
            let result = Self::perft(board, depth + 1, max_depth);
            board.unmake_move(change);
            perft.merge(&result);
        }
        return perft
    }

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
                cache_hit = map.get(&ctx.board.id).map(|s| *s);
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
                        map.insert(ctx.board.id, calculated_score);
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
