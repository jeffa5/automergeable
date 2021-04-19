mod diff;
mod document;

/// Convenience re-exports for maintaining the same automerge versions
pub use automerge;
pub use automerge_protocol;

pub use automergeable_traits::{
    Automergeable, FromAutomerge, FromAutomergeError, Text, ToAutomerge,
};
pub use diff::diff_values;
pub use document::{Document, DocumentChangeError};

/// Derive macro magic
extern crate automergeable_derive;
pub use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};

/// needed for derive macro inner workings
#[doc(hidden)]
pub mod unicode_segmentation {
    pub use unicode_segmentation::UnicodeSegmentation;
}
