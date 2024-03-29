use std::borrow::Cow;

use automerge::transaction::{Transactable, Transaction};
use automerge::{ChangeHash, ObjId, ObjType, ScalarValue, Value};

use crate::map::HistoricalMapView;
use crate::text::HistoricalTextView;
use crate::{
    HistoricalView, ListView, MapView, MutableMapView, MutableTextView, MutableView, TextView, View,
};

use super::HistoricalListView;

/// A mutable view over a list in the document.
#[derive(Debug)]
pub struct MutableListView<'a, 't> {
    pub(crate) obj: ObjId,
    pub(crate) tx: &'t mut Transaction<'a>,
}

impl<'a, 't> PartialEq for MutableListView<'a, 't> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 't> MutableListView<'a, 't> {
    pub fn into_immutable(self) -> ListView<'t, Transaction<'a>> {
        ListView {
            obj: self.obj,
            doc: self.tx,
        }
    }

    pub fn to_immutable<'s>(&'s self) -> ListView<'s, Transaction<'a>> {
        ListView {
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

    pub fn get<'s>(&'s self, index: usize) -> Option<View<'s, Transaction<'a>>> {
        match Transactable::get(self.tx, &self.obj, index) {
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
                Value::Scalar(s) => Some(View::Scalar(s.into_owned())),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_at<'s>(
        &'s self,
        index: usize,
        heads: Vec<ChangeHash>,
    ) -> Option<HistoricalView<'s, 'static, Transaction<'a>>> {
        match Transactable::get_at(self.tx, &self.obj, index, &heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(HistoricalView::Map(HistoricalMapView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(HistoricalView::List(HistoricalListView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Text) => Some(HistoricalView::Text(HistoricalTextView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Scalar(s) => Some(HistoricalView::Scalar(s.into_owned())),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut<'s>(&'s mut self, index: usize) -> Option<MutableView<'a, 's>> {
        match Transactable::get(self.tx, &self.obj, index) {
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
                Value::Scalar(s) => Some(MutableView::Scalar(s.into_owned())),
            },
            Ok(None) | Err(_) => None,
        }
    }

    /// Insert a new value into the list.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn insert<V: Into<ScalarValue>>(&mut self, index: usize, value: V) {
        self.tx.insert(&self.obj, index, value).unwrap()
    }

    /// Insert a new value into the list.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn insert_object<'s>(&'s mut self, index: usize, value: ObjType) -> MutableView<'a, 's> {
        let obj = self.tx.insert_object(&self.obj, index, value).unwrap();
        match value {
            ObjType::Map => MutableView::Map(MutableMapView { obj, tx: self.tx }),
            ObjType::Table => {
                todo!()
            }
            ObjType::List => MutableView::List(MutableListView { obj, tx: self.tx }),
            ObjType::Text => MutableView::Text(MutableTextView { obj, tx: self.tx }),
        }
    }

    /// Overwrite an existing item in the list.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn set<V: Into<ScalarValue>>(&mut self, index: usize, value: V) {
        self.tx.put(&self.obj, index, value).unwrap()
    }

    /// Overwrite an existing item in the list.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn set_object<'s>(&'s mut self, index: usize, value: ObjType) -> MutableView<'a, 's> {
        let obj = self.tx.put_object(&self.obj, index, value).unwrap();
        match value {
            ObjType::Map => MutableView::Map(MutableMapView { obj, tx: self.tx }),
            ObjType::Table => {
                todo!()
            }
            ObjType::List => MutableView::List(MutableListView { obj, tx: self.tx }),
            ObjType::Text => MutableView::Text(MutableTextView { obj, tx: self.tx }),
        }
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.get_mut(index).is_some() {
            self.tx.delete(&self.obj, index).unwrap();
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = View<Transaction<'a>>> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
