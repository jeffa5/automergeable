use std::collections::HashMap;

use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};
use automergeable_traits::{FromAutomerge, ToAutomerge};
use insta::{assert_debug_snapshot, assert_json_snapshot, Settings};

#[test]
fn from_automerge() {
    #[derive(ToAutomerge, FromAutomerge, Debug)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
        nah: Option<u64>,
        yep: Option<i64>,
        b: B,
    }

    #[derive(Automergeable, Debug)]
    struct B {
        inner: u64,
    }

    let mut a = A {
        list: Vec::new(),
        others: HashMap::new(),
        nah: None,
        yep: Some(-234),
        b: B { inner: 2 },
    };

    a.others.insert("a".to_owned(), "b".to_owned());
    assert_debug_snapshot!(A::from_automerge(&a.to_automerge()), @"");

    a.others.insert("c".to_owned(), "c".to_owned());
    a.yep = None;
    a.b.inner += 1;
    assert_debug_snapshot!(A::from_automerge(&a.to_automerge()), @"");
}

#[test]
fn from_automerge_attribute() {
    #[derive(Automergeable, Debug)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
        nah: Option<u64>,
        yep: Option<i64>,
        #[automergeable(representation = "Text")]
        some_text: String,
        #[automergeable(representation = "Counter")]
        a_counter: i64,
        #[automergeable(representation = "Timestamp")]
        a_timestamp: i64,
        b: B,
    }

    #[derive(Automergeable, Debug)]
    struct B {
        inner: u64,
    }

    let mut a = A {
        list: Vec::new(),
        others: HashMap::new(),
        nah: None,
        yep: Some(-234),
        some_text: String::new(),
        a_counter: 0,
        a_timestamp: 0,
        b: B { inner: 2 },
    };

    a.others.insert("a".to_owned(), "b".to_owned());
    a.some_text.push_str("hi");
    assert_debug_snapshot!(A::from_automerge(&a.to_automerge()), @"");

    a.others.insert("c".to_owned(), "c".to_owned());
    a.some_text.push_str(" world");
    a.yep = None;
    a.b.inner += 1;
    assert_debug_snapshot!(A::from_automerge(&a.to_automerge()), @"");
}
