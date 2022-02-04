use std::borrow::Cow;

use automerge::{Automerge, ChangeHash, Prop, ScalarValue, Value, ROOT};

mod list;
mod map;
mod text;

pub use list::{ListView, MutableListView};
pub use map::{MapView, MutableMapView};
pub use text::{MutableTextView, TextView};

/// A document that can be viewed, both immutably and mutably.
pub trait ViewableDoc {
    /// Create a new view over this document.
    fn view(&mut self) -> MapView;

    /// Create a new view over this document at historical point `heads`.
    fn view_at<'a, 'h>(&'a mut self, heads: &'h [ChangeHash]) -> MapView<'a, 'h>;

    /// Create a new mutable view over this document.
    fn view_mut(&mut self) -> MutableMapView;
}

impl ViewableDoc for Automerge {
    fn view(&mut self) -> MapView {
        let heads = self.get_heads();
        MapView {
            obj: ROOT,
            doc: self,
            heads: Cow::Owned(heads),
        }
    }

    fn view_at<'a>(&mut self, heads: &'a [ChangeHash]) -> MapView<'_, 'a> {
        MapView {
            obj: ROOT,
            doc: self,
            heads: Cow::Borrowed(heads),
        }
    }

    fn view_mut(&mut self) -> MutableMapView {
        MutableMapView {
            obj: ROOT,
            doc: self,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum View<'a, 'h> {
    Map(MapView<'a, 'h>),
    List(ListView<'a, 'h>),
    Text(TextView<'a, 'h>),
    Scalar(ScalarValue),
}

impl<'a, 'h> View<'a, 'h> {
    pub fn get<P: Into<Prop>>(&self, prop: P) -> Option<View<'a, 'h>> {
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

    pub fn len(&self) -> usize {
        match self {
            View::Map(map) => map.len(),
            View::List(list) => list.len(),
            View::Text(text) => text.len(),
            View::Scalar(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn map(&mut self) -> Option<&mut MapView<'a, 'h>> {
        if let View::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn list(&self) -> Option<ListView<'a, 'h>> {
        if let View::List(list) = self {
            Some(list.clone())
        } else {
            None
        }
    }

    pub fn text(&self) -> Option<TextView<'a, 'h>> {
        if let View::Text(text) = self {
            Some(text.clone())
        } else {
            None
        }
    }

    pub fn scalar(&self) -> Option<ScalarValue> {
        if let View::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MutableView<'a> {
    Map(MutableMapView<'a>),
    List(MutableListView<'a>),
    Text(MutableTextView<'a>),
    Scalar(ScalarValue),
}

impl<'a> MutableView<'a> {
    pub fn into_immutable(self) -> View<'a, 'static> {
        match self {
            MutableView::Map(map) => View::Map(map.into_immutable()),
            MutableView::List(list) => View::List(list.into_immutable()),
            MutableView::Text(text) => View::Text(text.into_immutable()),
            MutableView::Scalar(scalar) => View::Scalar(scalar),
        }
    }

    pub fn get<P: Into<Prop>>(&self, prop: P) -> Option<View> {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.get(key),
            (Prop::Seq(index), MutableView::List(l)) => l.get(index),
            (Prop::Seq(index), MutableView::Text(t)) => {
                t.get(index).map(|s| View::Scalar(ScalarValue::Str(s)))
            }
            (Prop::Seq(_), MutableView::Map(_))
            | (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    pub fn get_mut<P: Into<Prop>>(&mut self, prop: P) -> Option<MutableView> {
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

    pub fn insert<P: Into<Prop>, V: Into<Value>>(&mut self, prop: P, value: V) {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.insert(key, value),
            (Prop::Seq(index), MutableView::List(list)) => list.insert(index, value),
            (Prop::Seq(index), MutableView::Text(text)) => text.insert(index, value),
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => {}
        }
    }

    pub fn set<P: Into<Prop>, V: Into<Value>>(&mut self, prop: P, value: V) {
        match (prop.into(), self) {
            // map's insert does the same as a set would
            (Prop::Map(key), MutableView::Map(map)) => map.insert(key, value),
            (Prop::Seq(index), MutableView::List(list)) => list.set(index, value),
            (Prop::Seq(index), MutableView::Text(text)) => text.set(index, value),
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => {}
        }
    }

    pub fn remove<P: Into<Prop>>(&mut self, prop: P) -> Option<View> {
        match (prop.into(), self) {
            (Prop::Map(key), MutableView::Map(map)) => map.remove(key),
            (Prop::Seq(index), MutableView::List(list)) => list.remove(index),
            (Prop::Seq(index), MutableView::Text(text)) => text
                .remove(index)
                .map(|s| View::Scalar(ScalarValue::Str(s))),
            (Prop::Map(_), MutableView::List(_))
            | (Prop::Map(_), MutableView::Text(_))
            | (Prop::Seq(_), MutableView::Map(_))
            | (_, MutableView::Scalar(_)) => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            MutableView::Map(map) => map.len(),
            MutableView::List(list) => list.len(),
            MutableView::Text(text) => text.len(),
            MutableView::Scalar(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn map(&self) -> Option<MapView> {
        if let MutableView::Map(map) = self {
            Some(MapView {
                obj: map.obj.clone(),
                doc: map.doc,
                heads: Cow::Borrowed(&[]),
            })
        } else {
            None
        }
    }

    pub fn map_mut(&mut self) -> Option<&mut MutableMapView<'a>> {
        if let MutableView::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn list(&self) -> Option<ListView> {
        if let MutableView::List(list) = self {
            Some(ListView {
                obj: list.obj.clone(),
                doc: list.doc,
                heads: Cow::Borrowed(&[]),
            })
        } else {
            None
        }
    }

    pub fn list_mut(&mut self) -> Option<&mut MutableListView<'a>> {
        if let MutableView::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn text(&self) -> Option<TextView> {
        if let MutableView::Text(text) = self {
            Some(TextView {
                obj: text.obj.clone(),
                doc: text.doc,
                heads: Cow::Borrowed(&[]),
            })
        } else {
            None
        }
    }

    pub fn text_mut(&mut self) -> Option<&mut MutableTextView<'a>> {
        if let MutableView::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    pub fn scalar(&self) -> Option<ScalarValue> {
        if let MutableView::Scalar(scalar) = self {
            Some(scalar.clone())
        } else {
            None
        }
    }
}

impl From<u64> for View<'static, 'static> {
    fn from(u: u64) -> Self {
        View::Scalar(ScalarValue::Uint(u))
    }
}

impl From<i32> for View<'static, 'static> {
    fn from(i: i32) -> Self {
        View::Scalar(ScalarValue::Int(i as i64))
    }
}

#[cfg(test)]
fn automerge_doc(value: serde_json::Value) -> Result<Automerge, String> {
    use automerge::ObjId;
    use serde_json::Map;
    fn add_map(map: Map<String, serde_json::Value>, doc: &mut Automerge, obj: ObjId) {
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

    fn add_list(list: Vec<serde_json::Value>, doc: &mut Automerge, obj: ObjId) {
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
        add_map(o, &mut doc, ROOT);
        Ok(doc)
    } else {
        Err("wasn't an object".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use automerge::Automerge;
    use serde_json::json;

    use super::*;

    #[test]
    fn get_map_key() {
        let mut doc = automerge_doc(json!({"a": 1})).unwrap();

        let a_val = doc.view().get("a");
        assert!(matches!(a_val, Some(View::Scalar(ScalarValue::Uint(1)))));
    }

    #[test]
    fn get_nested_map() {
        let mut doc = automerge_doc(json!({"a": {"b": 1}})).unwrap();

        let b_val = doc.view().get("a").unwrap().get("b");

        assert!(matches!(b_val, Some(View::Scalar(ScalarValue::Uint(1)))));
    }

    #[test]
    fn set_nested_map() {
        let mut doc = Automerge::new();
        let mut root = doc.view_mut();
        root.insert("a", Value::map());
        let mut a = root.get_mut("a").unwrap();
        a.insert("b", 1);

        assert_eq!(a.get("b"), Some(1.into()));
    }
}
