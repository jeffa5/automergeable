use std::borrow::Cow;

use automerge::{ChangeHash, ObjId, ObjType, Value};

use crate::{historic::HistoricView, list::HistoricListView, text::HistoricTextView, Viewable};

/// A view over a map in this document.
#[derive(Debug)]
pub struct HistoricMapView<'a, 'h, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricMapView<'oa, 'oh, OV>>
    for HistoricMapView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricMapView<'oa, 'oh, OV>) -> bool {
        if self.obj == other.obj && self.len() == other.len() {
            let mut our_keys = self.iter().collect::<Vec<_>>();
            // TODO: our the keys guaranteed to be in sorted order? If so, we can skip the extra
            // sorting
            our_keys.sort_by_key(|(key, _)| key.clone());
            let mut other_keys = other.iter().collect::<Vec<_>>();
            other_keys.sort_by_key(|(key, _)| key.clone());
            our_keys
                .into_iter()
                .zip(other_keys)
                .all(|((ak, av), (bk, bv))| ak == bk && av == bv)
        } else {
            false
        }
    }
}

impl<'a, 'h, V> HistoricMapView<'a, 'h, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.keys_at(&self.obj, &self.heads).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the value at the given key in this map.
    pub fn get<S: Into<String>>(&self, key: S) -> Option<HistoricView<'a, 'h, V>> {
        match self.doc.value_at(&self.obj, key.into(), &self.heads) {
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

    /// Check if this map contains a key.
    pub fn contains_key<S: Into<String>>(&self, key: S) -> bool {
        self.get(key).is_some()
    }

    /// Get the keys in this map, returned in sorted order.
    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.doc.keys_at(&self.obj, &self.heads).into_iter()
    }

    /// Get the values in this map, returned in sorted order of their keys.
    pub fn values(&self) -> impl Iterator<Item = HistoricView<V>> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    /// Get both the keys and values in this map.
    pub fn iter(&self) -> impl Iterator<Item = (String, HistoricView<V>)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}
