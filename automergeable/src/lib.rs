mod diff;
mod document;

pub use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};
pub use automergeable_traits::{Automergeable, FromAutomerge, ToAutomerge};
pub use diff::{diff, diff_values};
pub use document::Document;
