use std::{sync::{Arc, RwLock}, cmp::Ordering, iter::Rev};

use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::{engine::{evaluation::Evaluator, scores::{best_score, is_better}}, util::{zobrist::ZobristHashMap, concurrency::{pools::AsyncPriorityThreadPool, tasks::AsyncTask, queues::{PriorityQueueWriter, PriorityQueueBuilder}}}, rules::{pieces::movement::{Move, NullMove}, board::Board}};


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


struct OrderedMoveIterator {
    base_iter: Rev<std::vec::IntoIter<Move>>,
    hash_move: Option<Move>,
    initialized: bool,
}

impl OrderedMoveIterator {
    pub fn from_moves(mut moves: Vec<Move>, hash_move: Option<Move>) -> Self {
        moves.sort();
        return Self {
            base_iter: moves.into_iter().rev(),
            hash_move: hash_move,
            initialized: false,
        }
    }
}

impl Iterator for OrderedMoveIterator {
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
}

impl AlphaBetaResult {
    pub fn new(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Empty,
            score: score,
            mov: None,
            evaluated_nodes: 0,
            cache_hits: 0,
        }
    }

    pub fn evaluated(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Evaluated,
            score: score,
            mov: None,
            evaluated_nodes: 1,
            cache_hits: 0,
        }
    }

    pub fn transposed(result: &Self) -> Self {
        return Self {
            result_type: result.result_type,
            score: result.score,
            mov: result.mov,
            evaluated_nodes: 0,
            cache_hits: 1,
        }
    }
}


enum AlphaBetaThreadContextParent {
    Channel(Sender<AlphaBetaResult>),
    Instance(Arc<RwLock<AlphaBetaThreadContext>>),
}


struct AlphaBetaThreadContext {
    transpositions: Arc<RwLock<ZobristHashMap<AlphaBetaResult>>>,
    parent: AlphaBetaThreadContextParent,
    board: Board,
    mov: Move,
    depth_remaining: u8,
    result: AlphaBetaResult,
    beta: i16,
    complete: bool,
    child_count: u8,
    children_complete: u8,
}

impl AlphaBetaThreadContext {
    pub fn initial(board: Board, channel: Sender<AlphaBetaResult>, depth: u8) -> Self {
        return Self {
            transpositions: Arc::new(RwLock::new(Default::default())),
            parent: AlphaBetaThreadContextParent::Channel(channel),
            board: board,
            mov: Move::NullMove(NullMove {}),
            depth_remaining: depth,
            result: AlphaBetaResult::new(best_score(board.state.get_move_color().swap())),
            beta: best_score(board.state.get_move_color()),
            complete: false,
            child_count: 0,
            children_complete: 0,
        }
    }

    pub fn advance(mut self) -> Result<Vec<Self>, ()> {
        if self.is_complete() {
            return Err(())
        }
        let transposition = self.transpositions.read().unwrap().get(&self.board.zobrist.get_id()).map(|s| *s);
        if let Some(result) = transposition {
            self.transpose(result);
            return Err(())
        }
        if self.depth_remaining <= 0 {
            self.evaluate();
            return Err(())
        }
        let mut moves = self.board.get_legal_moves();
        moves.sort();
        let prev_ctx = Arc::new(RwLock::new(self));
        return Ok(moves.into_iter().map(|mov| {
            { prev_ctx.write().unwrap().child_count += 1; }
            let mut new_board = prev_ctx.read().unwrap().board;
            new_board.make_move(&mov);
            Self {
                transpositions: Arc::clone(&prev_ctx.read().unwrap().transpositions),
                parent: AlphaBetaThreadContextParent::Instance(Arc::clone(&prev_ctx)),
                board: new_board,
                mov: mov,
                depth_remaining: prev_ctx.read().unwrap().depth_remaining - 1,
                result: AlphaBetaResult::new(prev_ctx.read().unwrap().beta),
                beta: prev_ctx.read().unwrap().result.score,
                complete: false,
                child_count: 0,
                children_complete: 0,
            }
        }).collect())
    }

    pub fn is_complete(&self) -> bool {
        return self.complete || match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.read().unwrap().is_complete(),
            AlphaBetaThreadContextParent::Channel(_) => false,
        }
    }

    fn evaluate(&mut self) {
        self.result = AlphaBetaResult::evaluated(Evaluator::evaluate_board(&self.board));
        self.finish();
    }

    fn transpose(&mut self, result: AlphaBetaResult) {
        self.result = AlphaBetaResult::transposed(&result);
        self.finish();
    }

    pub fn finish(&mut self) {
        self.complete = true;
        if self.transpositions.read().unwrap().get(&self.board.zobrist.get_id()).is_none() {
            self.transpositions.write().unwrap().insert(self.board.zobrist.get_id(), self.result);
        }
        match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.write().unwrap().complete_child(self.result, self.mov),
            AlphaBetaThreadContextParent::Channel(s) => s.send(self.result).expect("Error sending final result for threaded Alpha Beta Search."),
        }
    }

    pub fn complete_child(&mut self, result: AlphaBetaResult, child_move: Move) {
        if self.complete { return };
        self.children_complete += 1;
        self.result.evaluated_nodes += result.evaluated_nodes;
        self.result.cache_hits += result.cache_hits;
        if is_better(result.score, self.beta, self.board.state.get_move_color()) {
            self.result.result_type = AlphaBetaResultType::BetaCutoff;
            self.result.score = self.beta;
            self.result.mov = Some(child_move);
            self.finish();
            return;
        }
        if is_better(result.score, self.result.score, self.board.state.get_move_color()) {
            self.result.score = result.score;
            self.result.mov = Some(child_move);
        }
        if self.children_complete >= self.child_count {
            self.result.result_type = AlphaBetaResultType::Calculated;
            self.finish();
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

        for m in OrderedMoveIterator::from_moves(board.get_legal_moves(), hash_move) {
            let change = board.make_move(&m);
            let child_result = Self::search(board,beta, result.score, depth - 1, transpositions);
            board.unmake_move(change);
            result.evaluated_nodes += child_result.evaluated_nodes;
            result.cache_hits += child_result.cache_hits;
            if is_better(child_result.score, beta, board.state.get_move_color()) {
                result.result_type = AlphaBetaResultType::BetaCutoff;
                result.score = beta;
                result.mov = Some(m);
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

    pub fn do_threaded_search(board: Board, max_depth: u8, threads: u8) -> AlphaBetaResult {
        let queue_builder = PriorityQueueBuilder::from_priorities(Vec::from([
            AlphaBetaSearchPriority::FirstMove,
            AlphaBetaSearchPriority::Remainder,
        ]));
        let mut pool = AsyncPriorityThreadPool::from_builder(queue_builder);
        pool.init(threads);
        let (tx, rx) = unbounded();
        let ctx = AlphaBetaThreadContext::initial(board, tx.clone(), max_depth);
        Self::threaded_search(pool.clone_writer(), ctx);
        let result = rx.recv().expect("Error receiving final result from threaded Alpha Beta Search");
        pool.join();
        return result;
    }

    fn threaded_search(pool: Arc<RwLock<PriorityQueueWriter<AlphaBetaSearchPriority, AsyncTask>>>, ctx: AlphaBetaThreadContext) {
        if let Ok(contexts) = ctx.advance() {
            for (index, next_ctx) in contexts.into_iter().enumerate() {
                let next_pool = Arc::clone(&pool);
                pool.read().unwrap().enqueue(AsyncTask {
                    task: Box::new(move || {
                        Self::threaded_search(next_pool, next_ctx);
                    })
                }, &AlphaBetaSearchPriority::from_index(index)).expect("Error enqueueing AsyncTask for threaded Alpha Beta Search");
            }
        }
    }
}
