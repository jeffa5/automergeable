use std::borrow::Cow;

use automerge::{transaction::Transaction, Automerge, ChangeHash, ScalarValue, ROOT};

mod historic;
mod immutable;
mod list;
mod map;
mod mutable;
mod text;
mod viewable;

pub use historic::HistoricView;
pub use immutable::View;
pub use list::{ListView, MutableListView};
use map::HistoricMapView;
pub use map::{MapView, MutableMapView};
pub use mutable::MutableView;
pub use text::{MutableTextView, TextView};
pub use viewable::Viewable;

/// A document that can be viewed, both immutably and mutably.
pub trait ViewableDoc<V> {
    /// Create a new view over this document.
    fn view(&self) -> MapView<'_, V>;

    /// Create a new view over this document at historical point `heads`.
    fn view_at<'a, 'h>(&'a self, heads: &'h [ChangeHash]) -> HistoricMapView<'a, 'h, V>;
}

pub trait MutableDoc<'a> {
    /// Create a new mutable view over this document.
    fn view_mut<'t>(&'t mut self) -> MutableMapView<'a, 't>;
}

impl ViewableDoc<Automerge> for Automerge {
    fn view(&self) -> MapView<'_, Automerge> {
        MapView {
            obj: ROOT,
            doc: self,
        }
    }

    fn view_at<'h>(&self, heads: &'h [ChangeHash]) -> HistoricMapView<'_, 'h, Automerge> {
        HistoricMapView {
            obj: ROOT,
            doc: self,
            heads: Cow::Borrowed(heads),
        }
    }
}

impl<'a> MutableDoc<'a> for Transaction<'a> {
    fn view_mut<'t>(&'t mut self) -> MutableMapView<'a, 't> {
        MutableMapView {
            obj: ROOT,
            tx: self,
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

#[cfg(test)]
fn automerge_doc(value: serde_json::Value) -> Result<Automerge, String> {
    use automerge::transaction::Transactable;
    use automerge::{ObjId, Value};
    use serde_json::Map;
    fn add_map(map: Map<String, serde_json::Value>, doc: &mut Transaction, obj: ObjId) {
        for (k, v) in map.into_iter() {
            match v {
                serde_json::Value::Null => {
                    doc.set(&obj.clone(), k, ScalarValue::Null).unwrap();
                }
                serde_json::Value::Bool(b) => {
                    doc.set(&obj.clone(), k, b).unwrap();
                }
                serde_json::Value::Number(n) => {
                    doc.set(&obj.clone(), k, n.as_u64().unwrap())
                        .expect("no error");
                }
                serde_json::Value::String(s) => {
                    doc.set(&obj.clone(), k, s.to_owned()).unwrap().unwrap();
                }
                serde_json::Value::Array(a) => {
                    let obj = doc.set(&obj.clone(), k, Value::list()).unwrap().unwrap();
                    add_list(a, doc, obj)
                }
                serde_json::Value::Object(map) => {
                    let obj = doc.set(&obj.clone(), k, Value::map()).unwrap().unwrap();
                    add_map(map, doc, obj)
                }
            };
        }
    }

    fn add_list(list: Vec<serde_json::Value>, doc: &mut Transaction, obj: ObjId) {
        for (i, v) in list.into_iter().enumerate() {
            match v {
                serde_json::Value::Null => {
                    doc.set(&obj, i, ScalarValue::Null).unwrap();
                }
                serde_json::Value::Bool(b) => {
                    doc.set(&obj, i, b).unwrap();
                }
                serde_json::Value::Number(n) => {
                    doc.insert(&obj, i, n.as_u64().unwrap()).expect("no error");
                }
                serde_json::Value::String(s) => {
                    doc.set(&obj, i, s.to_owned()).unwrap().unwrap();
                }
                serde_json::Value::Array(a) => {
                    let obj = doc.set(&obj, i, Value::list()).unwrap().unwrap();
                    add_list(a, doc, obj)
                }
                serde_json::Value::Object(map) => {
                    let obj = doc.set(&obj, i, Value::map()).unwrap().unwrap();
                    add_map(map, doc, obj)
                }
            };
        }
    }

    if let serde_json::Value::Object(o) = value {
        let mut doc = Automerge::new();
        let mut tx = doc.transaction();
        add_map(o, &mut tx, ROOT);
        tx.commit();
        Ok(doc)
    } else {
        Err("wasn't an object".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use crate::MutableDoc;
    use automerge::{Automerge, Value};
    use serde_json::json;

    use super::*;

    #[test]
    fn get_map_key() {
        let doc = automerge_doc(json!({"a": 1})).unwrap();

        let a_val = doc.view().get("a");
        assert!(matches!(a_val, Some(View::Scalar(ScalarValue::Uint(1)))));
    }

    #[test]
    fn get_nested_map() {
        let doc = automerge_doc(json!({"a": {"b": 1}})).unwrap();

        let b_val = doc.view().get("a").unwrap().get("b");

        assert!(matches!(b_val, Some(View::Scalar(ScalarValue::Uint(1)))));
    }

    #[test]
    fn set_nested_map() {
        let mut doc = Automerge::new();
        let mut tx = doc.transaction();
        let mut root = tx.view_mut();
        root.insert("a", Value::map());
        let mut a = root.get_mut("a").unwrap();
        a.insert("b", 1);

        assert_eq!(a.get("b"), Some(1.into()));
    }
}
