use automerge::{ObjId, ObjType, Value};

use crate::{MapView, TextView, View, Viewable};

/// A view over a list in the document.
#[derive(Debug, Clone)]
pub struct ListView<'a, V> {
    pub(crate) obj: ObjId,
    pub(crate) doc: &'a V,
}

impl<'a, 'oa, V, OV> PartialEq<ListView<'oa, OV>> for ListView<'a, V>
where
    V: Viewable,
    OV: Viewable,
{
    fn eq(&self, other: &ListView<'oa, OV>) -> bool {
        self.obj == other.obj
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<'a, V> ListView<'a, V>
where
    V: Viewable,
{
    pub fn len(&self) -> usize {
        self.doc.length(&self.obj)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<View<'a, V>> {
        match self.doc.value(&self.obj, index) {
            Ok(Some((value, id))) => match value {
                Value::Object(ObjType::Map) => Some(View::Map(MapView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Table) => todo!(),
                Value::Object(ObjType::List) => Some(View::List(ListView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Object(ObjType::Text) => Some(View::Text(TextView {
                    obj: id,
                    doc: self.doc,
                })),
                Value::Scalar(s) => Some(View::Scalar(s)),
            },
            Ok(None) | Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = View<V>> {
        (0..self.len()).map(move |i| self.get(i).unwrap())
    }
}
