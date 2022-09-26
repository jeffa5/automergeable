use automerge::{
    transaction::Transactable, transaction::Transaction, ChangeHash, ObjId, ScalarValue,
};
use smol_str::SmolStr;

use crate::TextView;

/// A mutable view over some text in the document.
#[derive(Debug)]
pub struct MutableTextView<'a, 't> {
    pub(crate) obj: ObjId,
    pub(crate) tx: &'t mut Transaction<'a>,
}

impl<'a, 't> PartialEq for MutableTextView<'a, 't> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self
                .iter()
                .into_iter()
                .zip(other.iter().into_iter())
                .all(|(a, b)| a == b)
    }
}

impl<'a, 't> MutableTextView<'a, 't> {
    pub fn into_immutable(self) -> TextView<'t, Transaction<'a>> {
        TextView {
            obj: self.obj,
            doc: self.tx,
        }
    }

    pub fn to_immutable<'s>(&'s self) -> TextView<'s, Transaction<'a>> {
        TextView {
            obj: self.obj.clone(),
            doc: self.tx,
        }
    }

    pub fn len(&self) -> usize {
        Transactable::length(self.tx, &self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<SmolStr> {
        match Transactable::get(self.tx, &self.obj, index) {
            Ok(Some((value, _))) => value.into_string().ok().map(Into::into),
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_at(&self, index: usize, heads: Vec<ChangeHash>) -> Option<SmolStr> {
        match Transactable::get_at(self.tx, &self.obj, index, &heads) {
            Ok(Some((value, _))) => value.into_string().ok().map(Into::into),
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<SmolStr> {
        match Transactable::get(self.tx, &self.obj, index) {
            Ok(Some((value, _))) => value.into_string().ok().map(Into::into),
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<V: Into<ScalarValue>>(&mut self, index: usize, value: V) {
        self.tx.insert(&self.obj, index, value).unwrap();
    }

    pub fn set<V: Into<ScalarValue>>(&mut self, index: usize, value: V) {
        self.tx.put(&self.obj, index, value).unwrap();
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.get(index).is_some() {
            self.tx.delete(&self.obj, index).unwrap();
            true
        } else {
            false
        }
    }

    pub fn as_string(&self) -> String {
        self.tx.text(&self.obj).unwrap()
    }

    // TODO: return an iterator
    pub fn iter(&self) -> Vec<SmolStr> {
        (0..self.len())
            .map(move |i| self.get(i).unwrap())
            .collect::<Vec<SmolStr>>()
    }
}
