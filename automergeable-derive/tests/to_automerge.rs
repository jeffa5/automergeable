use std::collections::HashMap;

use automerge::{MapType, Primitive, Value};
use automergeable::{Automergeable, ToAutomerge};
use insta::{assert_json_snapshot, Settings};
use maplit::hashmap;
use pretty_assertions::assert_eq;

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
            "en": {
              "Str": "Part2"
            },
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
                {
                  "Str": "h"
                },
                {
                  "Str": "i"
                }
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
                {
                  "Str": "h"
                },
                {
                  "Str": "i"
                }
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
                {
                  "Str": "h"
                },
                {
                  "Str": "i"
                }
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

#[test]
fn unit_struct() {
    #[derive(ToAutomerge)]
    struct Unit;

    assert_eq!(Value::Primitive(Primitive::Null), Unit.to_automerge());
}

#[test]
fn tuple_struct_1() {
    #[derive(ToAutomerge)]
    struct Single(u64);

    assert_eq!(
        Value::Primitive(Primitive::Uint(1)),
        Single(1).to_automerge()
    );
}

#[test]
fn tuple_struct_2() {
    #[derive(ToAutomerge)]
    struct Double(u64, i64);

    assert_eq!(
        Value::Sequence(vec![
            Value::Primitive(Primitive::Uint(1)),
            Value::Primitive(Primitive::Int(-2))
        ]),
        Double(1, -2).to_automerge()
    );
}

#[test]
fn tuple_struct_3() {
    #[derive(ToAutomerge)]
    struct Triple(u64, i64, String);

    assert_eq!(
        Value::Sequence(vec![
            Value::Primitive(Primitive::Uint(1)),
            Value::Primitive(Primitive::Int(-2)),
            Value::Primitive(Primitive::Str(String::new()))
        ]),
        Triple(1, -2, String::new()).to_automerge()
    );
}

#[test]
fn struct_1() {
    #[derive(ToAutomerge)]
    struct Single {
        a: u64,
    }

    assert_eq!(
        Value::Map(
            hashmap! {"a".to_owned() => Value::Primitive(Primitive::Uint(1))},
            MapType::Map
        ),
        Single { a: 1 }.to_automerge()
    );
}

#[test]
fn struct_2() {
    #[derive(ToAutomerge)]
    struct Double {
        a: u64,
        b: String,
    }

    assert_eq!(
        Value::Map(
            hashmap! {
                "a".to_owned() => Value::Primitive(Primitive::Uint(1)),
                "b".to_owned() => Value::Primitive(Primitive::Str(String::new()))
            },
            MapType::Map
        ),
        Double {
            a: 1,
            b: String::new()
        }
        .to_automerge()
    );
}

#[test]
fn enum_multi() {
    #[derive(ToAutomerge)]
    enum E {
        W { a: i32, b: i32 },
        X(i32, i32),
        Y(i32),
        Z,
    }

    assert_eq!(
        Value::Map(
            hashmap! {
                "W".to_owned() => Value::Map(hashmap!{
                    "a".to_owned() => Value::Primitive(Primitive::Int(0)),
                    "b".to_owned() => Value::Primitive(Primitive::Int(1)),
                },MapType::Map),
            },
            MapType::Map
        ),
        E::W { a: 0, b: 1 }.to_automerge()
    );

    assert_eq!(
        Value::Map(
            hashmap! {
                "X".to_owned() => Value::Sequence(vec![
                    Value::Primitive(Primitive::Int(0)),
                    Value::Primitive(Primitive::Int(1)),
                ]),
            },
            MapType::Map
        ),
        E::X(0, 1).to_automerge()
    );

    assert_eq!(
        Value::Map(
            hashmap! {
                "Y".to_owned() => Value::Primitive(Primitive::Int(0)),
            },
            MapType::Map
        ),
        E::Y(0).to_automerge()
    );

    assert_eq!(
        Value::Primitive(Primitive::Str("Z".to_owned())),
        E::Z.to_automerge()
    );
}

#[test]
fn enum_names() {
    #[derive(ToAutomerge)]
    enum Names {
        A,
        B,
        C,
    }

    assert_eq!(
        Value::Primitive(Primitive::Str("A".to_owned())),
        Names::A.to_automerge()
    );
    assert_eq!(
        Value::Primitive(Primitive::Str("B".to_owned())),
        Names::B.to_automerge()
    );
    assert_eq!(
        Value::Primitive(Primitive::Str("C".to_owned())),
        Names::C.to_automerge()
    );
}
