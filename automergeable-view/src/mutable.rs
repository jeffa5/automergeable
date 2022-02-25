use automerge::{transaction::Transaction, Prop, ScalarValue, Value};

use crate::{ListView, MapView, MutableListView, MutableMapView, MutableTextView, TextView, View};

/// A mutable view over the document, allowing editing.
#[derive(Debug, PartialEq)]
pub enum MutableView<'a, 't> {
    Map(MutableMapView<'a, 't>),
    List(MutableListView<'a, 't>),
    Text(MutableTextView<'a, 't>),
    Scalar(ScalarValue),
}

impl<'a, 't> MutableView<'a, 't> {
    pub fn into_immutable(self) -> View<'t, Transaction<'a>> {
        match self {
            MutableView::Map(map) => View::Map(map.into_immutable()),
            MutableView::List(list) => View::List(list.into_immutable()),
            MutableView::Text(text) => View::Text(text.into_immutable()),
            MutableView::Scalar(scalar) => View::Scalar(scalar),
        }
    }

    /// Get the mutable view of the object at `prop`.
    pub fn get<'s, P: Into<Prop>>(&'s self, prop: P) -> Option<View<'s, Transaction<'a>>> {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.get(key),
            (Prop::Seq(index), MutableView::List(l)) => l.get(index),
            (Prop::Seq(index), MutableView::Text(t)) => {
                t.get(index).map(|s| View::Scalar(ScalarValue::Str(s)))
            }
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    /// Get the mutable view of the object at `prop`.
    pub fn get_mut<'s, P: Into<Prop>>(&'s mut self, prop: P) -> Option<MutableView<'a, 's>> {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.get_mut(key),
            (Prop::Seq(index), MutableView::List(l)) => l.get_mut(index),
            (Prop::Seq(index), MutableView::Text(t)) => t
                .get_mut(index)
                .map(|s| MutableView::Scalar(ScalarValue::Str(s))),
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    /// Insert the given value at the `prop`.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn insert<'s, P: Into<Prop>, V: Into<Value>>(
        &'s mut self,
        prop: P,
        value: V,
    ) -> Option<MutableView<'a, 's>> {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.insert(key, value),
            (Prop::Seq(index), MutableView::List(list)) => list.insert(index, value),
            (Prop::Seq(index), MutableView::Text(text)) => {
                text.insert(index, value);
                // text can't contain objects
                None
            }
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    /// Overwrite the `prop`'s current value with `value`.
    ///
    /// Returns a mutable view of the new object if the value created one.
    pub fn set<'s, P: Into<Prop>, V: Into<Value>>(
        &'s mut self,
        prop: P,
        value: V,
    ) -> Option<MutableView<'a, 's>> {
        match (prop.into(), self) {
            // map's insert does the same as a set would
            (Prop::Map(key), MutableView::Map(map)) => map.insert(key, value),
            (Prop::Seq(index), MutableView::List(list)) => list.set(index, value),
            (Prop::Seq(index), MutableView::Text(text)) => {
                text.set(index, value);
                None
            }
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    /// Remove the value at `prop` if it exists.
    pub fn remove<P: Into<Prop>>(&mut self, prop: P) -> bool {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.remove(key),
            (Prop::Seq(index), MutableView::List(list)) => list.remove(index),
            (Prop::Seq(index), MutableView::Text(text)) => text.remove(index),
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => false,
        }
    }

    /// Get the length of this object.
    pub fn len(&self) -> usize {
        match self {
            MutableView::Map(map) => map.len(),
            MutableView::List(list) => list.len(),
            MutableView::Text(text) => text.len(),
            MutableView::Scalar(_) => 0,
        }
    }

    /// Check if this object is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Try and extract a mutable map from this view.
    pub fn map<'s>(&'s self) -> Option<MapView<'s, Transaction<'a>>> {
        if let MutableView::Map(map) = self {
            Some(map.to_immutable())
        } else {
            None
        }
    }

    /// Try and extract a mutable map from this view.
    pub fn map_mut(&mut self) -> Option<&mut MutableMapView<'a, 't>> {
        if let MutableView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a mutable map from this view.
    pub fn into_map(self) -> Option<MapView<'t, Transaction<'a>>> {
        if let MutableView::Map(map) = self {
            Some(map.into_immutable())
        } else {
            None
        }
    }

    /// Try and extract a mutable map from this view.
    pub fn into_map_mut(self) -> Option<MutableMapView<'a, 't>> {
        if let MutableView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a mutable list from this view.
    pub fn list<'s>(&'s self) -> Option<ListView<'s, Transaction<'a>>> {
        if let MutableView::List(list) = self {
            Some(list.to_immutable())
        } else {
            None
        }
    }

    /// Try and extract a mutable list from this view.
    pub fn list_mut(&mut self) -> Option<&mut MutableListView<'a, 't>> {
        if let MutableView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract a mutable list from this view.
    pub fn into_list(self) -> Option<ListView<'t, Transaction<'a>>> {
        if let MutableView::List(list) = self {
            Some(list.into_immutable())
        } else {
            None
        }
    }

    /// Try and extract a mutable list from this view.
    pub fn into_list_mut(self) -> Option<MutableListView<'a, 't>> {
        if let MutableView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract mutable text from this view.
    pub fn text<'s>(&'s self) -> Option<TextView<'s, Transaction<'a>>> {
        if let MutableView::Text(text) = self {
            Some(text.to_immutable())
        } else {
            None
        }
    }

    /// Try and extract mutable text from this view.
    pub fn text_mut(&mut self) -> Option<&mut MutableTextView<'a, 't>> {
        if let MutableView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract mutable text from this view.
    pub fn into_text(self) -> Option<TextView<'t, Transaction<'a>>> {
        if let MutableView::Text(text) = self {
            Some(text.into_immutable())
        } else {
            None
        }
    }

    /// Try and extract mutable text from this view.
    pub fn into_text_mut(self) -> Option<MutableTextView<'a, 't>> {
        if let MutableView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract a scalar value from this view.
    pub fn scalar(&self) -> Option<ScalarValue> {
        if let MutableView::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}
