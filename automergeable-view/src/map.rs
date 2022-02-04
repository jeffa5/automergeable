use std::borrow::Cow;

use automerge::{Automerge, ChangeHash, ObjId, ObjType, Value};

use super::{list::MutableListView, ListView, MutableView, View};
use crate::{MutableTextView, TextView};

#[derive(Debug)]
pub struct MapView<'a, 'h> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a Automerge,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h> PartialEq for MapView<'a, 'h> {
    fn eq(&self, other: &Self) -> bool {
        if self.obj == other.obj && self.len() == other.len() {
            let mut our_keys = self.iter().collect::<Vec<_>>();
            our_keys.sort_by_key(|(key, _)| key.clone());
            let mut other_keys = other.iter().collect::<Vec<_>>();
            other_keys.sort_by_key(|(key, _)| key.clone());
            our_keys.into_iter().zip(other_keys).all(|(a, b)| a == b)
        } else {
            false
        }
    }
}

impl<'a, 'h> MapView<'a, 'h> {
    pub fn len(&self) -> usize {
        self.doc.keys_at(&self.obj, &self.heads).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<S: Into<String>>(&self, key: S) -> Option<View<'a, 'h>> {
        match self.doc.value_at(&self.obj, key.into(), &self.heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.doc,
                    heads: self.heads.clone(),
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn contains_key<S: Into<String>>(&self, key: S) -> bool {
        self.get(key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.doc.keys_at(&self.obj, &self.heads).into_iter()
    }

    pub fn values(&self) -> impl Iterator<Item = View> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, View)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}

// MapRefMut isn't allowed to travel to the past as it can't be mutated.
#[derive(Debug)]
pub struct MutableMapView<'a> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a mut Automerge,
}

impl<'a> PartialEq for MutableMapView<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.obj == other.obj && self.len() == other.len() {
            let mut our_keys = self.iter().collect::<Vec<_>>();
            our_keys.sort_by_key(|(key, _)| key.clone());
            let mut other_keys = other.iter().collect::<Vec<_>>();
            other_keys.sort_by_key(|(key, _)| key.clone());
            our_keys.into_iter().zip(other_keys).all(|(a, b)| a == b)
        } else {
            false
        }
    }
}

impl<'a> MutableMapView<'a> {
    pub fn into_immutable(self) -> MapView<'a, 'static> {
        let heads = self.doc.get_heads();
        MapView {
            obj: self.obj,
            doc: self.doc,
            heads: Cow::Owned(heads),
        }
    }

    pub fn len(&self) -> usize {
        self.doc.keys(&self.obj).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<S: Into<String>>(&self, key: S) -> Option<View> {
        match self.doc.value(&self.obj, key.into()) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Borrowed(&[]),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Borrowed(&[]),
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Borrowed(&[]),
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    fn get_at<S: Into<String>>(&self, key: S, heads: Vec<ChangeHash>) -> Option<View> {
        match self.doc.value_at(&self.obj, key.into(), &heads) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Owned(heads),
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.doc,
                    heads: Cow::Owned(heads),
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut<S: Into<String>>(&mut self, key: S) -> Option<MutableView> {
        match self.doc.value(&self.obj, key.into()) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(MutableView::Map(MutableMapView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(MutableView::List(MutableListView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Text) => Some(MutableView::Text(MutableTextView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Scalar(s) => Some(MutableView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<S: Into<String>, V: Into<Value>>(&mut self, key: S, value: V) {
        self.doc.set(&self.obj, key.into(), value).unwrap();
    }

    /// Remove a value from this map, returning a view of the removed value.
    pub fn remove<S: Into<String>>(&mut self, key: S) -> Option<View> {
        let heads = self.doc.get_heads();
        let key = key.into();
        if self.get(key.clone()).is_some() {
            self.doc.del(&self.obj, key.clone()).unwrap();
            Some(
                self.get_at(key, heads)
                    .expect("Deleted value isn't in the history"),
            )
        } else {
            None
        }
    }

    pub fn contains_key<S: Into<String>>(&self, key: S) -> bool {
        self.get(key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.doc.keys(&self.obj).into_iter()
    }

    pub fn values(&self) -> impl Iterator<Item = View> {
        self.keys().map(move |key| self.get(key).unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, View)> {
        self.keys().map(move |key| {
            let v = self.get(&key).unwrap();
            (key, v)
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{automerge_doc, ScalarValue, ViewableDoc};

    #[test]
    fn test_map() {
        let mut doc = automerge_doc(json!({
            "a": 1,
            "b": 2,
        }))
        .unwrap();

        let root = doc.view();

        assert!(matches!(
            root.get("a"),
            Some(View::Scalar(ScalarValue::Uint(1)))
        ));

        assert!(matches!(
            root.get("b"),
            Some(View::Scalar(ScalarValue::Uint(2)))
        ));

        assert_eq!(root.len(), 2);

        assert!(!root.is_empty());

        assert!(root.contains_key("a"));

        assert!(!root.contains_key("c"));

        assert_eq!(root.keys().collect::<Vec<String>>(), vec!["a", "b"]);

        assert_eq!(
            root.values().collect::<Vec<_>>(),
            vec![1u64.into(), 2u64.into()]
        );

        assert_eq!(
            root.iter().collect::<Vec<_>>(),
            vec![("a".to_owned(), 1u64.into()), ("b".to_owned(), 2u64.into())]
        );
    }

    #[test]
    fn test_map_mut() {
        let mut doc = automerge_doc(json!({
            "a": 1,
            "b": 2,
        }))
        .unwrap();

        let mut root = doc.view_mut();

        assert!(matches!(
            root.get("a"),
            Some(View::Scalar(ScalarValue::Uint(1)))
        ));

        assert!(matches!(
            root.get("b"),
            Some(View::Scalar(ScalarValue::Uint(2)))
        ));

        assert_eq!(root.len(), 2);

        assert!(!root.is_empty());

        assert!(root.contains_key("a"));

        assert!(!root.contains_key("c"));

        assert_eq!(root.keys().collect::<Vec<String>>(), vec!["a", "b"]);

        assert_eq!(
            root.values().collect::<Vec<_>>(),
            vec![1u64.into(), 2u64.into()]
        );

        assert_eq!(
            root.iter().collect::<Vec<_>>(),
            vec![("a".to_owned(), 1u64.into()), ("b".to_owned(), 2u64.into())]
        );

        root.insert("c", 5);

        assert_eq!(root.len(), 3);
        assert!(root.contains_key("c"));

        assert!(root.remove("a").is_some());
        assert!(root.remove("a").is_none());
        assert_eq!(root.len(), 2);

        let imm = root.into_immutable();
        assert!(imm.contains_key("c"));
    }

    #[test]
    fn nested_map() {
        let mut doc = Automerge::new();
        let mut root = doc.view_mut();

        root.insert("a", Value::map());
        let mut a = root.get_mut("a").unwrap();
        let a_map = a.map_mut().unwrap();
        a_map.insert("b", 1);

        assert!(a_map.contains_key("b"));
    }
}
