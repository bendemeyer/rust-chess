pub mod errors;
pub mod fen;
pub mod zobrist;

pub use std::ops::ControlFlow;

use fxhash::FxBuildHasher;
use indexmap::{IndexMap, IndexSet};

pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
pub type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;


pub trait UnwrapsAll<T> {
    fn unwrap_all(self) -> T;
}

impl<T> UnwrapsAll<T> for ControlFlow<T, T> {
    fn unwrap_all(self) -> T {
        return match self {
            ControlFlow::Continue(t) => t,
            ControlFlow::Break(t) => t,
        }
    }
}

pub struct FoldHelper<T, D> {
    pub accumulator: T,
    pub data: D,
}

impl<T, D> FoldHelper<T, D> {
    pub fn init(acc: T, data: D) -> FoldHelper<T, D> {
        return FoldHelper { accumulator: acc, data: data };
    }

    pub fn get_result(self) -> T {
        return self.accumulator;
    }
}