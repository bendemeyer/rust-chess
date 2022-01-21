use std::{sync::{Arc, atomic::{AtomicU8, AtomicI16, AtomicBool, Ordering as AtomicOrdering, AtomicU32}}, cmp::Ordering, iter::Rev, thread, time::Duration};

use crossbeam::{channel::{Sender, Receiver, unbounded}, atomic::AtomicCell};

use crate::{engine::{evaluation::Evaluator, scores::{best_score, is_better}}, util::{zobrist::{ZobristHashMap, ZobristLockfreeMap}, concurrency::{pools::AsyncPriorityThreadPool, tasks::AsyncTask, queues::{PriorityQueueWriter, PriorityQueueBuilder}}}, rules::{pieces::movement::{Move, NullMove}, board::Board, Color}};


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


struct MoveOrderIterator {
    base_iter: Rev<std::vec::IntoIter<Move>>,
    hash_move: Option<Move>,
    initialized: bool,
}

impl MoveOrderIterator {
    pub fn from_moves(mut moves: Vec<Move>, hash_move: Option<Move>) -> Self {
        moves.sort();
        return Self {
            base_iter: moves.into_iter().rev(),
            hash_move: hash_move,
            initialized: false,
        }
    }
}

impl Iterator for MoveOrderIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized {
            self.initialized = true;
            return match self.hash_move {
                Some(mov) => Some(mov),
                None => self.next(),
            }
        }
        return match self.base_iter.next() {
            Some(mov) => {
                if let Some(hash_move) = self.hash_move {
                    if hash_move == mov { return self.next() }
                }
                Some(mov)
            },
            None => None,
        }

    }
}


struct ThreadedMoveOrderIterator {
    base_iter: std::vec::IntoIter<AlphaBetaThreadContext>,
    first_move: Option<AlphaBetaThreadContext>,
}

impl ThreadedMoveOrderIterator {
    pub fn from_contexts(mut sorted_contexts: Vec<AlphaBetaThreadContext>) -> Self {
        let first = sorted_contexts.pop();
        return Self {
            base_iter: sorted_contexts.into_iter(),
            first_move: first,
        }
    }
}

impl Iterator for ThreadedMoveOrderIterator {
    type Item = (AlphaBetaSearchPriority, AlphaBetaThreadContext);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first_move.take() {
            return Some((AlphaBetaSearchPriority::FirstMove, first));
        }
        return match self.base_iter.next() {
            Some(ctx) => Some((AlphaBetaSearchPriority::Remainder, ctx)),
            None => None
        }
    }
}


#[derive(Clone, Copy)]
struct Transposition {
    result_type: AlphaBetaResultType,
    score: i16,
    mov: Option<Move>,
    depth: u8,
}

enum TranspositionMatch {
    FullMatch(AlphaBetaResult),
    BestMove(Move),
    None,
}

fn process_transposition(alpha: i16, beta: i16, depth: u8, move_color: Color, t: &Transposition) -> TranspositionMatch {
    if depth >= t.depth {
        if (t.result_type == AlphaBetaResultType::BetaCutoff && is_better(beta, t.score, move_color)) ||
            (t.result_type != AlphaBetaResultType::BetaCutoff && is_better(t.score, beta, move_color))
        {
            return TranspositionMatch::FullMatch(AlphaBetaResult {
                result_type: AlphaBetaResultType::BetaCutoff,
                score: beta,
                mov: t.mov,
                evaluated_nodes: 0,
                cache_hits: 1,
                beta_cutoffs: 1,
            });
        }
        if t.result_type != AlphaBetaResultType::BetaCutoff && is_better(alpha, t.score, move_color) {
            return TranspositionMatch::FullMatch(AlphaBetaResult {
                result_type: AlphaBetaResultType::AlphaFallback,
                score: alpha,
                mov: t.mov,
                evaluated_nodes: 0,
                cache_hits: 1,
                beta_cutoffs: 0,
            });
        }
        if t.result_type == AlphaBetaResultType::Calculated || t.result_type == AlphaBetaResultType::Evaluated {
            return TranspositionMatch::FullMatch(AlphaBetaResult {
                result_type: t.result_type,
                score: t.score,
                mov: t.mov,
                evaluated_nodes: 0,
                cache_hits: 1,
                beta_cutoffs: 0,
            });
        }
    }
    if let Some(m) = t.mov {
        return TranspositionMatch::BestMove(m);
    }
    return TranspositionMatch::None;
}


#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum AlphaBetaSearchPriority {
    FirstMove,
    Remainder,
}

impl AlphaBetaSearchPriority {
    pub fn from_index(index: usize) -> Self {
        return match index {
            0 => AlphaBetaSearchPriority::FirstMove,
            _ => AlphaBetaSearchPriority::Remainder,
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AlphaBetaResultType {
    Empty,
    Evaluated,
    Calculated,
    BetaCutoff,
    AlphaFallback,
}


#[derive(Copy, Clone)]
pub struct AlphaBetaResult {
    pub result_type: AlphaBetaResultType,
    pub score: i16,
    pub mov: Option<Move>,
    pub evaluated_nodes: u32,
    pub cache_hits: u32,
    pub beta_cutoffs: u32,
}

impl AlphaBetaResult {
    pub fn new(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Empty,
            score: score,
            mov: None,
            evaluated_nodes: 0,
            cache_hits: 0,
            beta_cutoffs: 0,
        }
    }

    pub fn evaluated(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Evaluated,
            score: score,
            mov: None,
            evaluated_nodes: 1,
            cache_hits: 0,
            beta_cutoffs: 0,
        }
    }

    pub fn transposed(result: &Self) -> Self {
        return Self {
            result_type: result.result_type,
            score: result.score,
            mov: result.mov,
            evaluated_nodes: 0,
            cache_hits: 1,
            beta_cutoffs: 0,
        }
    }
}


enum AlphaBetaThreadContextParent {
    Channel(Sender<AlphaBetaResult>),
    Instance(Arc<AlphaBetaThreadContext>),
}


struct AlphaBetaThreadContext {
    transpositions: Arc<ZobristLockfreeMap<Transposition>>,
    parent: AlphaBetaThreadContextParent,
    board: Board,
    mov: Move,
    depth_remaining: u8,
    alpha: AtomicI16,
    beta: i16,
    best_move: AtomicCell<Option<Move>>,
    evaluated: AtomicU32,
    transposed: AtomicU32,
    beta_cutoff: AtomicU32,
    complete: AtomicBool,
    child_count: u8,
    children_complete: AtomicU8,
}

impl AlphaBetaThreadContext {
    pub fn initial(board: Board, channel: Sender<AlphaBetaResult>, depth: u8) -> Self {
        return Self {
            transpositions: Arc::new(Default::default()),
            parent: AlphaBetaThreadContextParent::Channel(channel),
            board: board,
            mov: Move::NullMove(NullMove {}),
            depth_remaining: depth,
            alpha: AtomicI16::new(best_score(board.state.get_move_color().swap())),
            beta: best_score(board.state.get_move_color()),
            best_move: AtomicCell::new(None),
            evaluated: AtomicU32::new(0),
            transposed: AtomicU32::new(0),
            beta_cutoff: AtomicU32::new(0),
            complete: AtomicBool::new(false),
            child_count: 0,
            children_complete: AtomicU8::new(0),
        }
    }

    pub fn advance(mut self) -> Result<Vec<Self>, ()> {
        if self.is_complete() {
            return Err(())
        }
        let mut hash_move: Option<Move> = None;
        {
            let transposition = self.transpositions.get(&self.board.zobrist.get_id());
            if let Some(guard) = transposition {
                match process_transposition(self.alpha.load(AtomicOrdering::Acquire), self.beta, self.depth_remaining, self.board.state.get_move_color(), guard.val()) {
                    TranspositionMatch::FullMatch(r) => { self.transpose(r); return Err(()); },
                    TranspositionMatch::BestMove(m) => hash_move = Some(m),
                    TranspositionMatch::None => (),
                }
            }
        }
        if self.depth_remaining <= 0 {
            self.evaluate();
            return Err(())
        }
        let moves = self.board.get_legal_moves();
        if moves.len() == 0 {
            self.evaluate();
            return Err(())
        }
        self.child_count = moves.len() as u8;
        let prev_ctx = Arc::new(self);
        let result = Ok(MoveOrderIterator::from_moves(moves, hash_move).map(|mov| {
            let mut new_board = prev_ctx.board;
            new_board.make_move(&mov);
            Self {
                transpositions: Arc::clone(&prev_ctx.transpositions),
                parent: AlphaBetaThreadContextParent::Instance(Arc::clone(&prev_ctx)),
                board: new_board,
                mov: mov,
                depth_remaining: prev_ctx.depth_remaining - 1,
                alpha: AtomicI16::new(prev_ctx.beta),
                beta: prev_ctx.alpha.load(AtomicOrdering::Acquire),
                best_move: AtomicCell::new(None),
                evaluated: AtomicU32::new(0),
                transposed: AtomicU32::new(0),
                beta_cutoff: AtomicU32::new(0),
                complete: AtomicBool::new(false),
                child_count: 0,
                children_complete: AtomicU8::new(0),
            }
        }).collect());
        return result;
    }

    pub fn is_complete(&self) -> bool {
        return self.complete.load(AtomicOrdering::Acquire) || match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.is_complete(),
            AlphaBetaThreadContextParent::Channel(_) => false,
        }
    }

    fn evaluate(&self) {
        self.finish(AlphaBetaResult::evaluated(Evaluator::evaluate_board(&self.board)));
    }

    fn transpose(&self, result: AlphaBetaResult) {
        self.finish(result);
    }

    pub fn finish(&self, result: AlphaBetaResult) {
        self.complete.store(true, AtomicOrdering::Release);
        self.transpositions.insert(self.board.zobrist.get_id(), Transposition {
            result_type: result.result_type,
            score: result.score,
            mov: result.mov,
            depth: self.depth_remaining,
        });
        match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.complete_child(result, self.mov),
            AlphaBetaThreadContextParent::Channel(s) => s.send(result).expect("Error sending final result for threaded Alpha Beta Search."),
        }
    }

    pub fn complete_child(&self, result: AlphaBetaResult, child_move: Move) {
        self.children_complete.fetch_add(1, std::sync::atomic::Ordering::Release);
        if self.is_complete() {
            return
        };
        self.evaluated.fetch_add(result.evaluated_nodes, AtomicOrdering::Release);
        self.transposed.fetch_add(result.cache_hits, AtomicOrdering::Release);
        self.beta_cutoff.fetch_add(result.beta_cutoffs, AtomicOrdering::Release);
        if is_better(result.score, self.beta, self.board.state.get_move_color()) {
            self.finish(AlphaBetaResult {
                result_type: AlphaBetaResultType::BetaCutoff,
                score: self.beta,
                mov: Some(child_move),
                evaluated_nodes: self.evaluated.load(AtomicOrdering::Acquire),
                cache_hits: self.transposed.load(AtomicOrdering::Acquire),
                beta_cutoffs: self.beta_cutoff.load(AtomicOrdering::Acquire) + 1,
            });
            return;
        }
        if is_better(result.score, self.alpha.load(AtomicOrdering::Acquire), self.board.state.get_move_color()) {
            self.alpha.store(result.score, AtomicOrdering::Release);
            self.best_move.store(Some(child_move));
        }
        if self.children_complete.load(std::sync::atomic::Ordering::Acquire) >= self.child_count {
            self.finish(AlphaBetaResult {
                result_type: AlphaBetaResultType::Calculated,
                score: self.alpha.load(AtomicOrdering::Acquire),
                mov: self.best_move.take(),
                evaluated_nodes: self.evaluated.load(AtomicOrdering::Acquire),
                cache_hits: self.transposed.load(AtomicOrdering::Acquire),
                beta_cutoffs: self.beta_cutoff.load(AtomicOrdering::Acquire),
            });
        }
    }
}


struct AlphaBetaContext {
    board: Board,
    evaluated_nodes: u32,
    cache_hits: u32,
}


pub struct AlphaBetaSearch {
    close_comms: Vec<Sender<bool>>,
    result_comm: Receiver<AlphaBetaResult>,
}

impl AlphaBetaSearch {

    pub fn do_search(mut board: Board, depth: u8) -> AlphaBetaResult {
        let mut transpositions: ZobristHashMap<AlphaBetaResult> = Default::default();
        let alpha = best_score(board.state.get_move_color().swap());
        let beta = best_score(board.state.get_move_color());
        return Self::search(&mut board, alpha, beta, depth, &mut transpositions);
    }

    fn search(board: &mut Board, alpha: i16, beta: i16, depth: u8, transpositions: &mut ZobristHashMap<AlphaBetaResult>) -> AlphaBetaResult {
        let mut result = AlphaBetaResult::new(alpha);
        let mut hash_move: Option<Move> = None;
        if let Some(transposed_result) = transpositions.get(&board.zobrist.get_id()) {
            result.cache_hits += 1;
            if (transposed_result.result_type == AlphaBetaResultType::BetaCutoff && is_better(beta, transposed_result.score, board.state.get_move_color())) ||
               (transposed_result.result_type != AlphaBetaResultType::BetaCutoff && is_better(transposed_result.score, beta, board.state.get_move_color()))
            {
                result.result_type = AlphaBetaResultType::BetaCutoff;
                result.score = beta;
                result.mov = transposed_result.mov;
                return result;
            }
            if transposed_result.result_type != AlphaBetaResultType::BetaCutoff && is_better(alpha, transposed_result.score, board.state.get_move_color()) {
                result.result_type = AlphaBetaResultType::AlphaFallback;
                result.score = alpha;
                result.mov = transposed_result.mov;
                return result;
            }
            if transposed_result.result_type == AlphaBetaResultType::Calculated || transposed_result.result_type == AlphaBetaResultType::Evaluated {
                return AlphaBetaResult::transposed(transposed_result);
            }
            hash_move = transposed_result.mov;
        }

        if depth <= 0 {
            let evaluation = AlphaBetaResult::evaluated(Evaluator::evaluate_board(&board));
            transpositions.insert(board.zobrist.get_id(), evaluation);
            return evaluation;
        }

        for m in MoveOrderIterator::from_moves(board.get_legal_moves(), hash_move) {
            let change = board.make_move(&m);
            let child_result = Self::search(board,beta, result.score, depth - 1, transpositions);
            board.unmake_move(change);
            result.evaluated_nodes += child_result.evaluated_nodes;
            result.cache_hits += child_result.cache_hits;
            result.beta_cutoffs += child_result.beta_cutoffs;
            if is_better(child_result.score, beta, board.state.get_move_color()) {
                result.result_type = AlphaBetaResultType::BetaCutoff;
                result.score = beta;
                result.mov = Some(m);
                result.beta_cutoffs += 1;
                break;
            }
            if is_better(child_result.score, result.score, board.state.get_move_color()) {
                result.score = child_result.score;
                result.mov = Some(m);
            }
        }
        if result.result_type == AlphaBetaResultType::Empty && is_better(result.score, alpha, board.state.get_move_color()) {
            result.result_type = AlphaBetaResultType::Calculated;
        } else {
            result.result_type = AlphaBetaResultType::AlphaFallback;
        }
        transpositions.insert(board.zobrist.get_id(), result);
        return result;
    }

    pub fn do_threaded_search(board: Board, max_depth: u8, threads: u8, initial_sleep: u64) -> AlphaBetaResult {
        let queue_builder = PriorityQueueBuilder::from_priorities(Vec::from([
            AlphaBetaSearchPriority::FirstMove,
            AlphaBetaSearchPriority::Remainder,
        ]));
        let mut pool = AsyncPriorityThreadPool::from_builder(queue_builder);
        pool.start_workers(1);
        let (tx, rx) = unbounded();
        let ctx = AlphaBetaThreadContext::initial(board, tx, max_depth);
        Self::threaded_search(pool.clone_writer(), ctx);
        thread::sleep(Duration::from_millis(initial_sleep));
        pool.start_workers(threads - 1);
        let result = rx.recv().expect("Error receiving result of threaded Alpha Beta search.");
        pool.join();
        return result;
    }

    fn threaded_search(pool: PriorityQueueWriter<AlphaBetaSearchPriority, AsyncTask>, ctx: AlphaBetaThreadContext) {
        if let Ok(contexts) = ctx.advance() {
            for (priority, next_ctx) in ThreadedMoveOrderIterator::from_contexts(contexts) {
                let next_pool = pool.clone();
                pool.enqueue(AsyncTask {
                    task: Box::new(move || {
                        Self::threaded_search(next_pool, next_ctx);
                    })
                }, &priority).expect("Error enqueueing AsyncTask for threaded Alpha Beta Search");
            }
        }
    }
}
