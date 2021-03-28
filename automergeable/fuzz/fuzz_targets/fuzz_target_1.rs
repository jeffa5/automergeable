#![no_main]
use std::collections::HashMap;

use automergeable::{
    automerge::{Backend, InvalidChangeRequest, MapType, Primitive, Value},
    DocumentChangeError,
};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Clone)]
struct Val(Value);

impl Default for Val {
    fn default() -> Self {
        Val(Value::Map(HashMap::new(), MapType::Map))
    }
}

impl automergeable::traits::ToAutomerge for Val {
    fn to_automerge(&self) -> Value {
        self.0.clone()
    }
}

impl automergeable::traits::FromAutomerge for Val {
    fn from_automerge(value: &Value) -> Result<Self, automergeable::traits::FromAutomergeError> {
        Ok(Self(value.clone()))
    }
}

fuzz_target!(|values: Vec<automergeable::automerge::Value>| {
    for val in &values {
        if let Value::Map(_, MapType::Map) = val {
        } else {
            return;
        }

        if has_cursor(val) {
            return;
        }
    }

    let mut doc: automergeable::Document<Val> = automergeable::Document::new();

    let mut backend_bytes = Vec::new();

    for val in values {
        let change = doc.change::<_, InvalidChangeRequest>(|old| {
            *old = Val(val);
            Ok(())
        });

        match change {
            Ok(c) => {
                if let Some(c) = c {
                    let mut backend = if backend_bytes.is_empty() {
                        Backend::init()
                    } else {
                        Backend::load(backend_bytes).unwrap()
                    };
                    let (_, _) = backend.apply_local_change(c).unwrap();
                    backend_bytes = backend.save().unwrap();
                }
            }
            Err(DocumentChangeError::InvalidChangeRequest(
                InvalidChangeRequest::InsertNonTextInTextObject { .. },
            )) => return,
            Err(DocumentChangeError::InvalidChangeRequest(
                InvalidChangeRequest::CannotOverwriteCounter { .. },
            )) => return,
            Err(e) => panic!("error from change {:?}", e),
        }
    }
});

fn has_cursor(v: &Value) -> bool {
    match v {
        Value::Map(m, _) => m.values().any(|v| has_cursor(v)),
        Value::Sequence(v) => v.iter().any(|i| has_cursor(i)),
        Value::Text(_) => false,
        Value::Primitive(p) => {
            if let Primitive::Cursor(_) = p {
                true
            } else {
                false
            }
        }
    }
}
