use crossbeam_channel::unbounded;
use num_format::{ToFormattedString, Locale};
use tabled::Tabled;

use crate::{rules::{board::Board, pieces::movement::{Move, NullMove}}, util::concurrency::{pools::ThreadPool, tasks::Task}};


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

    pub fn get_analysis(&self) -> Vec<PrintablePerft> {
        return self.levels.iter().rev().map(|l| PrintablePerft::from_level(l)).collect();
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


#[derive(Default)]
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


#[derive(Tabled)]
pub struct PrintablePerft {
    pub size: String,
    pub captures: String,
    pub en_passants: String,
    pub castles: String,
    pub promotions: String,
    pub checks: String,
}

impl PrintablePerft {
    pub fn from_level(level: &LevelPerft) -> Self {
        return Self {
            size: level.size.to_formatted_string(&Locale::en),
            captures: level.captures.to_formatted_string(&Locale::en),
            en_passants: level.en_passants.to_formatted_string(&Locale::en),
            castles: level.castles.to_formatted_string(&Locale::en),
            promotions: level.promotions.to_formatted_string(&Locale::en),
            checks: level.checks.to_formatted_string(&Locale::en),
        }
    }
}


pub struct PerftContext {
    pub board: Board,
    pub last_move: Move,
    pub depth: u8,
}

impl PerftContext {
    pub fn clone_from_move(&self, mov: &Move) -> Self {
        let mut new_board = self.board;
        new_board.make_move(mov);
        return Self {
            board: new_board,
            last_move: *mov,
            depth: self.depth - 1
        }
    }
}

pub struct PerftRunner {}

impl PerftRunner {
    pub fn do_threaded_perft(board: Board, depth: u8, threads: u8) -> Perft {
        let mut result: Perft = Default::default();
        let mut thread_pool = ThreadPool::new();
        thread_pool.init(threads);
        let (tx, rx) = unbounded();
        for mov in board.get_legal_moves() {
            let mut thread_board = board;
            thread_board.make_move(&mov);
            thread_pool.enqueue(Task {
                task: Box::new(move || {
                    Self::perft(PerftContext {
                        board: thread_board,
                        last_move: mov,
                        depth: depth - 1,
                    })
                }),
                comm: tx.clone()
            });
        }
        drop(tx);
        while let Ok(node_result) = rx.recv() {
            result.merge(&node_result);
        };
        return result;
    }


    pub fn do_perft(board: Board, depth: u8) -> Perft {
        return Self::perft(PerftContext{
            board: board,
            last_move: Move::NullMove(NullMove {}),
            depth: depth,
        });
    }


    fn perft(ctx: PerftContext) -> Perft {
        let mut result: Perft = Default::default();
        result.increment_size(ctx.depth);
        if ctx.board.in_check() {
            result.increment_checks(ctx.depth);
        }
        match ctx.last_move.get_capture() {
            Some(_) => result.increment_captures(ctx.depth),
            None => ()
        }
        match ctx.last_move {
            Move::EnPassant(_) => result.increment_en_passants(ctx.depth),
            Move::Promotion(_) => result.increment_promotions(ctx.depth),
            Move::Castle(_) => result.increment_castles(ctx.depth),
            _ => (),
        }
        if ctx.depth <= 0 {
            return result;
        }
        let moves = ctx.board.get_legal_moves();
        for next_move in moves {
            let next_ctx = ctx.clone_from_move(&next_move);
            result.merge(&Self::perft(next_ctx));
        }
        return result
    }
}