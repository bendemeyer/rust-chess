use std::{time::{Duration, Instant}, mem};

use fxhash::FxHashSet;

use crate::{util::zobrist::ZobristHashMap, rules::{board::Board, pieces::movement::{Move, NullMove}}};


#[derive(Clone)]
pub struct Collision {
    pub cause: Move,
    pub fen_1: String,
    pub fen_2: String,
    pub hash: u64,
}


#[derive(Default)]
pub struct ZobristCollisionTestContext {
    positions_checked: u32,
    hash_matches_detected: u32,
    collisions_detected: u32,
    start: Option<Instant>,
    memory_used: u64,
    collision_hashes: FxHashSet<u64>,
    all_boards: ZobristHashMap<FxHashSet<String>>,
    collisions: Vec<Collision>,
}

impl ZobristCollisionTestContext {
    pub fn init(&mut self) {
        self.start = Some(Instant::now())
    }

    pub fn has_collision(&self) -> bool {
        return self.collisions_detected > 0;
    }

    pub fn process(&mut self, board: Board, mov: Move) {
        self.positions_checked += 1;
        let mut fen = board.to_fen();
        fen = String::from(&fen[..fen.len() - 4]);
        let mem_size = fen.len();
        match self.all_boards.get_mut(&board.zobrist.get_id()) {
            Some(fens) => {
                self.hash_matches_detected += 1;
                if !fens.contains(&fen) || fens.len() > 1 {
                    self.collisions.push(Collision {
                        cause: mov,
                        fen_1: fen.clone(),
                        fen_2: fens.iter().next().unwrap().clone(),
                        hash: board.zobrist.get_id(),
                    });
                    self.collisions_detected += 1;
                    if self.collision_hashes.insert(board.zobrist.get_id()){
                        self.memory_used += mem::size_of::<u64>() as u64;
                    };
                }
                if fens.insert(fen) {
                    self.memory_used += mem_size as u64;
                }
            },
            None => {
                self.all_boards.insert(board.zobrist.get_id(), FxHashSet::from_iter([fen].into_iter()));
                self.memory_used += mem::size_of::<u64>() as u64;
                self.memory_used += mem_size as u64;
            },
        }
    }

    pub fn complete(&self) -> ZobristCollisionTestResult {
        return ZobristCollisionTestResult {
            positions_checked: self.positions_checked,
            hash_matches: self.hash_matches_detected,
            transpositions: self.hash_matches_detected - self.collisions_detected,
            collisions: self.collisions_detected,
            collided_hash_count: self.collision_hashes.len() as u32,
            duration: self.start.expect("Zobrist Collision Test Context completed before it was initialized!").elapsed(),
            memory_size: self.memory_used,
            collision_pairs: self.collisions.clone(),
        }
    }
}


pub struct ZobristCollisionTestResult {
    pub positions_checked: u32,
    pub hash_matches: u32,
    pub transpositions: u32,
    pub collisions: u32,
    pub collided_hash_count: u32,
    pub duration: Duration,
    pub memory_size: u64,
    pub collision_pairs: Vec<Collision>,
}


pub struct ZobristCollisionTester {}

impl ZobristCollisionTester {
    pub fn do_test(board: Board, depth: u8) -> ZobristCollisionTestResult {
        let mut ctx: ZobristCollisionTestContext = Default::default();
        ctx.init();
        Self::collision_test(board, Move::NullMove(NullMove {}), depth, &mut ctx);
        return ctx.complete();
    }

    pub fn collision_test(board: Board, last_move: Move, depth: u8, ctx: &mut ZobristCollisionTestContext) {
        ctx.process(board, last_move);
        if ctx.has_collision() { return }
        if depth <= 0 { return }
        for mov in board.get_legal_moves() {
            let mut new_board = board;
            new_board.make_move(&mov);
            Self::collision_test(new_board, mov, depth - 1, ctx)
        }
    }
}