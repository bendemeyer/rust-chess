use indexmap::IndexMap;
use rand::seq::IteratorRandom;

use crate::rules::{Color, board::Board, pieces::{Piece, movement::Move}};

pub mod evaluation;


pub struct Engine {
    root: EngineNode,
    depth: u8,
}

impl Engine {
    pub fn new(depth: u8) -> Engine {
        let mut engine = Engine {
            root: EngineNode::from_board(Board::from_starting_position()),
            depth: depth,
        };
        engine.root.init(depth);
        return engine;
    }

    pub fn from_position(board: Board, depth: u8) -> Engine {
        let mut engine = Engine {
            root: EngineNode::from_board(board),
            depth: depth,
        };
        engine.root.init(depth);
        return engine;
    }

    pub fn make_move(&mut self, new_move: &Move) {
        self.root = self.root.children.remove(new_move).unwrap();
        self.root.init(self.depth);
    }

    pub fn get_moves(&self) -> Vec<&Move> {
        return self.root.children.keys().collect()
    }

    pub fn suggest_moves(&self, n: u8) -> Vec<&Move> {
        let mut rng = rand::thread_rng();
        (0..n).fold(Vec::new(), |mut moves, _| {
            moves.push(self.root.children.keys().choose(&mut rng).unwrap());
            moves
        })
    }

    pub fn size(&self) -> u64 {
        return self.root.compute_size()
    }

    pub fn piece_at(&self, square: u8) -> Option<&Piece> {
        return self.root.board.piece_map.get(&square);
    }

    pub fn turn(&self) -> Color {
        return self.root.board.state.to_move;
    }
}


struct EngineNode {
    board: Board,
    children: IndexMap<Move, EngineNode>,
    initialized: bool,
}

impl EngineNode {
    pub fn from_board(board: Board) -> EngineNode {
        return EngineNode {
            board: board,
            children: Default::default(),
            initialized: false,
        }
    }

    fn compute_size(&self) -> u64 {
        if !self.initialized { return 1 }
        return self.children.values().fold(1, |sum, node| sum + node.compute_size())
    }

    fn init(&mut self, depth: u8) {
        if depth == 0 { return; }
        if !self.initialized {
            for m in self.board.get_legal_moves() {
                let new_board = self.board.new_board_from_move(&m);
                self.children.insert(m, EngineNode::from_board(new_board));
            }
            self.initialized = true;
        }
        for node in self.children.values_mut() {
            node.init(depth - 1)
        }
    }
}