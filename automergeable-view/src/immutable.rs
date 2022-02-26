use automerge::{Prop, ScalarValue};

use crate::{ListView, MapView, TextView, Viewable};

/// A view over a document, providing a lightweight way to traverse it.
#[derive(Debug)]
pub enum View<'a, V> {
    Map(MapView<'a, V>),
    List(ListView<'a, V>),
    Text(TextView<'a, V>),
    Scalar(ScalarValue),
}

impl<'a, 'oa, V, OV> PartialEq<View<'oa, OV>> for View<'a, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &View<'oa, OV>) -> bool {
        match (self, other) {
            (Self::Map(l0), View::Map(r0)) => l0 == r0,
            (Self::List(l0), View::List(r0)) => l0 == r0,
            (Self::Text(l0), View::Text(r0)) => l0 == r0,
            (Self::Scalar(l0), View::Scalar(r0)) => l0 == r0,
            (Self::Map(_), View::List(_))
            | (Self::Map(_), View::Text(_))
            | (Self::Map(_), View::Scalar(_))
            | (Self::List(_), View::Map(_))
            | (Self::List(_), View::Text(_))
            | (Self::List(_), View::Scalar(_))
            | (Self::Text(_), View::Map(_))
            | (Self::Text(_), View::List(_))
            | (Self::Text(_), View::Scalar(_))
            | (Self::Scalar(_), View::Map(_))
            | (Self::Scalar(_), View::List(_))
            | (Self::Scalar(_), View::Text(_)) => false,
        }
    }
}

impl<'a, V> View<'a, V>
where
    V: Viewable,
{
    /// Get the view of the object at `prop`.
    pub fn get<P: Into<Prop>>(&self, prop: P) -> Option<View<'a, V>> {
        match (prop.into(), self) {
            (Prop::Map(key), View::Map(map)) => map.get(key),
            (Prop::Seq(index), View::List(l)) => l.get(index),
            (Prop::Seq(index), View::Text(t)) => {
                t.get(index).map(|s| View::Scalar(ScalarValue::Str(s)))
            }
            (Prop::Seq(_), View::Map(_))
            | (Prop::Map(_), View::List(_))
            | (Prop::Map(_), View::Text(_))
            | (_, View::Scalar(_)) => None,
        }
    }

    /// Get the length of this object.
    pub fn len(&self) -> usize {
        match self {
            View::Map(map) => map.len(),
            View::List(list) => list.len(),
            View::Text(text) => text.len(),
            View::Scalar(_) => 0,
        }
    }

    /// Check if this object is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Try and extract a map from this view.
    pub fn map(&self) -> Option<&MapView<'a, V>> {
        if let View::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a map from this view.
    pub fn into_map(self) -> Option<MapView<'a, V>> {
        if let View::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn list(&self) -> Option<&ListView<'a, V>> {
        if let View::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn into_list(self) -> Option<ListView<'a, V>> {
        if let View::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn text(&self) -> Option<&TextView<'a, V>> {
        if let View::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn into_text(self) -> Option<TextView<'a, V>> {
        if let View::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract a scalar value from this view.
    pub fn scalar(&self) -> Option<ScalarValue> {
        if let View::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}

impl<V> From<u64> for View<'static, V> {
    fn from(u: u64) -> Self {
        View::Scalar(ScalarValue::Uint(u))
    }
}

impl<V> From<i32> for View<'static, V> {
    fn from(i: i32) -> Self {
        View::Scalar(ScalarValue::Int(i as i64))
    }
}
