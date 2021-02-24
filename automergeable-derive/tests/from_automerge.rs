use std::collections::HashMap;

use automergeable_derive::{Automergeable, FromAutomerge, ToAutomerge};
use automergeable_traits::{FromAutomerge, ToAutomerge};
use insta::{assert_json_snapshot, Settings};
use serde::Serialize;

#[test]
fn from_automerge() {
    #[derive(ToAutomerge, FromAutomerge, Debug, Serialize)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
        nah: Option<u64>,
        yep: Option<i64>,
        b: B,
    }

    #[derive(Automergeable, Debug, Serialize)]
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

    let mut settings = Settings::new();
    settings.set_sort_maps(true);
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
    {
      "list": [],
      "others": {},
      "nah": null,
      "yep": -234,
      "b": {
        "inner": 2
      }
    }
    "###);
    });

    a.others.insert("a".to_owned(), "b".to_owned());
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
    {
      "list": [],
      "others": {
        "a": "b"
      },
      "nah": null,
      "yep": -234,
      "b": {
        "inner": 2
      }
    }
    "###);
    });

    a.others.insert("c".to_owned(), "c".to_owned());
    a.yep = None;
    a.b.inner += 1;
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
        {
          "list": [],
          "others": {
            "a": "b",
            "c": "c"
          },
          "nah": null,
          "yep": null,
          "b": {
            "inner": 3
          }
        }
        "###);
    })
}

#[test]
fn from_automerge_attribute() {
    #[derive(Automergeable, Debug, Serialize)]
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

    #[derive(Automergeable, Debug, Serialize)]
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
    let mut settings = Settings::new();
    settings.set_sort_maps(true);
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
        {
          "list": [],
          "others": {},
          "nah": null,
          "yep": -234,
          "some_text": "",
          "a_counter": 0,
          "a_timestamp": 0,
          "b": {
            "inner": 2
          }
        }
        "###);
    });

    a.others.insert("a".to_owned(), "b".to_owned());
    a.some_text.push_str("hi");
    a.a_counter += 2;
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
        {
          "list": [],
          "others": {
            "a": "b"
          },
          "nah": null,
          "yep": -234,
          "some_text": "hi",
          "a_counter": 2,
          "a_timestamp": 0,
          "b": {
            "inner": 2
          }
        }
        "###);
    });

    a.others.insert("c".to_owned(), "c".to_owned());
    a.some_text.push_str(" world");
    a.yep = None;
    a.b.inner += 1;
    a.a_timestamp += 60;
    settings.bind(|| {
        assert_json_snapshot!(A::from_automerge(&a.to_automerge()).ok(), @r###"
        {
          "list": [],
          "others": {
            "a": "b",
            "c": "c"
          },
          "nah": null,
          "yep": null,
          "some_text": "hi world",
          "a_counter": 2,
          "a_timestamp": 60,
          "b": {
            "inner": 3
          }
        }
        "###);
    });
}
