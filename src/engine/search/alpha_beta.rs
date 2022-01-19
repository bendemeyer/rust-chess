use std::{sync::{Mutex, Arc, RwLock}, cmp::Ordering};

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


#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum AlphaBetaSearchPriority {
    FirstMove,
    NextTwo,
    NextFour,
    Remainder,
}

impl AlphaBetaSearchPriority {
    pub fn from_index(index: usize) -> Self {
        return match index {
            0 => AlphaBetaSearchPriority::FirstMove,
            1..=2 => AlphaBetaSearchPriority::NextTwo,
            3..=6 => AlphaBetaSearchPriority::NextFour,
            _ => AlphaBetaSearchPriority::Remainder,
        }
    }
}


#[derive(Copy, Clone)]
pub enum AlphaBetaResultType {
    Empty,
    Cutoff,
    Transposition,
    Calculated,
}


#[derive(Copy, Clone)]
pub struct AlphaBetaResult {
    pub result_type: AlphaBetaResultType,
    pub score: i16,
    pub mov: Move,
    pub calculated_nodes: u32,
    pub cache_hits: u32,
}

impl AlphaBetaResult {
    pub fn new(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Empty,
            score: score,
            mov: Move::NullMove(NullMove {}),
            calculated_nodes: 0,
            cache_hits: 0,
        }
    }

    pub fn evaluated(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Calculated,
            score: score,
            mov: Move::NullMove(NullMove {}),
            calculated_nodes: 1,
            cache_hits: 0,
        }
    }

    pub fn transposed(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Transposition,
            score: score,
            mov: Move::NullMove(NullMove {}),
            calculated_nodes: 0,
            cache_hits: 1
        }
    }
}


enum AlphaBetaThreadContextParent {
    Channel(Sender<AlphaBetaResult>),
    Instance(Arc<RwLock<AlphaBetaThreadContext>>),
}


struct AlphaBetaThreadContext {
    transpositions: Arc<RwLock<ZobristHashMap<i16>>>,
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
        if self.depth_remaining <= 0 {
            self.evaluate();
            return Err(())
        }
        let transposition = self.transpositions.read().unwrap().get(&self.board.zobrist.get_id()).map(|s| *s);
        if let Some(score) = transposition {
            self.transpose(score);
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

    fn transpose(&mut self, score: i16) {
        self.result = AlphaBetaResult::transposed(score);
        self.finish();
    }

    pub fn finish(&mut self) {
        self.complete = true;
        if self.transpositions.read().unwrap().get(&self.board.zobrist.get_id()).is_none() {
            self.transpositions.write().unwrap().insert(self.board.zobrist.get_id(), self.result.score);
        }
        match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.write().unwrap().complete_child(self.result, self.mov),
            AlphaBetaThreadContextParent::Channel(s) => s.send(self.result).expect("Error sending final result for threaded Alpha Beta Search."),
        }
    }

    pub fn complete_child(&mut self, result: AlphaBetaResult, child_move: Move) {
        if self.complete { return };
        self.children_complete += 1;
        if is_better(result.score, self.beta, self.board.state.get_move_color()) {
            self.result.result_type = AlphaBetaResultType::Cutoff;
            self.result.score = self.beta;
            self.result.mov = child_move;
            self.finish();
            return;
        }
        if is_better(result.score, self.result.score, self.board.state.get_move_color()) {
            self.result.score = result.score;
            self.result.mov = child_move;
        }
        if self.children_complete >= self.child_count {
            self.result.result_type = AlphaBetaResultType::Calculated;
            self.finish();
        }
    }
}


struct AlphaBetaContext {
    board: Board,
    calculated_nodes: u32,
    cache_hits: u32,
}


pub struct AlphaBetaSearch {
    close_comms: Vec<Sender<bool>>,
    result_comm: Receiver<AlphaBetaResult>,
}

impl AlphaBetaSearch {

    fn search(mut best_forcible: i16, opponent_best_forcible: i16, depth: u8, transpositions: &Mutex<ZobristHashMap<i16>>, ctx: &mut AlphaBetaContext) -> i16 {
        if depth <= 0 {
            ctx.calculated_nodes += 1;
            return Evaluator::evaluate_board(&ctx.board)
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
            if is_better(score, opponent_best_forcible, ctx.board.state.get_move_color()) { return opponent_best_forcible; }
            best_forcible = if is_better(score, best_forcible, ctx.board.state.get_move_color()) { score } else { best_forcible };
        }
        return best_forcible;
    }

    pub fn do_threaded_search(board: Board, max_depth: u8, threads: u8) -> AlphaBetaResult {
        let queue_builder = PriorityQueueBuilder::from_priorities(Vec::from([
            AlphaBetaSearchPriority::FirstMove,
            AlphaBetaSearchPriority::NextTwo,
            AlphaBetaSearchPriority::NextFour,
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

    fn threaded_search(pool: PriorityQueueWriter<AlphaBetaSearchPriority, AsyncTask>, ctx: AlphaBetaThreadContext) {
        if let Ok(contexts) = ctx.advance() {
            for (index, next_ctx) in contexts.into_iter().enumerate() {
                let next_pool = pool.clone();
                pool.enqueue(AsyncTask {
                    task: Box::new(move || {
                        Self::threaded_search(next_pool, next_ctx);
                    })
                }, &AlphaBetaSearchPriority::from_index(index)).expect("Error enqueueing AsyncTask for threaded Alpha Beta Search");
            }
        }
    }
}
