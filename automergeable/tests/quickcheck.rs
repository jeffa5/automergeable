use std::{collections::HashMap, convert::Infallible};

use automerge::{InvalidChangeRequest, MapType, Path, Primitive, Value};
use automergeable::diff_values;
use maplit::hashmap;
use quickcheck::{empty_shrinker, single_shrinker, Arbitrary, Gen, QuickCheck, TestResult};

#[derive(Debug, Clone, PartialEq)]
struct Prim(Primitive);

impl Arbitrary for Prim {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let prims = [
            0, // Str(String),
            1, // Int(i64),
            2, // Uint(u64),
            3, // F64(f64),
            4, // F32(f32),
            5, // Counter(i64),
            6, // Timestamp(i64),
            7, // Boolean(bool),
            8, // Cursor(Cursor),
            9, // Null
        ];
        let prim = g.choose(&prims).unwrap();
        let p = match prim {
            0 => Primitive::Str(String::arbitrary(g)),
            1 => Primitive::Int(i64::arbitrary(g)),
            2 => Primitive::Uint(u64::arbitrary(g)),
            3 => Primitive::F64(i32::arbitrary(g) as f64), /* avoid having NaN in as it breaks the equality */
            4 => Primitive::F32(i32::arbitrary(g) as f32), /* avoid having NaN in as it breaks the equality */
            5 => Primitive::Counter(i64::arbitrary(g)),
            6 => Primitive::Timestamp(i64::arbitrary(g)),
            7 => Primitive::Boolean(bool::arbitrary(g)),
            8 => Primitive::Null, // TODO: convert this case to use an arbitrary cursor
            _ => Primitive::Null,
        };
        Self(p)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match &self.0 {
            Primitive::Str(s) => {
                if s.is_empty() {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(s.shrink().map(Primitive::Str).map(Prim))
                }
            }
            Primitive::Int(i) => {
                if *i == 0 {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(i.shrink().map(Primitive::Int).map(Prim))
                }
            }
            Primitive::Uint(u) => {
                if *u == 0 {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(u.shrink().map(Primitive::Uint).map(Prim))
                }
            }
            Primitive::F64(f) => {
                if *f == 0. {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(f.shrink().map(Primitive::F64).map(Prim))
                }
            }
            Primitive::F32(f) => {
                if *f == 0. {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(f.shrink().map(Primitive::F32).map(Prim))
                }
            }
            Primitive::Counter(c) => {
                if *c == 0 {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(c.shrink().map(Primitive::Counter).map(Prim))
                }
            }
            Primitive::Timestamp(t) => {
                if *t == 0 {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(t.shrink().map(Primitive::Timestamp).map(Prim))
                }
            }
            Primitive::Boolean(b) => {
                if !b {
                    Box::new(single_shrinker(Prim(Primitive::Null)))
                } else {
                    Box::new(b.shrink().map(Primitive::Boolean).map(Prim))
                }
            }
            Primitive::Cursor(_) => empty_shrinker(),
            Primitive::Null => empty_shrinker(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MapTy(MapType);

impl Arbitrary for MapTy {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if *g.choose(&[0, 1]).unwrap() == 0 {
            MapTy(MapType::Map)
        } else {
            MapTy(MapType::Table)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Val(Value);

impl Default for Val {
    fn default() -> Self {
        Self(Value::Primitive(Primitive::Null))
    }
}

impl Arbitrary for Val {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let depth = g.choose(&[1, 2, 3]).unwrap();
        arbitrary_value(g, *depth)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match &self.0 {
            Value::Map(m, mt) => {
                if m.is_empty() {
                    single_shrinker(Val(Value::Primitive(Primitive::Null)))
                } else {
                    let m = m
                        .iter()
                        .map(|(k, v)| (k.clone(), Val(v.clone())))
                        .collect::<HashMap<_, _>>();
                    let mt = *mt;
                    Box::new(
                        m.shrink()
                            .map(move |m| {
                                let m = m
                                    .into_iter()
                                    .map(|(k, v)| (k, v.0))
                                    .collect::<HashMap<_, _>>();
                                Value::Map(m, mt)
                            })
                            .map(Val),
                    )
                }
            }
            Value::Sequence(v) => {
                if v.is_empty() {
                    single_shrinker(Val(Value::Primitive(Primitive::Null)))
                } else {
                    let v = v.iter().map(|v| Val(v.clone())).collect::<Vec<_>>();
                    Box::new(
                        v.shrink()
                            .map(|v| {
                                let v = v.into_iter().map(|i| i.0).collect::<Vec<_>>();
                                Value::Sequence(v)
                            })
                            .map(Val),
                    )
                }
            }
            Value::Text(v) => Box::new(v.shrink().map(Value::Text).map(Val)),
            Value::Primitive(p) => Box::new(
                Prim(p.clone())
                    .shrink()
                    .map(|p| p.0)
                    .map(Value::Primitive)
                    .map(Val),
            ),
        }
    }
}

fn arbitrary_value(g: &mut Gen, depth: usize) -> Val {
    let vals = if depth == 0 {
        &[
            2, // Text(Vec<char, Global>),
            3, // Primitive(Primitive),
        ][..]
    } else {
        &[
            0, // Map(HashMap<String, Value, RandomState>, MapType),
            1, // Sequence(Vec<Value, Global>),
            2, // Text(Vec<char, Global>),
            3, // Primitive(Primitive),
        ][..]
    };
    let val = g.choose(vals).unwrap();
    let v = match val {
        0 => {
            let smaller_depth = depth / 2;
            let map = HashMap::<String, ()>::arbitrary(g);
            let map = map
                .into_iter()
                .map(|(k, ())| (k, arbitrary_value(g, smaller_depth).0))
                .collect::<HashMap<_, _>>();
            let map_type = MapTy::arbitrary(g);
            Value::Map(map, map_type.0)
        }
        1 => {
            let smaller_depth = depth / 2;
            let vec = Vec::<()>::arbitrary(g);
            let vec = vec
                .into_iter()
                .map(|()| arbitrary_value(g, smaller_depth).0)
                .collect::<Vec<_>>();
            Value::Sequence(vec)
        }
        // 2 => {
        //     let vec = Vec::<char>::arbitrary(g);
        //     Value::Text(vec)
        // }
        _ => Value::Primitive(Prim::arbitrary(g).0),
    };
    Val(v)
}

#[test]
fn equal_primitives_give_no_diff() {
    fn no_diff(p1: Prim, p2: Prim) -> TestResult {
        if p1 != p2 {
            return TestResult::discard();
        }
        let v1 = Value::Primitive(p1.0);
        let v2 = Value::Primitive(p2.0);
        let changes = diff_values(&v1, &v2);
        if let Ok(changes) = changes {
            if changes.is_empty() {
                TestResult::passed()
            } else {
                println!("{:?}", changes);
                TestResult::failed()
            }
        } else {
            TestResult::discard()
        }
    }
    QuickCheck::new()
        .tests(100_000_000)
        .quickcheck(no_diff as fn(Prim, Prim) -> TestResult)
}

#[test]
fn equal_values_give_no_diff() {
    fn no_diff(v1: Val, v2: Val) -> TestResult {
        if v1 != v2 {
            return TestResult::discard();
        }
        let changes = diff_values(&v1.0, &v2.0);
        if let Ok(changes) = changes {
            if changes.is_empty() {
                TestResult::passed()
            } else {
                println!("{:?}", changes);
                TestResult::failed()
            }
        } else {
            TestResult::discard()
        }
    }
    QuickCheck::new()
        .tests(100_000_000)
        .gen(Gen::new(20))
        .quickcheck(no_diff as fn(Val, Val) -> TestResult)
}

#[test]
fn applying_primitive_diff_result_to_old_gives_new() {
    fn apply_diff(p1: Prim, p2: Prim) -> TestResult {
        let mut h1 = HashMap::new();
        h1.insert("k".to_owned(), Value::Primitive(p1.0));
        let v1 = Value::Map(h1, MapType::Map);
        let mut h2 = HashMap::new();
        h2.insert("k".to_owned(), Value::Primitive(p2.0));
        let v2 = Value::Map(h2, MapType::Map);
        let changes = diff_values(&v1, &v2);
        let changes = if let Ok(changes) = changes {
            changes
        } else {
            return TestResult::discard();
        };
        let mut b = automerge::Backend::new();
        // new with old value
        let (mut f, c) = automerge::Frontend::new_with_initial_state(v2).unwrap();
        let (p, _) = b.apply_local_change(c).unwrap();
        f.apply_patch(p).unwrap();

        // apply changes to reach new value
        let ((), c) = f
            .change::<_, _, Infallible>(None, |d| {
                for change in changes {
                    d.add_change(change).unwrap()
                }
                Ok(())
            })
            .unwrap();
        if let Some(c) = c {
            let (p, _) = b.apply_local_change(c).unwrap();
            f.apply_patch(p).unwrap();
        }

        let val = f.get_value(&Path::root()).unwrap();
        if val == v1 {
            TestResult::passed()
        } else {
            println!("expected: {:?}, found: {:?}", v1, val);
            TestResult::failed()
        }
    }

    QuickCheck::new()
        .tests(100_000_000)
        .quickcheck(apply_diff as fn(Prim, Prim) -> TestResult)
}

#[test]
fn applying_value_diff_result_to_old_gives_new() {
    fn apply_diff(v1: Val, v2: Val) -> TestResult {
        if let Val(Value::Map(_, MapType::Map)) = v1 {
        } else {
            return TestResult::discard();
        }
        if let Val(Value::Map(_, MapType::Map)) = v2 {
        } else {
            return TestResult::discard();
        }
        let changes = diff_values(&v1.0, &v2.0);
        let changes = if let Ok(changes) = changes {
            changes
        } else {
            return TestResult::discard();
        };
        let mut b = automerge::Backend::new();
        // new with old value
        let (mut f, c) = automerge::Frontend::new_with_initial_state(v2.0).unwrap();
        let (p, _) = b.apply_local_change(c).unwrap();
        f.apply_patch(p).unwrap();

        // apply changes to reach new value
        let c = f.change::<_, _, InvalidChangeRequest>(None, |d| {
            for change in &changes {
                d.add_change(change.clone())?
            }
            Ok(())
        });
        if let Ok(((), c)) = c {
            if let Some(c) = c {
                let (p, _) = b.apply_local_change(c).unwrap();
                if let Err(e) = f.apply_patch(p) {
                    println!("{:?} {:?}", changes, e);
                    return TestResult::failed();
                }
            }
        } else {
            println!("changes {:?} {:?}", changes, c);
            return TestResult::failed();
        }

        let val = f.get_value(&Path::root()).unwrap();
        if val == v1.0 {
            TestResult::passed()
        } else {
            println!("changes {:?}", changes);
            println!("expected: {:?}, found: {:?}", v1, val);
            TestResult::failed()
        }
    }

    QuickCheck::new()
        .tests(100_000_000)
        .gen(Gen::new(10))
        .quickcheck(apply_diff as fn(Val, Val) -> TestResult)
}

#[test]
fn broken_reordering_of_values_2() {
    let v1 = Val(Value::Map(
        hashmap! {"".to_owned()=> Value::Sequence(vec![ Value::Primitive(Primitive::Uint(0)), Value::Primitive(Primitive::Null)])},
        MapType::Map,
    ));

    let v2 = Val(Value::Map(
        hashmap! {"".to_owned()=> Value::Sequence(vec![ Value::Primitive(Primitive::Null)])},
        MapType::Map,
    ));

    let changes = diff_values(&v1.0, &v2.0).unwrap();
    let mut b = automerge::Backend::new();
    // new with old value
    let (mut f, c) = automerge::Frontend::new_with_initial_state(v2.0).unwrap();
    let (p, _) = b.apply_local_change(c).unwrap();
    f.apply_patch(p).unwrap();

    // apply changes to reach new value
    let c = f.change::<_, _, InvalidChangeRequest>(None, |d| {
        for change in &changes {
            d.add_change(change.clone())?
        }
        Ok(())
    });
    if let Ok(((), c)) = c {
        if let Some(c) = c {
            let (p, _) = b.apply_local_change(c).unwrap();
            if let Err(e) = f.apply_patch(p) {
                println!("{:?} {:?}", changes, e);
                panic!("failed apply_patch")
            }
        }
    } else {
        println!("changes {:?} {:?}", changes, c);
        panic!("failed change")
    }

    let val = f.get_value(&Path::root()).unwrap();
    assert_eq!(val, v1.0);
}

#[test]
fn save_then_load() {
    fn apply_diff(vals: Vec<Val>) -> TestResult {
        for val in &vals {
            if let Val(Value::Map(_, MapType::Map)) = val {
            } else {
                return TestResult::discard();
            }
        }

        let mut backend_bytes = Vec::new();
        let mut old: Option<Val> = None;
        let mut change_history = Vec::new();
        for val in vals {
            let changes = diff_values(&val.0, &old.unwrap_or_default().0);
            match changes {
                Err(InvalidChangeRequest::CannotOverwriteCounter { .. }) => {
                    return TestResult::discard()
                }
                Err(e) => {
                    println!("failed: {:?}", e);
                    return TestResult::failed();
                }
                Ok(changes) => {
                    change_history.push(changes.clone());
                    old = Some(val);
                    let mut backend = if backend_bytes.is_empty() {
                        automerge::Backend::new()
                    } else {
                        let b = automerge::Backend::load(backend_bytes);
                        if let Ok(b) = b {
                            b
                        } else {
                            println!("changes: {:?}", change_history);
                            println!("error loading: {:?}", b);
                            return TestResult::failed();
                        }
                    };

                    let mut frontend = automerge::Frontend::new();
                    let patch = backend.get_patch().unwrap();
                    frontend.apply_patch(patch).unwrap();

                    let ((), c) = frontend
                        .change::<_, _, InvalidChangeRequest>(None, |d| {
                            for change in &changes {
                                d.add_change(change.clone())?
                            }
                            Ok(())
                        })
                        .unwrap();
                    if let Some(change) = c {
                        backend.apply_local_change(change).unwrap();
                    }
                    backend_bytes = backend.save().unwrap();
                }
            }
        }
        TestResult::passed()
    }

    QuickCheck::new()
        .tests(100)
        .gen(Gen::new(30))
        .quickcheck(apply_diff as fn(Vec<Val>) -> TestResult)
}

#[test]
fn broken_save_load() {
    let mut m = HashMap::new();
    m.insert(
        "\u{0}\u{0}".to_owned(),
        Value::Sequence(vec![
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Boolean(false)),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Int(0)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Boolean(false)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::F64(0.0)),
        ]),
    );
    m.insert(
        "\u{2}".to_owned(),
        Value::Sequence(vec![
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
        ]),
    );
    m.insert(
        "\u{0}".to_owned(),
        Value::Sequence(vec![
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Int(0)),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::F32(0.0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::F32(0.0)),
            Value::Primitive(Primitive::Str("".to_owned())),
        ]),
    );
    m.insert(
        "".to_owned(),
        Value::Sequence(vec![
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::Int(0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::F32(0.0)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Uint(0)),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Timestamp(0)),
            Value::Primitive(Primitive::Str("".to_owned())),
            Value::Primitive(Primitive::Boolean(false)),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Int(0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Null),
            Value::Primitive(Primitive::F64(0.0)),
            Value::Primitive(Primitive::Counter(0)),
            Value::Primitive(Primitive::Boolean(false)),
        ]),
    );
    m.insert(
        "\u{1}".to_owned(),
        Value::Map(
            {
                let mut m = HashMap::new();
                m.insert("".to_owned(), Value::Primitive(Primitive::F64(0.0)));
                m
            },
            MapType::Table,
        ),
    );
    let vals = vec![
        Val(Value::Map(m, MapType::Map)),
        Val(Value::Map(HashMap::new(), MapType::Map)),
    ];

    let mut backend_bytes = Vec::new();
    let mut old: Option<Val> = None;
    for val in vals {
        let changes = diff_values(&val.0, &old.unwrap_or_default().0).unwrap();
        println!("changes: {:?}", changes);
        old = Some(val);
        let mut backend = if backend_bytes.is_empty() {
            automerge::Backend::new()
        } else {
            let b = automerge::Backend::load(backend_bytes);
            if let Ok(b) = b {
                b
            } else {
                println!("error loading: {:?}", b);
                panic!("failed loading")
            }
        };

        let mut frontend = automerge::Frontend::new();
        let patch = backend.get_patch().unwrap();
        frontend.apply_patch(patch).unwrap();

        let ((), c) = frontend
            .change::<_, _, InvalidChangeRequest>(None, |d| {
                for change in &changes {
                    d.add_change(change.clone())?
                }
                Ok(())
            })
            .unwrap();
        if let Some(change) = c {
            backend.apply_local_change(change).unwrap();
        }
        backend_bytes = backend.save().unwrap();
    }
}

#[test]
fn broken_save_load_2() {
    let mut hm1 = HashMap::new();
    hm1.insert("a".to_owned(), Value::Primitive(Primitive::Null));
    let mut hm2 = HashMap::new();
    hm2.insert("".to_owned(), Value::Primitive(Primitive::Null));
    let values = vec![
        Value::Map(HashMap::new(), MapType::Map),
        Value::Map(hm1, MapType::Map),
        Value::Map(hm2, MapType::Map),
        Value::Map(HashMap::new(), MapType::Map),
    ];

    let mut frontend = automerge::Frontend::new();
    let mut backend_bytes = Vec::new();
    for val_pair in values.windows(2) {
        let changes = diff_values(&val_pair[1], &val_pair[0]).unwrap();
        println!("changes: {:?}", changes);
        let mut backend = if backend_bytes.is_empty() {
            automerge::Backend::new()
        } else {
            let b = automerge::Backend::load(backend_bytes);
            if let Ok(b) = b {
                b
            } else {
                println!("error loading: {:?}", b);
                panic!("failed loading")
            }
        };

        let ((), c) = frontend
            .change::<_, _, InvalidChangeRequest>(None, |d| {
                for change in &changes {
                    d.add_change(change.clone())?
                }
                Ok(())
            })
            .unwrap();
        if let Some(change) = c {
            backend.apply_local_change(change).unwrap();
        }
        backend_bytes = backend.save().unwrap();
    }
}

#[test]
fn broken_reordering_of_values() {
    // setup
    let mut hm = std::collections::HashMap::new();
    hm.insert(
        "".to_owned(),
        automerge::Value::Sequence(vec![automerge::Value::Primitive(Primitive::Null)]),
    );
    let mut backend = automerge::Backend::new();

    // new frontend with initial state
    let (mut frontend, change) =
        automerge::Frontend::new_with_initial_state(Value::Map(hm, automerge::MapType::Map))
            .unwrap();

    println!("change1 {:?}", change);

    // get patch and apply
    let (patch, _) = backend.apply_local_change(change).unwrap();
    frontend.apply_patch(patch).unwrap();

    // change first value and insert into the sequence
    let ((), c) = frontend
        .change::<_, _, automerge::InvalidChangeRequest>(None, |d| {
            d.add_change(automerge::LocalChange::set(
                automerge::Path::root().key("").index(0),
                automerge::Value::Primitive(automerge::Primitive::Int(0)),
            ))
            .unwrap();
            d.add_change(automerge::LocalChange::insert(
                automerge::Path::root().key("").index(1),
                automerge::Value::Primitive(automerge::Primitive::Boolean(false)),
            ))
            .unwrap();
            Ok(())
        })
        .unwrap();

    // setup first expected
    let mut ehm = HashMap::new();
    ehm.insert(
        "".to_owned(),
        automerge::Value::Sequence(vec![
            automerge::Value::Primitive(automerge::Primitive::Int(0)),
            automerge::Value::Primitive(automerge::Primitive::Boolean(false)),
        ]),
    );
    let expected = automerge::Value::Map(ehm.clone(), automerge::MapType::Map);

    // ok, sequence has int then bool
    assert_eq!(expected, frontend.get_value(&Path::root()).unwrap());

    // now apply the change to the backend and bring the patch back to the frontend
    if let Some(c) = c {
        println!("change2 {:?}", c);
        let (p, _) = backend.apply_local_change(c).unwrap();
        frontend.apply_patch(p).unwrap();
    }
    let v = frontend.get_value(&Path::root()).unwrap();

    let expected = automerge::Value::Map(ehm, automerge::MapType::Map);
    // not ok! sequence has bool then int
    assert_eq!(expected, v);
}
