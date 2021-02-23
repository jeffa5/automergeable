mod diff;
mod to;

pub use diff::diff;
pub use to::ToAutomerge;

/// Overall trait for requiring all automerge sub-traits
pub trait Automergeable: to::ToAutomerge {}
