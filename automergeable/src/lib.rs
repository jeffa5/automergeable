mod diff;
mod document;

pub mod traits {
    pub use automergeable_traits::{Automergeable, FromAutomerge, FromAutomergeError, ToAutomerge};
}
pub use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};
pub use diff::diff_values;
pub use document::Document;
