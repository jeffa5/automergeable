use automerge::{
    transaction::Transactable, transaction::Transaction, ChangeHash, ObjId, ScalarValue, Value,
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
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
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
        match Transactable::value(self.tx, &self.obj, index) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_at(&self, index: usize, heads: Vec<ChangeHash>) -> Option<SmolStr> {
        match Transactable::value_at(self.tx, &self.obj, index, &heads) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<SmolStr> {
        match Transactable::value(self.tx, &self.obj, index) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.tx.insert(&self.obj, index, value).unwrap();
    }

    pub fn set<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.tx.set(&self.obj, index, value).unwrap();
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.get(index).is_some() {
            self.tx.del(&self.obj, index).unwrap();
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = SmolStr> + 'a + 't {
        // (0..self.len()).map(|i| self.get(i).unwrap())
        std::iter::empty()
    }
}
