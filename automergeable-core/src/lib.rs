mod diff;
mod document;
mod from;
mod to;

pub use diff::diff;
pub use document::Document;
pub use from::FromAutomerge;
pub use to::ToAutomerge;

/// Overall trait for requiring all automerge sub-traits
pub trait Automergeable: to::ToAutomerge + from::FromAutomerge + Clone + Default {}
