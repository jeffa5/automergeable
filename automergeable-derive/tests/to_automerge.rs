use std::collections::HashMap;

use automergeable_derive::{Automergeable, ToAutomerge};
use automergeable_traits::ToAutomerge;
use insta::{assert_json_snapshot, Settings};

#[test]
fn to_automerge() {
    #[derive(ToAutomerge, Debug, Default)]
    struct A {
        list: Vec<String>,
        others: HashMap<String, String>,
        nah: Option<u64>,
        yep: Option<i64>,
        b: B,
    }

    #[derive(Automergeable, Debug, Default)]
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
                "inner": {
                  "Uint": 2
                }
              },
              "map"
            ],
            "list": [],
            "nah": "Null",
            "others": [
              {
                "a": {
                  "Str": "b"
                }
              },
              "map"
            ],
            "yep": {
              "Int": -234
            }
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
                "inner": {
                  "Uint": 3
                }
              },
              "map"
            ],
            "list": [],
            "nah": "Null",
            "others": [
              {
                "a": {
                  "Str": "b"
                },
                "c": {
                  "Str": "c"
                }
              },
              "map"
            ],
            "yep": "Null"
          },
          "map"
        ]
        "###)
    });
}

#[test]
fn to_automerge_attribute() {
    #[derive(ToAutomerge, Debug, Default)]
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

    #[derive(Automergeable, Debug, Default)]
    struct B {
        inner: u64,
    }

    #[derive(ToAutomerge, Debug, Default)]
    struct Tuple(#[automergeable(representation = "Text")] String, Vec<char>);

    #[derive(ToAutomerge, Debug, Default)]
    struct Unit;

    #[derive(ToAutomerge, Debug)]
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
    settings.bind(|| assert_json_snapshot!(a.to_automerge(), @r###"
                     [
                       {
                         "a_counter": {
                           "Counter": 0
                         },
                         "a_timestamp": {
                           "Timestamp": 0
                         },
                         "b": [
                           {
                             "inner": {
                               "Uint": 2
                             }
                           },
                           "map"
                         ],
                         "en": [
                           {
                             "Part2": "Null"
                           },
                           "map"
                         ],
                         "list": [],
                         "nah": "Null",
                         "others": [
                           {},
                           "map"
                         ],
                         "some_text": [],
                         "tup": [
                           [
                             "a",
                             " ",
                             "t",
                             "u",
                             "p",
                             "l",
                             "e"
                           ],
                           [
                             "h",
                             "i"
                           ]
                         ],
                         "u": "Null",
                         "yep": {
                           "Int": -234
                         }
                       },
                       "map"
                     ]
                     "###));

    a.others.insert("a".to_owned(), "b".to_owned());
    a.some_text.push_str("hi");
    a.en = En::Part1(String::new(), 42);
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
        [
          {
            "a_counter": {
              "Counter": 0
            },
            "a_timestamp": {
              "Timestamp": 0
            },
            "b": [
              {
                "inner": {
                  "Uint": 2
                }
              },
              "map"
            ],
            "en": [
              {
                "Part1": [
                  [],
                  {
                    "Int": 42
                  }
                ]
              },
              "map"
            ],
            "list": [],
            "nah": "Null",
            "others": [
              {
                "a": {
                  "Str": "b"
                }
              },
              "map"
            ],
            "some_text": [
              "h",
              "i"
            ],
            "tup": [
              [
                "a",
                " ",
                "t",
                "u",
                "p",
                "l",
                "e"
              ],
              [
                "h",
                "i"
              ]
            ],
            "u": "Null",
            "yep": {
              "Int": -234
            }
          },
          "map"
        ]
        "###)
    });

    a.others.insert("c".to_owned(), "c".to_owned());
    a.some_text.push_str(" world");
    a.yep = None;
    a.b.inner += 1;
    a.en = En::Part3 { a: String::new() };
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
        [
          {
            "a_counter": {
              "Counter": 0
            },
            "a_timestamp": {
              "Timestamp": 0
            },
            "b": [
              {
                "inner": {
                  "Uint": 3
                }
              },
              "map"
            ],
            "en": [
              {
                "Part3": [
                  {
                    "a": {
                      "Str": ""
                    }
                  },
                  "map"
                ]
              },
              "map"
            ],
            "list": [],
            "nah": "Null",
            "others": [
              {
                "a": {
                  "Str": "b"
                },
                "c": {
                  "Str": "c"
                }
              },
              "map"
            ],
            "some_text": [
              "h",
              "i",
              " ",
              "w",
              "o",
              "r",
              "l",
              "d"
            ],
            "tup": [
              [
                "a",
                " ",
                "t",
                "u",
                "p",
                "l",
                "e"
              ],
              [
                "h",
                "i"
              ]
            ],
            "u": "Null",
            "yep": "Null"
          },
          "map"
        ]
        "###)
    });
}
