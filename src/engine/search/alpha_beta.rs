use std::{sync::{Mutex, Arc, RwLock}, cmp::Ordering, time::{Instant, Duration}};

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


pub struct AlphaBetaResult {
    color: Color,
    best_move: Move,
    best_move_score: i16,
    pub calculated_nodes: u32,
    pub cache_hits: u32,
    start_time: Option<Instant>,
    pub search_time: Duration,
}

impl AlphaBetaResult {
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


struct AlphaBetaThreadContext {
    move_color: Color,
    alpha: i16,
    beta: i16,
    beta_cutoff: bool,
    sibling_count: u8,
    completed: u8,
    completion_callback: Box<dyn Fn(i16) + Send + Sync>
}

impl AlphaBetaThreadContext {
    pub fn initial(board: Board, callback: Box<dyn Fn(i16) + Send + Sync>) -> Self {
        return Self {
            move_color: board.state.get_move_color(),
            alpha: best_score(board.state.get_move_color().swap()),
            beta: best_score(board.state.get_move_color()),
            beta_cutoff: false,
            sibling_count: 1,
            completed: 0,
            completion_callback: callback,
        }
    }

    pub fn next_context(&self, move_count: u8, callback: Box<dyn Fn(i16) + Send + Sync>) -> Result<Self, ()> {
        if self.beta_cutoff { return Err(()) };
        return Ok(Self {
            move_color: self.move_color.swap(),
            alpha: self.beta,
            beta: self.alpha,
            beta_cutoff: false,
            sibling_count: move_count,
            completed: 0,
            completion_callback: callback,
        });
    }

    pub fn is_beta_cutoff(&self) -> bool {
        return self.beta_cutoff;
    }

    pub fn current_alpha(&self) -> i16 {
        return self.alpha;
    }

    pub fn complete_sibling(&mut self, score: i16) {
        self.completed += 1;
        if self.beta_cutoff { return };
        if is_better(score, self.beta, self.move_color) {
            self.beta_cutoff = true;
            (self.completion_callback)(self.beta);
        }
        if is_better(score, self.alpha, self.move_color) {
            self.alpha = score;
        }
        if self.completed >= self.sibling_count {
            (self.completion_callback)(self.alpha)
        }
    }
}


struct AlphaBetaContext {
    board: Board,
    calculated_nodes: u32,
    cache_hits: u32,
}

impl AlphaBetaContext {
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
            if ctx.is_better(score, opponent_best_forcible) { return opponent_best_forcible; }
            best_forcible = ctx.get_best(score, best_forcible);
        }
        return best_forcible;
    }

    pub fn do_threaded_search(board: Board, max_depth: u8, threads: u8) -> i16 {
        let queue_builder = PriorityQueueBuilder::from_priorities(Vec::from([
            AlphaBetaSearchPriority::FirstMove,
            AlphaBetaSearchPriority::NextTwo,
            AlphaBetaSearchPriority::NextFour,
            AlphaBetaSearchPriority::Remainder,
        ]));
        let mut pool = AsyncPriorityThreadPool::from_builder(queue_builder);
        pool.init(threads);
        let transpositions: Arc<RwLock<ZobristHashMap<i16>>> = Arc::new(RwLock::new(Default::default()));
        let (tx, rx) = unbounded();
        let ctx = Arc::new(RwLock::new(AlphaBetaThreadContext::initial(board, Box::new(move |score| {
            tx.send(score).expect("Error sending final result for threaded Alpha Beta Search.");
        }))));
        Self::threaded_search(board, max_depth, pool.clone_writer(), transpositions, ctx);
        let result = rx.recv().expect("Error receiving final result from threaded Alpha Beta Search");
        pool.join();
        return result;
    }

    fn threaded_search(board: Board, depth: u8, pool: PriorityQueueWriter<AlphaBetaSearchPriority, AsyncTask>, transpositions: Arc<RwLock<ZobristHashMap<i16>>>, ctx: Arc<RwLock<AlphaBetaThreadContext>>) {
        let transposition_id = board.zobrist.get_id();
        if let Some(transposition_score) = transpositions.read().unwrap().get(&transposition_id) {
            ctx.write().unwrap().complete_sibling(*transposition_score);
            return;
        }
        if depth <= 0 {
            ctx.write().unwrap().complete_sibling(Evaluator::evaluate_board(&board));
            return;
        }
        let prev_ctx = Arc::clone(&ctx);
        let prev_transpositions = Arc::clone(&transpositions);
        let moves = board.get_legal_moves();
        let ctx_result = ctx.read().unwrap().next_context(moves.len() as u8, Box::new(move |score| {
            prev_transpositions.write().unwrap().insert(transposition_id, score);
            prev_ctx.write().unwrap().complete_sibling(score)
        }));
        if let Ok(prepared_ctx) = ctx_result {
            let wrapped_ctx = Arc::new(RwLock::new(prepared_ctx));
            for (index, mov) in moves.iter().enumerate() {
                let next_pool = pool.clone();
                let next_transpositions = Arc::clone(&transpositions);
                let next_ctx = Arc::clone(&wrapped_ctx);
                let mut next_board = board;
                next_board.make_move(mov);
                pool.enqueue(AsyncTask {
                    task: Box::new(move || {
                        Self::threaded_search(next_board, depth - 1, next_pool, next_transpositions, next_ctx);
                    })
                }, &AlphaBetaSearchPriority::from_index(index)).expect("Error enqueueing AsyncTask for threaded Alpha Beta Search");
            }
        }
    }
}
