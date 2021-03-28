mod diff;
mod document;

pub mod traits {
    pub use automergeable_traits::{
        Automergeable, FromAutomerge, FromAutomergeError, Text, ToAutomerge,
    };
}
pub use automerge;
pub use automerge_protocol;
pub use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};
pub use diff::diff_values;
pub use document::{Document, DocumentChangeError};
pub mod unicode_segmentation {
    pub use unicode_segmentation::UnicodeSegmentation;
}
