#![warn(missing_docs)]
#![warn(missing_crate_level_docs)]
#![warn(missing_doc_code_examples)]

//! Strongly typed automerge documents.
//!
//! This crate provides functionality for building and working with strongly typed automerge documents.

//! Typically an automerge document consists of the [`automerge::Value`] enum and manipulating this
//! with the default change function can be less idiomatic and we can't use arbitrary Rust structs!
//! This library provides functionality for converting to and from [`automerge::Value`]s as well as
//! using these with a change function to use arbitrary Rust structs and enumsin the document
//! seamlessly.

//! # A Document
//!
//! A document provides the typed wrapper to an [`automerge::Frontend`].
//!
//! ```rust
//! # use std::collections::HashMap;
//! # use automergeable::Document;
//! # use maplit::hashmap;
//! # fn main() -> Result<(), automergeable::DocumentChangeError> {
//! let mut doc = Document::<HashMap<String, String>>::new();
//! let ((), change) = doc.change(|map| {
//!     map.insert("my_value".to_owned(), "Hey, a nice Rust Map!".to_owned());
//!     Ok(())
//! })?;
//! // apply the change to a backend
//! assert_eq!(
//!     doc.get().unwrap().unwrap(),
//!     hashmap! {"my_value".to_owned() => "Hey, a nice Rust Map!".to_owned()}
//! );
//! # Ok(())
//! # }
//! ```

//! # Using structs and enums
//!
//! To use structs and enums in our documents we need to be able to convert them to and from
//! [`automerge::Value`]s. This is characterised by the [`ToAutomerge`] and [`FromAutomerge`]
//! traits. These are implemented for most of the `std` types already (if any are missing that you
//! want to use, please raise an issue or PR!).
//!
//! ## Deriving Automergeable
//!
//! ```rust
//! # use automergeable::Automergeable;
//! #[derive(Clone, Default, Automergeable)]
//! struct A {
//!     b: u32,
//!     c: String,
//!     #[automergeable(representation = "text")]
//!     d: String,
//!     #[automergeable(representation = "counter")]
//!     e: i64,
//!     #[automergeable(representation = "timestamp")]
//!     f: i64,
//! }
//! ```
//!
//! This can then be used in a document as such:
//!
//! ```rust
//! # use automergeable::Automergeable;
//! # #[derive(Clone, Default, Debug, PartialEq, Automergeable)]
//! # struct A {
//! #   b: u32,
//! #   c: String,
//! #   #[automergeable(representation = "text")]
//! #   d: String,
//! #   #[automergeable(representation = "counter")]
//! #   e: i64,
//! #   #[automergeable(representation = "timestamp")]
//! #   f: i64,
//! # }
//! #
//! # use std::collections::HashMap;
//! # use automergeable::Document;
//! # fn main() -> Result<(), automergeable::DocumentChangeError> {
//! let mut doc = Document::<A>::new();
//! let ((), change) = doc.change(|a| {
//!     a.b += 1;
//!     a.d = "Hello world!".to_owned();
//!     Ok(())
//! })?;
//! // apply the change to a backend
//! assert_eq!(
//!     doc.get().unwrap().unwrap(),
//!     A {
//!         b: 1,
//!         c: String::new(),
//!         d: "Hello world!".to_owned(),
//!         e: 0,
//!         f: 0
//!     }
//! );
//! # Ok(())
//! # }
//! ```

mod diff;
mod document;

/// Convenience re-exports for maintaining the same automerge versions
#[doc(hidden)]
pub use automerge;
#[doc(hidden)]
pub use automerge_protocol;
#[doc(hidden)]
pub use automergeable_traits::Text;
pub use automergeable_traits::{Automergeable, FromAutomerge, FromAutomergeError, ToAutomerge};
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
