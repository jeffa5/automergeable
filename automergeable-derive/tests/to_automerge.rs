use std::collections::HashMap;

use automergeable_core::ToAutomerge;
use automergeable_derive::Automergeable;
use insta::{assert_json_snapshot, Settings};

#[test]
fn to_automerge() {
    #[derive(Automergeable, Debug)]
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
    let mut settings = Settings::new();
    settings.set_sort_maps(true);
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
         [
           {
             "b": [
               {
                 "inner": 2
               },
               "map"
             ],
             "list": [],
             "nah": null,
             "others": [
               {
                 "a": "b"
               },
               "map"
             ],
             "yep": -234
           },
           "map"
         ]
         "###)
    });

    a.others.insert("c".to_owned(), "c".to_owned());
    a.yep = None;
    a.b.inner += 1;
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
         [
           {
             "b": [
               {
                 "inner": 3
               },
               "map"
             ],
             "list": [],
             "nah": null,
             "others": [
               {
                 "a": "b",
                 "c": "c"
               },
               "map"
             ],
             "yep": null
           },
           "map"
         ]
         "###)
    });
}
