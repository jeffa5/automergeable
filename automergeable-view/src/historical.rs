use automerge::{Prop, ScalarValue};

use crate::{list::HistoricalListView, map::HistoricalMapView, text::HistoricalTextView, Viewable};

/// A view over a document, providing a lightweight way to traverse it.
#[derive(Debug)]
pub enum HistoricalView<'a, 'h, V> {
    Map(HistoricalMapView<'a, 'h, V>),
    List(HistoricalListView<'a, 'h, V>),
    Text(HistoricalTextView<'a, 'h, V>),
    Scalar(ScalarValue),
}

impl<'a, 'h, 'oa, 'oh, V, OV> PartialEq<HistoricalView<'oa, 'oh, OV>> for HistoricalView<'a, 'h, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &HistoricalView<'oa, 'oh, OV>) -> bool {
        match (self, other) {
            (Self::Map(l0), HistoricalView::Map(r0)) => l0 == r0,
            (Self::List(l0), HistoricalView::List(r0)) => l0 == r0,
            (Self::Text(l0), HistoricalView::Text(r0)) => l0 == r0,
            (Self::Scalar(l0), HistoricalView::Scalar(r0)) => l0 == r0,
            (Self::Map(_), HistoricalView::List(_))
            | (Self::Map(_), HistoricalView::Text(_))
            | (Self::Map(_), HistoricalView::Scalar(_))
            | (Self::List(_), HistoricalView::Map(_))
            | (Self::List(_), HistoricalView::Text(_))
            | (Self::List(_), HistoricalView::Scalar(_))
            | (Self::Text(_), HistoricalView::Map(_))
            | (Self::Text(_), HistoricalView::List(_))
            | (Self::Text(_), HistoricalView::Scalar(_))
            | (Self::Scalar(_), HistoricalView::Map(_))
            | (Self::Scalar(_), HistoricalView::List(_))
            | (Self::Scalar(_), HistoricalView::Text(_)) => false,
        }
    }
}

impl<'a, 'h, V> HistoricalView<'a, 'h, V>
where
    V: Viewable,
{
    /// Get the view of the object at `prop`.
    pub fn get<P: Into<Prop>>(&self, prop: P) -> Option<HistoricalView<'a, 'h, V>> {
        match (prop.into(), self) {
            (Prop::Map(key), HistoricalView::Map(map)) => map.get(key),
            (Prop::Seq(index), HistoricalView::List(l)) => l.get(index),
            (Prop::Seq(index), HistoricalView::Text(t)) => t
                .get(index)
                .map(|s| HistoricalView::Scalar(ScalarValue::Str(s))),
            (Prop::Seq(_), HistoricalView::Map(_))
            | (Prop::Map(_), HistoricalView::List(_))
            | (Prop::Map(_), HistoricalView::Text(_))
            | (_, HistoricalView::Scalar(_)) => None,
        }
    }

    /// Get the length of this object.
    pub fn len(&self) -> usize {
        match self {
            HistoricalView::Map(map) => map.len(),
            HistoricalView::List(list) => list.len(),
            HistoricalView::Text(text) => text.len(),
            HistoricalView::Scalar(_) => 0,
        }
    }

    /// Check if this object is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Try and extract a map from this view.
    pub fn map(&self) -> Option<&HistoricalMapView<'a, 'h, V>> {
        if let HistoricalView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a map from this view.
    pub fn into_map(self) -> Option<HistoricalMapView<'a, 'h, V>> {
        if let HistoricalView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn list(&self) -> Option<&HistoricalListView<'a, 'h, V>> {
        if let HistoricalView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract a list from this view.
    pub fn into_list(self) -> Option<HistoricalListView<'a, 'h, V>> {
        if let HistoricalView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn text(&self) -> Option<&HistoricalTextView<'a, 'h, V>> {
        if let HistoricalView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract text from this view.
    pub fn into_text(self) -> Option<HistoricalTextView<'a, 'h, V>> {
        if let HistoricalView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Try and extract a scalar value from this view.
    pub fn scalar(&self) -> Option<ScalarValue> {
        if let HistoricalView::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}
