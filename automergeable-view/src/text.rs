use std::borrow::Cow;

use automerge::{Automerge, ChangeHash, ObjId, ScalarValue, Value};
use smol_str::SmolStr;

/// A view over some text in the document.
#[derive(Debug, Clone)]
pub struct TextView<'a, 'h> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a Automerge,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h> PartialEq for TextView<'a, 'h> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 'h> TextView<'a, 'h> {
    pub fn len(&self) -> usize {
        self.doc.length(&self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<SmolStr> {
        match self.doc.value(&self.obj, index) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = SmolStr> + '_ {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}

/// A mutable view over some text in the document.
#[derive(Debug)]
pub struct MutableTextView<'a> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a mut Automerge,
}

impl<'a> PartialEq for MutableTextView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a> MutableTextView<'a> {
    pub fn into_immutable(self) -> TextView<'a, 'static> {
        let heads = self.doc.get_heads();
        TextView {
            obj: self.obj,
            doc: self.doc,
            heads: Cow::Owned(heads),
        }
    }

    pub fn len(&self) -> usize {
        self.doc.length(&self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<SmolStr> {
        match self.doc.value(&self.obj, index) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    fn get_at(&self, index: usize, heads: Vec<ChangeHash>) -> Option<SmolStr> {
        match self.doc.value_at(&self.obj, index, &heads) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<SmolStr> {
        match self.doc.value(&self.obj, index) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.doc.insert(&self.obj, index, value).unwrap();
    }

    pub fn set<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.doc.set(&self.obj, index, value).unwrap();
    }

    pub fn remove(&mut self, index: usize) -> Option<SmolStr> {
        let heads = self.doc.get_heads();
        if self.get(index).is_some() {
            self.doc.del(&self.obj, index).unwrap();
            Some(self.get_at(index, heads).unwrap())
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = SmolStr> + '_ {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ViewableDoc;

    #[test]
    fn test_text() {
        let mut doc = Automerge::new();
        doc.view_mut().insert("a", Value::text());
        doc.view_mut().get_mut("a").unwrap().insert(0, "b");
        doc.view_mut().get_mut("a").unwrap().insert(1, "c");

        let text = doc.view().get("a").unwrap().text().unwrap();

        assert_eq!(text.get(0), Some("b".into()));

        assert_eq!(text.len(), 2);

        assert!(!text.is_empty());

        assert_eq!(
            text.iter().collect::<Vec<_>>(),
            vec!["b".to_string(), "c".to_string()]
        );
    }
}