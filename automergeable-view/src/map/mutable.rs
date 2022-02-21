use automerge::{transaction::Transactable, transaction::Transaction, ObjId, ObjType, Value};

use crate::{ListView, MapView, MutableListView, MutableTextView, MutableView, TextView, View};

/// A mutable view over a map in the document.
#[derive(Debug)]
pub struct MutableMapView<'a, 't> {
    pub(crate) obj: ObjId,
    pub(crate) tx: &'t mut Transaction<'a>,
}

impl<'a, 't> PartialEq for MutableMapView<'a, 't> {
    fn eq(&self, other: &Self) -> bool {
        if self.obj == other.obj && self.len() == other.len() {
            let mut our_keys = self.iter().collect::<Vec<_>>();
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

impl<'a, 't> MutableMapView<'a, 't> {
    pub fn into_immutable(self) -> MapView<'t, Transaction<'a>> {
        MapView {
            obj: self.obj,
            doc: self.tx,
        }
    }

    pub fn len(&self) -> usize {
        Transactable::keys(self.tx, &self.obj).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<'s, S: Into<String>>(&'s self, key: S) -> Option<View<'s, Transaction<'a>>> {
        match Transactable::value(self.tx, &self.obj, key.into()) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.tx,
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.tx,
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.tx,
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut<'s, S: Into<String>>(&'s mut self, key: S) -> Option<MutableView<'a, 's>> {
        match Transactable::value(self.tx, &self.obj, key.into()) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(MutableView::Map(MutableMapView {
                    obj: id,
                    tx: self.tx,
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(MutableView::List(MutableListView {
                    obj: id,
                    tx: self.tx,
                })),
                Value::Object(ObjType::Text) => Some(MutableView::Text(MutableTextView {
                    obj: id,
                    tx: self.tx,
                })),
                Value::Scalar(s) => Some(MutableView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<S: Into<String>, V: Into<Value>>(&mut self, key: S, value: V) {
        self.tx.set(&self.obj, key.into(), value).unwrap();
    }

    /// Remove a value from this map, returning a whether a value was removed or not.
    pub fn remove<S: Into<String>>(&mut self, key: S) -> bool {
        let key = key.into();
        if self.get(key.clone()).is_some() {
            self.tx.del(&self.obj, key).unwrap();
            true
        } else {
            false
        }
    }

    pub fn contains_key<S: Into<String>>(&self, key: S) -> bool {
        self.get(key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        Transactable::keys(self.tx, &self.obj).into_iter()
    }

    pub fn values(&self) -> impl Iterator<Item = View<Transaction<'a>>> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, View<Transaction<'a>>)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}