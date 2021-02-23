use std::collections::HashMap;

use automergable_derive::Automergable;

#[test]
fn simple_diff() {
    #[derive(Automergable, Default)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
    }

    assert_eq!(
        <Vec<automerge::LocalChange>>::new(),
        A::default().diff(&A::default())
    );
}
