use automerge::{ObjId, ScalarValue, Value};
use smol_str::SmolStr;

use crate::Viewable;

/// A view over some text in the document.
#[derive(Debug, Clone)]
pub struct TextView<'a, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
}

impl<'a, 'oa, V, OV> PartialEq<TextView<'oa, OV>> for TextView<'a, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &TextView<'oa, OV>) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, V> TextView<'a, V>
where
    V: Viewable,
{
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

    pub fn as_string(&self) -> String {
        self.doc.text(&self.obj).unwrap()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = SmolStr> + '_ {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
