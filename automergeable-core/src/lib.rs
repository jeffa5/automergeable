mod diff;
mod to;

pub use diff::diff;
pub use to::ToAutomerge;
pub trait Automergeable: to::ToAutomerge {}
