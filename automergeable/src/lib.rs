mod diff;
mod document;

pub use automergeable_derive::{Automergeable, ToAutomerge};
pub use automergeable_traits::{Automergeable, FromAutomerge, ToAutomerge};
pub use diff::diff;
pub use document::Document;
