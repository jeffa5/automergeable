#![no_main]

use automerge::{Backend, InvalidChangeRequest, Primitive, Value};
use automergeable::{unicode_segmentation::UnicodeSegmentation, DocumentChangeError};
use libfuzzer_sys::fuzz_target;
use pretty_assertions::assert_eq;

fuzz_target!(|values: Vec<automerge::Value>| {
    for val in &values {
        // ensure the root is always a map
        if let Value::Map(_) = val {
        } else {
            return;
        }

        // don't work with cursors for now
        if has_cursor(val) {
            return;
        }

        // don't work with cursors for now
        if has_table(val) {
            return;
        }

        // don't allow empty text
        if has_empty_text(val) {
            return;
        }

        // nan aren't Eq so can't compare them with assert_eq
        if has_nan(val) {
            return;
        }
    }

    let mut doc = automergeable::Document::<Value, _>::new(automerge::Frontend::default());

    let mut backend_bytes = Vec::new();

    for val in values {
        let change = doc.change::<_, _, InvalidChangeRequest>(|old| {
            *old = val.clone();
            Ok(())
        });

        match change {
            Ok(c) => {
                if let ((), Some(c)) = c {
                    let mut backend = if backend_bytes.is_empty() {
                        Backend::new()
                    } else {
                        Backend::load(backend_bytes).unwrap()
                    };
                    let (patch, _) = backend.apply_local_change(c).unwrap();
                    assert_eq!(doc.get(), &val);
                    doc.apply_patch(patch).unwrap();

                    let doc_val = doc.get();
                    if doc_val != &val {
                        println!(
                            "changes: {:?}",
                            backend
                                .get_changes(&[])
                                .iter()
                                .map(|c| c.decode())
                                .collect::<Vec<_>>()
                        );
                        assert_eq!(doc_val, &val);
                    }
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
        Value::Map(m) | Value::Table(m) => m.values().any(|v| has_cursor(v)),
        Value::SortedMap(m) => m.values().any(|v| has_cursor(v)),
        Value::List(v) => v.iter().any(|i| has_cursor(i)),
        Value::Text(_) => false,
        Value::Primitive(p) => {
            matches!(p, Primitive::Cursor(_))
        }
    }
}

fn has_table(v: &Value) -> bool {
    match v {
        Value::Map(m) => m.values().any(|v| has_table(v)),
        Value::SortedMap(m) => m.values().any(|v| has_table(v)),
        Value::Table(_) => true,
        Value::List(v) => v.iter().any(|i| has_table(i)),
        Value::Text(_) | Value::Primitive(_) => false,
    }
}

fn has_empty_text(v: &Value) -> bool {
    match v {
        Value::Map(m) | Value::Table(m) => m.values().any(|v| has_empty_text(v)),
        Value::SortedMap(m) => m.values().any(|v| has_empty_text(v)),
        Value::List(v) => v.iter().any(|i| has_empty_text(i)),
        Value::Text(t) => t.iter().any(|i| i.graphemes(true).count() != 1),
        Value::Primitive(_) => false,
    }
}

fn has_nan(v: &Value) -> bool {
    match v {
        Value::Map(m) | Value::Table(m) => m.values().any(|v| has_nan(v)),
        Value::SortedMap(m) => m.values().any(|v| has_nan(v)),
        Value::List(v) => v.iter().any(|i| has_nan(i)),
        Value::Text(_) => false,
        Value::Primitive(p) => match p {
            Primitive::F64(f) => f.is_nan(),
            _ => false,
        },
    }
}
