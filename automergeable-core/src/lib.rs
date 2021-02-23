mod to;
mod diff;

pub use to::ToAutomerge;
pub use diff::diff;
pub trait Automergeable : to::ToAutomerge{}
