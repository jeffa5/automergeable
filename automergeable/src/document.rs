use std::{error::Error, fmt::Debug};

use automerge::{Path, Value};
use automerge_frontend::MutableDocument;
use automerge_protocol::Patch;

use crate::Automergeable;

/// An error type for change operations on documents.
#[derive(Debug, thiserror::Error)]
pub enum DocumentChangeError<E: Error = std::convert::Infallible> {
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
pub enum ApplyPatchError<E: Error> {
    #[error("frontend error: {0}")]
    FrontendError(E),
    #[error(transparent)]
    FromError(#[from] crate::FromAutomergeError),
}

pub trait Frontend {
    type Error: Error;

    fn get_value(&self, path: &Path) -> Result<Option<Value>, Self::Error>;

    fn change<C, E>(
        &mut self,
        message: Option<String>,
        closure: C,
    ) -> Result<Option<automerge_protocol::Change>, E>
    where
        C: FnOnce(&mut dyn MutableDocument) -> Result<(), E>,
        E: Error;

    fn apply_patch(&mut self, patch: Patch) -> Result<(), Self::Error>;
}

impl Frontend for automerge::Frontend {
    type Error = automerge_frontend::InvalidPatch;

    fn get_value(&self, path: &Path) -> Result<Option<Value>, Self::Error> {
        Ok(self.get_value(path))
    }

    fn change<C, E>(
        &mut self,
        message: Option<String>,
        closure: C,
    ) -> Result<Option<automerge_protocol::Change>, E>
    where
        C: FnOnce(&mut dyn MutableDocument) -> Result<(), E>,
        E: Error,
    {
        let ((), change) = self.change(message, closure)?;
        Ok(change)
    }

    fn apply_patch(&mut self, patch: Patch) -> Result<(), Self::Error> {
        self.apply_patch(patch)
    }
}

/// A typed automerge document, wrapping a typical frontend.
///
/// This provides similar functionality to an automerge frontend (including [`Deref`] to one) but with
/// stronger typed data.
///
/// For instance from a document we can get the value as a typical Rust struct and perform
/// automerge change operations on it with automatic diffing behind the scenes.
#[derive(Debug)]
pub struct Document<T, F>
where
    T: Automergeable,
    F: Frontend,
{
    frontend: F,
    value: T,
    original: Value,
}

impl<T, F> Document<T, F>
where
    T: Automergeable + Clone,
    F: Frontend,
{
    /// Construct a new document.
    pub fn new(frontend: F) -> Self {
        let original = frontend
            .get_value(&Path::root())
            .expect("Failed to get root value")
            .expect("No root value");
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

    fn get_root(&self) -> Result<Value, F::Error> {
        Ok(self
            .frontend
            .get_value(&Path::root())?
            .expect("Failed to get root value"))
    }

    fn change_inner<C, O, E>(
        &mut self,
        message: Option<String>,
        change: C,
    ) -> Result<(O, Option<automerge_protocol::Change>), DocumentChangeError<E>>
    where
        E: Error,
        C: FnOnce(&mut T) -> Result<O, E>,
    {
        let mut new_t = self.value.clone();
        let res = change(&mut new_t).map_err(DocumentChangeError::ChangeError)?;
        let new_original = new_t.to_automerge();
        let changes = crate::diff_values(&new_original, &self.original)?;
        let change =
            self.frontend
                .change::<_, automerge::InvalidChangeRequest>(message, |doc| {
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
    pub fn change<C, O, E>(
        &mut self,
        change: C,
    ) -> Result<(O, Option<automerge_protocol::Change>), DocumentChangeError<E>>
    where
        E: Error,
        C: FnOnce(&mut T) -> Result<O, E>,
    {
        self.change_inner(None, change)
    }

    /// Perform a change on the frontend with a message.
    pub fn change_with_message<C, O, E>(
        &mut self,
        message: String,
        change: C,
    ) -> Result<(O, Option<automerge_protocol::Change>), DocumentChangeError<E>>
    where
        E: Error,
        C: FnOnce(&mut T) -> Result<O, E>,
    {
        self.change_inner(Some(message), change)
    }

    /// Apply a patch to the frontend, updating the stored value in the process.
    pub fn apply_patch(&mut self, patch: Patch) -> Result<(), ApplyPatchError<F::Error>> {
        self.frontend
            .apply_patch(patch)
            .map_err(ApplyPatchError::FrontendError)?;
        self.refresh_value()?;
        Ok(())
    }

    /// Set the internal typed value to that obtained from the frontend.
    ///
    /// This is intended to be used in case of interacting with the frontend directly.
    fn refresh_value(&mut self) -> Result<(), ApplyPatchError<F::Error>> {
        // TODO: change this to a new error type
        self.original = self.get_root().map_err(ApplyPatchError::FrontendError)?;
        self.value = T::from_automerge(&self.original)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use automerge::Frontend;

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

        let mut doc = Document::<A, _>::new(Frontend::new());

        let mut back = automerge::Backend::new();
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

        let mut doc = Document::<A, _>::new(Frontend::new());

        let mut back = automerge::Backend::new();
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

        let mut doc = Document::<A, _>::new(Frontend::new());

        let mut back = automerge::Backend::new();
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
