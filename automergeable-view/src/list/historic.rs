use std::borrow::Cow;

use automerge::{ChangeHash, ObjId, ObjType, Value};

use crate::{historic::HistoricView, map::HistoricMapView, text::HistoricTextView, Viewable};

/// A view over a list in the document.
#[derive(Debug, Clone)]
pub struct HistoricListView<'a, 'h, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricListView<'oa, 'oh, OV>>
    for HistoricListView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricListView<'oa, 'oh, OV>) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 'h, V> HistoricListView<'a, 'h, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.length_at(&self.obj, &self.heads)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<HistoricView<'a, 'h, V>> {
        match self.doc.value_at(&self.obj, index, &self.heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(HistoricView::Map(HistoricMapView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(HistoricView::List(HistoricListView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Text) => Some(HistoricView::Text(HistoricTextView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Scalar(s) => Some(HistoricView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = HistoricView<V>> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
