use automerge::{ObjId, ObjType, Value};

use crate::{ListView, TextView, View, Viewable};

/// A view over a map in this document.
#[derive(Debug)]
pub struct MapView<'a, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
}

impl<'a, 'oa, V, OV> PartialEq<MapView<'oa, OV>> for MapView<'a, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &MapView<'oa, OV>) -> bool {
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

impl<'a, V> MapView<'a, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.keys(&self.obj).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the value at the given key in this map.
    pub fn get<S: Into<String>>(&self, key: S) -> Option<View<'a, V>> {
        match self.doc.value(&self.obj, key.into()) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
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
        self.doc.keys(&self.obj).into_iter()
    }

    /// Get the values in this map, returned in sorted order of their keys.
    pub fn values(&self) -> impl Iterator<Item = View<V>> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    /// Get both the keys and values in this map.
    pub fn iter(&self) -> impl Iterator<Item = (String, View<V>)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}
