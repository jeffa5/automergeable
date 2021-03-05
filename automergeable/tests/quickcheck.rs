use std::collections::HashMap;

use automerge::{MapType, Primitive, Value};
use automergeable::diff_values;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

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
            3 => Primitive::F64(f64::arbitrary(g)),
            4 => Primitive::F32(f32::arbitrary(g)),
            5 => Primitive::Counter(i64::arbitrary(g)),
            6 => Primitive::Timestamp(i64::arbitrary(g)),
            7 => Primitive::Boolean(bool::arbitrary(g)),
            8 => Primitive::Null, // TODO: convert this case to use an arbitrary cursor
            _ => Primitive::Null,
        };
        Self(p)
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

impl Arbitrary for Val {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let depth = g.choose(&[1, 2, 3]).unwrap();
        arbitrary_value(g, *depth)
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
        2 => {
            let vec = Vec::<char>::arbitrary(g);
            Value::Text(vec)
        }
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
        if changes.is_empty() {
            TestResult::passed()
        } else {
            println!("{:?}", changes);
            TestResult::failed()
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
        if changes.is_empty() {
            TestResult::passed()
        } else {
            println!("{:?}", changes);
            TestResult::failed()
        }
    }
    QuickCheck::new()
        .tests(100_000_000)
        .gen(Gen::new(20))
        .quickcheck(no_diff as fn(Val, Val) -> TestResult)
}
