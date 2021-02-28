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
    }

    #[derive(Automergeable, Debug, Default)]
    struct B {
        inner: u64,
    }

    #[derive(ToAutomerge, Debug, Default)]
    struct Tuple(#[automergeable(representation = "Text")] String, Vec<char>);

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
    };

    a.others.insert("a".to_owned(), "b".to_owned());
    a.some_text.push_str("hi");
    let mut settings = Settings::new();
    settings.set_sort_maps(true);
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
        [
          {
            "a_counter": 0,
            "a_timestamp": 0,
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
            "some_text": [
              "h",
              "i"
            ],
            "tup": [
              {
                "Tuple": [
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
                ]
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
    a.some_text.push_str(" world");
    a.yep = None;
    a.b.inner += 1;
    settings.bind(|| {
        assert_json_snapshot!(a.to_automerge(), @r###"
        [
          {
            "a_counter": 0,
            "a_timestamp": 0,
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
              {
                "Tuple": [
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
                ]
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
