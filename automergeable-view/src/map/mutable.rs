use automerge::{transaction::Transactable, transaction::Transaction, Keys, ObjId, ObjType, Value};

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

    pub fn to_immutable<'s>(&'s self) -> MapView<'s, Transaction<'a>> {
        MapView {
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

    /// Overwrite the `prop`'s current value with `value`.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn insert<'s, S: Into<String>, V: Into<Value>>(
        &'s mut self,
        key: S,
        value: V,
    ) -> Option<MutableView<'a, 's>> {
        let value: Value = value.into();
        let typ = if let Value::Object(typ) = value {
            Some(typ)
        } else {
            None
        };
        self.tx
            .set(&self.obj, key.into(), value)
            .unwrap()
            .map(move |obj| match typ {
                Some(ObjType::Map) => MutableView::Map(MutableMapView { obj, tx: self.tx }),
                Some(ObjType::Table) => {
                    todo!()
                }
                Some(ObjType::List) => MutableView::List(MutableListView { obj, tx: self.tx }),
                Some(ObjType::Text) => MutableView::Text(MutableTextView { obj, tx: self.tx }),
                None => unreachable!(),
            })
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

    pub fn keys(&self) -> Keys {
        Transactable::keys(self.tx, &self.obj)
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = View<Transaction<'a>>> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (String, View<Transaction<'a>>)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}
