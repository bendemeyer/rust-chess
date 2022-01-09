pub mod concurrency;
pub mod errors;
pub mod fen;
pub mod zobrist;

pub use std::ops::ControlFlow;


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
