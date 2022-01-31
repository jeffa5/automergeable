use std::borrow::Cow;

use automerge::{Automerge, ChangeHash, ObjId, ObjType, Value};

use super::{MapView, MutableMapView, MutableView, View};

#[derive(Debug, Clone)]
pub struct ListView<'a, 'h> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a Automerge,
    pub(crate) heads: Cow<'h, [ChangeHash]>,
}

impl<'a, 'h> PartialEq for ListView<'a, 'h> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, 'h> ListView<'a, 'h> {
    pub fn len(&self) -> usize {
        self.doc.length(&self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<View<'a, 'h>> {
        match self.doc.value(&self.obj, index) {
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
                Value::Object(ObjType::Text) => todo!(),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = View> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}

#[derive(Debug)]
pub struct MutableListView<'a> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a mut Automerge,
}

impl<'a> PartialEq for MutableListView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a> MutableListView<'a> {
    pub fn into_immutable(self) -> ListView<'a, 'static> {
        let heads = self.doc.get_heads();
        ListView {
            obj: self.obj,
            doc: self.doc,
            heads: Cow::Owned(heads),
        }
    }

    pub fn len(&self) -> usize {
        self.doc.length(&self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<View> {
        match self.doc.value(&self.obj, index) {
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
                Value::Object(ObjType::Text) => todo!(),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<MutableView> {
        match self.doc.value(&self.obj, index) {
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
                Value::Object(ObjType::Text) => todo!(),
                Value::Scalar(s) => Some(MutableView::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn insert<V: Into<Value>>(&mut self, index: usize, value: V) {
        self.doc.set(&self.obj, index, value).unwrap();
    }

    // TODO: change this to return the valueref that was removed, using the old heads, once
    // valueref can work in the past
    pub fn remove(&mut self, index: usize) -> bool {
        let exists = self.get(index).is_some();
        self.doc.del(&self.obj, index).unwrap();
        exists
    }

    pub fn iter(&self) -> impl Iterator<Item = View> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{automerge_doc, ScalarValue, ViewableDoc};

    #[test]
    fn test_list() {
        let mut doc = automerge_doc(json!({
            "a": [1, 2],
        }))
        .unwrap();

        let list = doc.view().get("a").unwrap().list().unwrap();

        assert_eq!(list.get(0), Some(View::Scalar(ScalarValue::Uint(1))));

        assert_eq!(list.len(), 2);

        assert!(!list.is_empty());

        assert_eq!(
            list.iter().collect::<Vec<_>>(),
            vec![1u64.into(), 2u64.into()]
        );
    }
}
