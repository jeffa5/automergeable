use std::borrow::Cow;

use automerge::transaction::{Transactable, Transaction};
use automerge::{ChangeHash, ObjId, ObjType, Value};

use crate::map::HistoricMapView;
use crate::text::HistoricTextView;
use crate::{
    HistoricView, ListView, MapView, MutableMapView, MutableTextView, MutableView, TextView, View,
};

use super::HistoricListView;

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

    pub fn len(&self) -> usize {
        Transactable::length(self.tx, &self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<'s>(&'s self, index: usize) -> Option<View<'s, Transaction<'a>>> {
        match Transactable::value(self.tx, &self.obj, index) {
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

    pub fn get_at<'s>(
        &'s self,
        index: usize,
        heads: Vec<ChangeHash>,
    ) -> Option<HistoricView<'s, 'static, Transaction<'a>>> {
        match Transactable::value_at(self.tx, &self.obj, index, &heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(HistoricView::Map(HistoricMapView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(HistoricView::List(HistoricListView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Text) => Some(HistoricView::Text(HistoricTextView {
                    obj: id,
                    doc: self.tx,
                    heads: Cow::Owned(heads),
                })),
                Value::Scalar(s) => Some(HistoricView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut<'s>(&'s mut self, index: usize) -> Option<MutableView<'a, 's>> {
        match Transactable::value(self.tx, &self.obj, index) {
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

    /// Insert a new value into the list.
    pub fn insert<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.tx.insert(&self.obj, index, value).unwrap();
    }

    /// Overwrite an existing item in the list.
    pub fn set<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.tx.set(&self.obj, index, value).unwrap();
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.get_mut(index).is_some() {
            self.tx.del(&self.obj, index).unwrap();
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = View<Transaction<'a>>> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
