use std::{
    error::Error,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use automerge::{Path, Value};

use crate::Automergeable;

#[derive(Default)]
pub struct Document<T>
where
    T: Automergeable,
{
    frontend: automerge::Frontend,
    _data: PhantomData<T>,
}

impl<T> Document<T>
where
    T: Automergeable + Clone + Default,
{
    pub fn new() -> Self {
        Self {
            frontend: automerge::Frontend::new(),
            _data: PhantomData,
        }
    }

    fn change_inner<F, E>(
        &mut self,
        message: Option<String>,
        change: F,
    ) -> Result<Option<automerge_protocol::UncompressedChange>, Box<dyn Error>>
    where
        E: Error,
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        let original = self
            .frontend
            .get_value(&Path::root())
            .map(|value| match &value {
                Value::Map(hm, automerge::MapType::Map) if !hm.is_empty() => {
                    Some(T::from_automerge(&value).expect("original from frontend was not valid"))
                }
                _ => None,
            })
            .flatten();
        let mut new_t = original.as_ref().cloned().unwrap_or_default();
        change(&mut new_t).unwrap();
        let changes = crate::diff(&Some(new_t), &original);
        let change =
            self.frontend
                .change::<_, automerge::InvalidChangeRequest>(message, |doc| {
                    for change in changes {
                        doc.add_change(change)?
                    }
                    Ok(())
                })?;
        Ok(change)
    }

    pub fn change<F, E>(
        &mut self,
        change: F,
    ) -> Result<Option<automerge_protocol::UncompressedChange>, Box<dyn Error>>
    where
        E: Error,
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        self.change_inner(None, change)
    }

    pub fn change_with_message<F, E>(
        &mut self,
        message: String,
        change: F,
    ) -> Result<Option<automerge_protocol::UncompressedChange>, Box<dyn Error>>
    where
        E: Error,
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        self.change_inner(Some(message), change)
    }
}

impl<T> Deref for Document<T>
where
    T: Automergeable,
{
    type Target = automerge::Frontend;

    fn deref(&self) -> &Self::Target {
        &self.frontend
    }
}

impl<T> DerefMut for Document<T>
where
    T: Automergeable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frontend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_empty() {
        #[derive(crate::Automergeable, Debug, Clone, Default)]
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
        struct B {
            inner: u64,
        }
        let mut doc: Document<A> = Document::new();
        let mut back = automerge::Backend::init();
        let change = doc
            .change::<_, automerge::InvalidChangeRequest>(|_t| Ok(()))
            .unwrap();
        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }
    }

    #[test]
    fn create_from_empty_then_add_some_fields() {
        #[derive(crate::Automergeable, Debug, Clone, Default)]
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
        struct B {
            inner: u64,
        }
        let mut doc: Document<A> = Document::new();
        let mut back = automerge::Backend::init();
        let change = doc
            .change::<_, automerge::InvalidChangeRequest>(|t| {
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
        struct B {
            inner: u64,
        }
        let mut doc: Document<A> = Document::new();
        let mut back = automerge::Backend::init();
        let change = doc
            .change::<_, automerge::InvalidChangeRequest>(|t| {
                t.list.push("hi".to_owned());
                t.others.insert("hellow there".to_owned(), "abc".to_owned());
                Ok(())
            })
            .unwrap();

        if let Some(change) = change {
            let (patch, _) = back.apply_local_change(change).unwrap();
            doc.apply_patch(patch).unwrap();
        }

        let change = doc
            .change::<_, automerge::InvalidChangeRequest>(|t| {
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
