use std::collections::HashMap;

use automergeable::{
    traits::{FromAutomerge, ToAutomerge},
    Automergeable, FromAutomerge, ToAutomerge,
};
use insta::{assert_json_snapshot, Settings};
use serde::Serialize;

#[test]
fn from_automerge() {
    #[derive(ToAutomerge, FromAutomerge, Debug, Default, Serialize)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
        nah: Option<u64>,
        yep: Option<i64>,
        b: B,
    }

    #[derive(Automergeable, Debug, Default, Serialize)]
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
    #[derive(Automergeable, Debug, Default, Serialize)]
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
        tup: Tuple,
        en: En,
        u: Unit,
    }

    #[derive(Automergeable, Debug, Default, Serialize)]
    struct B {
        inner: u64,
    }

    #[derive(Automergeable, Debug, Default, Serialize)]
    struct Tuple(#[automergeable(representation = "Text")] String, Vec<char>);

    #[derive(Automergeable, Debug, Default, Serialize)]
    struct Unit;

    #[derive(Automergeable, Debug, Serialize)]
    enum En {
        Part1(#[automergeable(representation = "Text")] String, i64),
        Part2,
        Part3 { a: String },
    }

    impl Default for En {
        fn default() -> Self {
            Self::Part2
        }
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
        tup: Tuple("a tuple".to_owned(), vec!['h', 'i']),
        en: En::default(),
        u: Unit,
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
          },
          "tup": [
            "a tuple",
            [
              "h",
              "i"
            ]
          ],
          "en": "Part2",
          "u": null
        }
        "###);
    });

    a.others.insert("a".to_owned(), "b".to_owned());
    a.some_text.push_str("hi");
    a.a_counter += 2;
    a.en = En::Part1(String::new(), 42);
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
          },
          "tup": [
            "a tuple",
            [
              "h",
              "i"
            ]
          ],
          "en": {
            "Part1": [
              "",
              42
            ]
          },
          "u": null
        }
        "###);
    });

    a.others.insert("c".to_owned(), "c".to_owned());
    a.some_text.push_str(" world");
    a.yep = None;
    a.b.inner += 1;
    a.a_timestamp += 60;
    a.en = En::Part3 { a: String::new() };
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
          },
          "tup": [
            "a tuple",
            [
              "h",
              "i"
            ]
          ],
          "en": {
            "Part3": {
              "a": ""
            }
          },
          "u": null
        }
        "###);
    });
}
