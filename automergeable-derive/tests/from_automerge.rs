use automergeable::{Automergeable, FromAutomerge, ToAutomerge};
use insta::{assert_json_snapshot, Settings};
use pretty_assertions::assert_eq;
use serde::Serialize;
use std::collections::HashMap;

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

#[test]
fn unit_struct() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Unit;

    assert_eq!(Unit, Unit::from_automerge(&Unit.to_automerge()).unwrap());
}

#[test]
fn tuple_struct_1() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Single(u64);

    assert_eq!(
        Single(1),
        Single::from_automerge(&Single(1).to_automerge()).unwrap()
    );

    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct SingleVec(Vec<u8>);

    assert_eq!(
        SingleVec(vec![1]),
        SingleVec::from_automerge(&SingleVec(vec![1]).to_automerge()).unwrap()
    );
}

#[test]
fn tuple_struct_2() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Double(u64, i64);

    assert_eq!(
        Double(1, -2),
        Double::from_automerge(&Double(1, -2).to_automerge()).unwrap()
    );
}

#[test]
fn tuple_struct_3() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Triple(u64, i64, String);

    assert_eq!(
        Triple(1, -2, String::new()),
        Triple::from_automerge(&Triple(1, -2, String::new()).to_automerge()).unwrap()
    );
}

#[test]
fn struct_1() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Single {
        a: u64,
    }

    assert_eq!(
        Single { a: 1 },
        Single::from_automerge(&Single { a: 1 }.to_automerge()).unwrap()
    );
}

#[test]
fn struct_2() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct Double {
        a: u64,
        b: String,
    }

    assert_eq!(
        Double {
            a: 1,
            b: String::new()
        },
        Double::from_automerge(
            &Double {
                a: 1,
                b: String::new()
            }
            .to_automerge()
        )
        .unwrap()
    );
}

#[test]
fn enum_multi() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    enum E {
        W { a: i32, b: i32 },
        X(i32, i32),
        Y(i32),
        Z,
    }

    assert_eq!(
        E::W { a: 0, b: 1 },
        E::from_automerge(&E::W { a: 0, b: 1 }.to_automerge()).unwrap()
    );

    assert_eq!(
        E::X(0, 1),
        E::from_automerge(&E::X(0, 1).to_automerge()).unwrap()
    );

    assert_eq!(E::Y(0), E::from_automerge(&E::Y(0).to_automerge()).unwrap());

    assert_eq!(E::Z, E::from_automerge(&E::Z.to_automerge()).unwrap());
}

#[test]
fn enum_names() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    enum Names {
        A,
        B,
        C,
    }

    assert_eq!(
        Names::A,
        Names::from_automerge(&Names::A.to_automerge()).unwrap()
    );
    assert_eq!(
        Names::B,
        Names::from_automerge(&Names::B.to_automerge()).unwrap()
    );
    assert_eq!(
        Names::C,
        Names::from_automerge(&Names::C.to_automerge()).unwrap()
    );
}

#[test]
fn single_generics() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct A<T: ToAutomerge + FromAutomerge + Default> {
        inner: T,
    }

    assert_eq!(
        A { inner: 0u32 },
        A::from_automerge(&A { inner: 0u32 }.to_automerge()).unwrap()
    );

    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    enum B<T: ToAutomerge + FromAutomerge> {
        C(T),
    }

    assert_eq!(
        B::C(0u32),
        B::from_automerge(&B::C(0u32).to_automerge()).unwrap()
    )
}

#[test]
fn multi_generics() {
    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    struct A<T: ToAutomerge + FromAutomerge + Default, U: ToAutomerge + FromAutomerge + Default> {
        inner: T,
        outer: U,
    }

    assert_eq!(
        A {
            inner: 0u32,
            outer: String::new()
        },
        A::from_automerge(
            &A {
                inner: 0u32,
                outer: String::new()
            }
            .to_automerge()
        )
        .unwrap()
    );

    #[derive(ToAutomerge, FromAutomerge, PartialEq, Debug)]
    enum B<T: ToAutomerge + FromAutomerge + Default, U: ToAutomerge + FromAutomerge + Default> {
        C(T),
        D(U),
    }

    assert_eq!(
        B::<u32, String>::C(0u32),
        B::from_automerge(&B::<u32, String>::C(0u32).to_automerge()).unwrap()
    );
    assert_eq!(
        B::<u32, String>::D(String::new()),
        B::from_automerge(&B::<u32, String>::D(String::new()).to_automerge()).unwrap()
    );
}
