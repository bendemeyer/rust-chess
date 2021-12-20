pub mod errors;
pub mod fen;

pub use std::ops::ControlFlow;

use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};

pub type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;
pub type FnvIndexSet<T> = IndexSet<T, FnvBuildHasher>;


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