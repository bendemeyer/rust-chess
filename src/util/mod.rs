pub mod errors;
pub mod fen;

use fnv::FnvBuildHasher;
use indexmap::IndexSet;

pub type FnvIndexSet<T> = IndexSet<T, FnvBuildHasher>;