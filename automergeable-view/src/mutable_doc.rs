use automerge::{transaction::Transaction, ROOT};

use crate::MutableMapView;

pub trait MutableDoc<'a> {
    /// Create a new mutable view over this document.
    fn view_mut<'t>(&'t mut self) -> MutableMapView<'a, 't>;
}

impl<'a> MutableDoc<'a> for Transaction<'a> {
    fn view_mut<'t>(&'t mut self) -> MutableMapView<'a, 't> {
        MutableMapView {
            obj: ROOT,
            tx: self,
        }
    }
}
