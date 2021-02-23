use std::collections::HashMap;

use automergable_derive::Automergable;
use automerge::Path;

#[test]
fn simple_from_value() {
    #[derive(Automergable)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
    }

    // assert_eq!(vec![], Vec::from(automerge::Value::Sequence));
}
