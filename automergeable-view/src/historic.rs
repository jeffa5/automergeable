use automerge::{Prop, ScalarValue};

use crate::{list::HistoricListView, map::HistoricMapView, text::HistoricTextView, Viewable};

/// A view over a document, providing a lightweight way to traverse it.
#[derive(Debug)]
pub enum HistoricView<'a, 'h, V> {
    Map(HistoricMapView<'a, 'h, V>),
    List(HistoricListView<'a, 'h, V>),
    Text(HistoricTextView<'a, 'h, V>),
    Scalar(ScalarValue),
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricView<'oa, 'oh, OV>> for HistoricView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricView<'oa, 'oh, OV>) -> bool {
        match (self, other) {
            (Self::Map(l0), HistoricView::Map(r0)) => l0 == r0,
            (Self::List(l0), HistoricView::List(r0)) => l0 == r0,
            (Self::Text(l0), HistoricView::Text(r0)) => l0 == r0,
            (Self::Scalar(l0), HistoricView::Scalar(r0)) => l0 == r0,
            (Self::Map(_), HistoricView::List(_))
            | (Self::Map(_), HistoricView::Text(_))
            | (Self::Map(_), HistoricView::Scalar(_))
            | (Self::List(_), HistoricView::Map(_))
            | (Self::List(_), HistoricView::Text(_))
            | (Self::List(_), HistoricView::Scalar(_))
            | (Self::Text(_), HistoricView::Map(_))
            | (Self::Text(_), HistoricView::List(_))
            | (Self::Text(_), HistoricView::Scalar(_))
            | (Self::Scalar(_), HistoricView::Map(_))
            | (Self::Scalar(_), HistoricView::List(_))
            | (Self::Scalar(_), HistoricView::Text(_)) => false,
        }
    }
}

impl<'a, 'h, V> HistoricView<'a, 'h, V>
where
    V: Viewable,
{
    /// Get the view of the object at `prop`.
    pub fn get<P: Into<Prop>>(&self, prop: P) -> Option<HistoricView<'a, 'h, V>> {
        match (prop.into(), self) {
            (Prop::Map(key), HistoricView::Map(map)) => map.get(key),
            (Prop::Seq(index), HistoricView::List(l)) => l.get(index),
            (Prop::Seq(index), HistoricView::Text(t)) => t
                .get(index)
                .map(|s| HistoricView::Scalar(ScalarValue::Str(s))),
            (Prop::Seq(_), HistoricView::Map(_))
            | (Prop::Map(_), HistoricView::List(_))
            | (Prop::Map(_), HistoricView::Text(_))
            | (_, HistoricView::Scalar(_)) => None,
        }
    }

    /// Get the length of this object.
    pub fn len(&self) -> usize {
        match self {
            HistoricView::Map(map) => map.len(),
            HistoricView::List(list) => list.len(),
            HistoricView::Text(text) => text.len(),
            HistoricView::Scalar(_) => 0,
        }
    }

    /// Check if this object is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Try and extract a map from this view.
    pub fn map(&self) -> Option<&HistoricMapView<'a, 'h, V>> {
        if let HistoricView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a map from this view.
    pub fn into_map(self) -> Option<HistoricMapView<'a, 'h, V>> {
        if let HistoricView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn list(&self) -> Option<&HistoricListView<'a, 'h, V>> {
        if let HistoricView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn into_list(self) -> Option<HistoricListView<'a, 'h, V>> {
        if let HistoricView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn text(&self) -> Option<&HistoricTextView<'a, 'h, V>> {
        if let HistoricView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn into_text(self) -> Option<HistoricTextView<'a, 'h, V>> {
        if let HistoricView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract a scalar value from this view.
    pub fn scalar(&self) -> Option<ScalarValue> {
        if let HistoricView::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}
