mod from;
mod to;

pub use from::{FromAutomerge, FromAutomergeError};
pub use to::ToAutomerge;

/// Overall trait for requiring all automerge sub-traits.
pub trait Automergeable: to::ToAutomerge + from::FromAutomerge {}
