use std::borrow::Cow;

use automerge::{ChangeHash, ObjId, ObjType, Value};

use crate::{
    historical::HistoricalView, map::HistoricalMapView, text::HistoricalTextView, Viewable,
};

/// A view over a list in the document.
#[derive(Debug, Clone)]
pub struct HistoricalListView<'a, 'h, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricalListView<'oa, 'oh, OV>>
    for HistoricalListView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricalListView<'oa, 'oh, OV>) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 'h, V> HistoricalListView<'a, 'h, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.length_at(&self.obj, &self.heads)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<HistoricalView<'a, 'h, V>> {
        match self.doc.value_at(&self.obj, index, &self.heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(HistoricalView::Map(HistoricalMapView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(HistoricalView::List(HistoricalListView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Text) => Some(HistoricalView::Text(HistoricalTextView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Scalar(s) => Some(HistoricalView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = HistoricalView<V>> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
