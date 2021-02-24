use std::{
    error::Error,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use automerge::Path;

#[derive(Default)]
pub struct Document<T>
where
    T: crate::Automergeable,
{
    frontend: automerge::Frontend,
    _data: PhantomData<T>,
}

impl<T> Document<T>
where
    T: crate::Automergeable,
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
        <T as crate::FromAutomerge>::Error: std::error::Error,
    {
        let original = match self.frontend.get_value(&Path::root()) {
            Some(value) => T::from_automerge(&value).unwrap(),
            None => T::default(),
        };
        let mut new_t = original.clone();
        change(&mut new_t).unwrap();
        let changes = crate::diff(&new_t.to_automerge(), &original.to_automerge());
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
        <T as crate::FromAutomerge>::Error: Error,
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
        <T as crate::FromAutomerge>::Error: Error,
    {
        self.change_inner(Some(message), change)
    }
}

impl<T> Deref for Document<T>
where
    T: crate::Automergeable,
{
    type Target = automerge::Frontend;

    fn deref(&self) -> &Self::Target {
        &self.frontend
    }
}

impl<T> DerefMut for Document<T>
where
    T: crate::Automergeable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frontend
    }
}
