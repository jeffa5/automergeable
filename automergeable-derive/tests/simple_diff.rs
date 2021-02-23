use std::collections::HashMap;
use automerge::Path;
use automerge::Value;
use automerge::LocalChange;
use automergeable_core::AutoDiff;

use automergeable_derive::Automergeable;
use pretty_assertions::assert_eq;

#[test]
fn simple_diff() {
    #[derive(Automergeable, Default, Clone)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
    }

    let original = A::default();
    let mut updated = A::default();
    assert_eq!(
        <Vec<automerge::LocalChange>>::new(),
        updated.diff(Path::root(),&original)
    );

    updated.list.push("1".to_owned());
    assert_eq!(
        vec![LocalChange::insert(A::list_path().index(0), Value::Primitive(automerge::ScalarValue::Str("1".to_string())))],
        updated.diff(Path::root(), &original)
    );
    let original = updated.clone();

    updated.list.push("2".to_owned());
    updated.others.insert("4".to_owned(), String::new());
    assert_eq!(
        vec![LocalChange::insert(A::list_path().index(1), Value::Primitive(automerge::ScalarValue::Str("2".to_string()))),
        LocalChange::insert(A::others_path().key("4"), Value::Primitive(automerge::ScalarValue::Str("".to_string())))],
        updated.diff(Path::root(), &original)
    );
}
