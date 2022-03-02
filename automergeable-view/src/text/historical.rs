use std::borrow::Cow;

use automerge::{ChangeHash, ObjId, ScalarValue, Value};
use smol_str::SmolStr;

use crate::Viewable;

/// A view over some text in the document.
#[derive(Debug, Clone)]
pub struct HistoricalTextView<'a, 'h, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricalTextView<'oa, 'oh, OV>>
    for HistoricalTextView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricalTextView<'oa, 'oh, OV>) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 'h, V> HistoricalTextView<'a, 'h, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.length_at(&self.obj, &self.heads)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<SmolStr> {
        match self.doc.value_at(&self.obj, index, &self.heads) {
            Ok(Some((value, _))) => match value {
                Value::Scalar(ScalarValue::Str(s)) => Some(s),
                Value::Object(_) | Value::Scalar(_) => None,
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn as_string(&self) -> String {
        self.doc.text_at(&self.obj, &self.heads).unwrap()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = SmolStr> + '_ {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
