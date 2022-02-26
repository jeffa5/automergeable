use std::borrow::Cow;

use automerge::{Automerge, ChangeHash, ROOT};

use crate::{HistoricalMapView, MapView};

/// A document that can be viewed, both immutably and mutably.
pub trait ViewableDoc<V> {
    /// Create a new view over this document.
    fn view(&self) -> MapView<'_, V>;

    /// Create a new view over this document at historical point `heads`.
    fn view_at<'a, 'h>(&'a self, heads: &'h [ChangeHash]) -> HistoricalMapView<'a, 'h, V>;
}

impl ViewableDoc<Automerge> for Automerge {
    fn view(&self) -> MapView<'_, Automerge> {
        MapView {
            obj: ROOT,
            doc: self,
        }
    }

    fn view_at<'h>(&self, heads: &'h [ChangeHash]) -> HistoricalMapView<'_, 'h, Automerge> {
        HistoricalMapView {
            obj: ROOT,
            doc: self,
            heads: Cow::Borrowed(heads),
        }
    }
}
