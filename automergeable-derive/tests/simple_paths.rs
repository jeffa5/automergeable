use std::collections::HashMap;

use automergeable_derive::Automergeable;
use automerge::Path;

#[test]
fn simple_paths() {
    #[derive(Automergeable)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
    }

    assert_eq!(Path::root().key("list"), A::list_path());
    assert_eq!(Path::root().key("others"), A::others_path());
}
