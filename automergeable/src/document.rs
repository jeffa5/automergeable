use std::fmt::{Debug, Display};

use automerge::{Frontend, Path, Value};
use automerge_protocol::Patch;

use crate::Automergeable;

/// An error type for change operations on documents.
#[derive(Debug, thiserror::Error)]
pub enum DocumentChangeError<E: Debug + Display = std::convert::Infallible> {
    /// An invalid change request was created.
    ///
    /// Automerge imposes some limits on what can be changed and how. See the
    /// [`automerge::InvalidChangeRequest`] documentation for more details.
    #[error(transparent)]
    InvalidChangeRequest(#[from] automerge::InvalidChangeRequest),
    /// A failure to convert the value in automerge to a typed value.
    #[error(transparent)]
    FromError(#[from] crate::FromAutomergeError),
    /// A custom error from the users closure.
    #[error(
        "change error:
        {0}"
    )]
    ChangeError(E),
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyPatchError {
    #[error(transparent)]
    InvalidPatch(#[from] automerge_frontend::InvalidPatch),
    #[error(transparent)]
    FromError(#[from] crate::FromAutomergeError),
}

/// A typed automerge document, wrapping a typical frontend.
///
/// This provides similar functionality to an automerge frontend (including [`Deref`] to one) but with
/// stronger typed data.
///
/// For instance from a document we can get the value as a typical Rust struct and perform
/// automerge change operations on it with automatic diffing behind the scenes.
#[derive(Debug)]
pub struct Document<T>
where
    T: Automergeable,
{
    frontend: Frontend,
    value: T,
    original: Value,
}

impl<T> Default for Document<T>
where
    T: Automergeable + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Document<T>
where
    T: Automergeable + Clone,
{
    /// Construct a new document.
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        let frontend = automerge::Frontend::new();
        let original = frontend
            .get_value(&Path::root())
            .expect("Failed to get root value");
        let value = T::from_automerge(&original).expect("Failed to load value");
        Self {
            frontend,
            value,
            original,
        }
    }

    /// Construct a new document with a given actor id.
    #[cfg(feature = "std")]
    pub fn new_with_actor_id(actor_id: uuid::Uuid) -> Self {
        let frontend = automerge::Frontend::new_with_actor_id(actor_id);
        let original = frontend
            .get_value(&Path::root())
            .expect("Failed to get root value");
        let value = T::from_automerge(&original).expect("Failed to load value");
        Self {
            frontend,
            value,
            original,
        }
    }

    /// Construct a new document with a given timestamper function.
    pub fn new_with_timestamper(t: Box<(dyn Fn() -> Option<i64>)>) -> Self {
        let frontend = automerge::Frontend::new_with_timestamper(t);
        let original = frontend
            .get_value(&Path::root())
            .expect("Failed to get root value");
        let value = T::from_automerge(&original).expect("Failed to load value");
        Self {
            frontend,
            value,
            original,
        }
    }

    /// Retrieve the root value from the frontend and convert it.
    pub fn get(&self) -> &T {
        &self.value
    }

    fn get_root(&self) -> Value {
        self.frontend
            .get_value(&Path::root())
            .expect("Failed to get root value")
    }

    fn change_inner<F, O, E>(
        &mut self,
        message: Option<String>,
        change: F,
    ) -> Result<(O, Option<automerge_protocol::UncompressedChange>), DocumentChangeError<E>>
    where
        E: Debug + Display,
        F: FnOnce(&mut T) -> Result<O, E>,
    {
        let mut new_t = self.value.clone();
        let res = change(&mut new_t).map_err(DocumentChangeError::ChangeError)?;
        let new_original = new_t.to_automerge();
        let changes = crate::diff_values(&new_original, &self.original)?;
        let ((), change) = self
            .frontend
            .change::<_, _, automerge::InvalidChangeRequest>(message, |doc| {
                for change in changes {
                    doc.add_change(change)?
                }
                Ok(())
            })?;
        self.value = new_t;
        self.original = new_original;
        Ok((res, change))
    }

    /// Perform a change on the frontend.
    pub fn change<F, O, E>(
        &mut self,
        change: F,
    ) -> Result<(O, Option<automerge_protocol::UncompressedChange>), DocumentChangeError<E>>
    where
        E: Debug + Display,
        F: FnOnce(&mut T) -> Result<O, E>,
    {
        self.change_inner(None, change)
    }

    /// Perform a change on the frontend with a message.
    pub fn change_with_message<F, O, E>(
        &mut self,
        message: String,
        change: F,
    ) -> Result<(O, Option<automerge_protocol::UncompressedChange>), DocumentChangeError<E>>
    where
        E: Debug + Display,
        F: FnOnce(&mut T) -> Result<O, E>,
    {
        self.change_inner(Some(message), change)
    }

    /// Apply a patch to the frontend, updating the stored value in the process.
    pub fn apply_patch(&mut self, patch: Patch) -> Result<(), ApplyPatchError> {
        self.frontend.apply_patch(patch)?;
        self.refresh_value()?;
        Ok(())
    }

    /// Set the internal typed value to that obtained from the frontend.
    ///
    /// This is intended to be used in case of interacting with the frontend directly.
    fn refresh_value(&mut self) -> Result<(), crate::FromAutomergeError> {
        self.original = self.get_root();
        self.value = T::from_automerge(&self.original)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_empty() {
        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct A {
            list: Vec<String>,
            others: std::collections::HashMap<String, String>,
            nah: Option<u64>,
            yep: Option<i64>,
            #[automergeable(representation = "Text")]
            some_text: String,
            #[automergeable(representation = "Counter")]
            a_counter: i64,
            #[automergeable(representation = "Timestamp")]
            a_timestamp: i64,
            b: B,
        }

        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct B {
            inner: u64,
        }
        #[cfg(feature = "std")]
        let mut doc = Document::<A>::new();
        #[cfg(not(feature = "std"))]
        let mut doc = Document::<A>::new_with_timestamper(Box::new(|| None));

        let mut back = automerge::Backend::init();
        let ((), change) = doc
            .change::<_, _, automerge::InvalidChangeRequest>(|_t| Ok(()))
            .unwrap();
        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }
    }

    #[test]
    fn create_from_empty_then_add_some_fields() {
        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct A {
            list: Vec<String>,
            others: std::collections::HashMap<String, String>,
            nah: Option<u64>,
            yep: Option<i64>,
            #[automergeable(representation = "Text")]
            some_text: String,
            #[automergeable(representation = "Counter")]
            a_counter: i64,
            #[automergeable(representation = "Timestamp")]
            a_timestamp: i64,
            b: B,
        }

        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct B {
            inner: u64,
        }

        #[cfg(feature = "std")]
        let mut doc = Document::<A>::new();
        #[cfg(not(feature = "std"))]
        let mut doc = Document::<A>::new_with_timestamper(Box::new(|| None));

        let mut back = automerge::Backend::init();
        let ((), change) = doc
            .change::<_, _, automerge::InvalidChangeRequest>(|t| {
                t.list.push("hi".to_owned());
                t.others.insert("hellow there".to_owned(), "abc".to_owned());
                Ok(())
            })
            .unwrap();
        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }
    }

    #[test]
    fn create_from_empty_then_add_some_fields_later() {
        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct A {
            list: Vec<String>,
            others: std::collections::HashMap<String, String>,
            nah: Option<u64>,
            yep: Option<i64>,
            #[automergeable(representation = "Text")]
            some_text: String,
            #[automergeable(representation = "Counter")]
            a_counter: i64,
            #[automergeable(representation = "Timestamp")]
            a_timestamp: i64,
            b: B,
        }

        #[derive(crate::Automergeable, Debug, Clone, Default)]
        #[automergeable(crate_path = "crate")]
        struct B {
            inner: u64,
        }
        #[cfg(feature = "std")]
        let mut doc = Document::<A>::new();
        #[cfg(not(feature = "std"))]
        let mut doc = Document::<A>::new_with_timestamper(Box::new(|| None));

        let mut back = automerge::Backend::init();
        let ((), change) = doc
            .change::<_, _, automerge::InvalidChangeRequest>(|t| {
                t.list.push("hi".to_owned());
                t.others.insert("hellow there".to_owned(), "abc".to_owned());
                Ok(())
            })
            .unwrap();

        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }

        let ((), change) = doc
            .change::<_, _, automerge::InvalidChangeRequest>(|t| {
                t.b.inner += 1;
                Ok(())
            })
            .unwrap();

        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }
    }
}
