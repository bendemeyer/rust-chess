use std::{sync::{Mutex, Arc, RwLock}, cmp::Ordering};

use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::{engine::{evaluation::Evaluator, scores::{best_score, is_better}}, util::{zobrist::ZobristHashMap, concurrency::{pools::AsyncPriorityThreadPool, tasks::AsyncTask, queues::{PriorityQueueWriter, PriorityQueueBuilder}}}, rules::{pieces::movement::{Move, NullMove}, Color, board::Board}};


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
    Calculated(Move),
}


#[derive(Copy, Clone)]
pub struct AlphaBetaResult {
    pub result_type: AlphaBetaResultType,
    pub score: i16,
    pub calculated_nodes: u32,
    pub cache_hits: u32,
}

impl AlphaBetaResult {
    pub fn new(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Empty,
            score: score,
            calculated_nodes: 0,
            cache_hits: 0,
        }
    }

    pub fn evaluated(score: i16, mov: Move) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Calculated(mov),
            score: score,
            calculated_nodes: 1,
            cache_hits: 0,
        }
    }

    pub fn transposed(score: i16) -> Self {
        return Self {
            result_type: AlphaBetaResultType::Transposition,
            score: score,
            calculated_nodes: 0,
            cache_hits: 1,
        }
    }

    pub fn merge(&mut self, other: Self, color: Color) {
        self.calculated_nodes += other.calculated_nodes;
        self.cache_hits += other.cache_hits;
        if is_better(other.score, self.score, color) {
            self.result_type = other.result_type;
            self.score = other.score;
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
    move_color: Color,
    result: AlphaBetaResult,
    beta: i16,
    complete: bool,
    child_count: u8,
    children_complete: u8,
}

impl AlphaBetaThreadContext {
    pub fn initial(board: Board, channel: Sender<AlphaBetaResult>) -> Self {
        return Self {
            transpositions: Arc::new(RwLock::new(Default::default())),
            parent: AlphaBetaThreadContextParent::Channel(channel),
            move_color: board.state.get_move_color(),
            result: AlphaBetaResult::new(best_score(board.state.get_move_color().swap())),
            beta: best_score(board.state.get_move_color()),
            complete: false,
            child_count: 1,
            children_complete: 0,
        }
    }

    pub fn next_context(prev: &Arc<RwLock<AlphaBetaThreadContext>>, move_count: u8) -> Result<Self, ()> {
        let prev_ref = prev.read().unwrap();
        if prev_ref.is_complete() {
            return Err(());
        }
        return Ok(Self {
            transpositions: Arc::clone(&prev_ref.transpositions),
            parent: AlphaBetaThreadContextParent::Instance(Arc::clone(prev)),
            move_color: prev_ref.move_color.swap(),
            result: AlphaBetaResult::new(prev_ref.beta),
            beta: prev_ref.result.score,
            complete: false,
            child_count: move_count,
            children_complete: 0,
        });
    }

    pub fn is_complete(&self) -> bool {
        return self.complete || match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.read().unwrap().is_complete(),
            AlphaBetaThreadContextParent::Channel(_) => true,
        }
    }

    pub fn check_transpositions(&self, id: u64) -> Option<i16> {
        return self.transpositions.read().unwrap().get(&id).map(|s| *s);
    }

    pub fn finish(&mut self) {
        self.complete = true;
        if let AlphaBetaResultType::Calculated(_mov) = self.result.result_type {
            // TODO: Write score to transposition table
        }
        match &self.parent {
            AlphaBetaThreadContextParent::Instance(p) => p.write().unwrap().complete_sibling(self.result),
            AlphaBetaThreadContextParent::Channel(s) => s.send(self.result).expect("Error sending final result for threaded Alpha Beta Search."),
        }
    }

    pub fn complete_sibling(&mut self, result: AlphaBetaResult) {
        self.children_complete += 1;
        if self.complete { return };
        if is_better(result.score, self.beta, self.move_color) {
            self.result.result_type = AlphaBetaResultType::Cutoff;
            self.finish();
        }
        self.result.merge(result, self.move_color);
        if self.children_complete >= self.child_count {
            self.finish()
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
        let ctx = Arc::new(RwLock::new(AlphaBetaThreadContext::initial(board, tx.clone())));
        Self::threaded_search(board, Move::NullMove(NullMove {}), max_depth, pool.clone_writer(), ctx);
        let result = rx.recv().expect("Error receiving final result from threaded Alpha Beta Search");
        pool.join();
        return result;
    }

    fn threaded_search(board: Board, mov: Move, depth: u8, pool: PriorityQueueWriter<AlphaBetaSearchPriority, AsyncTask>, ctx: Arc<RwLock<AlphaBetaThreadContext>>) {
        if ctx.read().unwrap().is_complete() {
            return;
        }
        if depth <= 0 {
            ctx.write().unwrap().complete_sibling(AlphaBetaResult::evaluated(Evaluator::evaluate_board(&board), mov));
            return;
        }
        if let Some(score) = ctx.read().unwrap().check_transpositions(board.zobrist.get_id()) {
            ctx.write().unwrap().complete_sibling(AlphaBetaResult::transposed(score));
            return;
        }
        let moves = board.get_legal_moves();
        if let Ok(prepared_ctx) = AlphaBetaThreadContext::next_context(&ctx, moves.len() as u8) {
            let wrapped_ctx = Arc::new(RwLock::new(prepared_ctx));
            for (index, mov) in moves.into_iter().enumerate() {
                let next_pool = pool.clone();
                let next_ctx = Arc::clone(&wrapped_ctx);
                let mut next_board = board;
                next_board.make_move(&mov);
                pool.enqueue(AsyncTask {
                    task: Box::new(move || {
                        Self::threaded_search(next_board, mov, depth - 1, next_pool, next_ctx);
                    })
                }, &AlphaBetaSearchPriority::from_index(index)).expect("Error enqueueing AsyncTask for threaded Alpha Beta Search");
            }
        }
    }
}
