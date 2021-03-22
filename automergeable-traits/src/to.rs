use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryInto,
};

use automerge::{MapType, Primitive, Value};

/// Require a method to convert the current value into an automerge value.
pub trait ToAutomerge {
    fn to_automerge(&self) -> Value;
}

impl ToAutomerge for Value {
    fn to_automerge(&self) -> Value {
        self.clone()
    }
}

impl ToAutomerge for Vec<char> {
    fn to_automerge(&self) -> Value {
        Value::Text(self.clone())
    }
}

impl<T> ToAutomerge for Vec<T>
where
    T: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let vals = self.iter().map(|v| v.to_automerge()).collect::<Vec<_>>();
        Value::Sequence(vals)
    }
}

impl<T> ToAutomerge for HashSet<T>
where
    T: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let vals = self.iter().map(|v| v.to_automerge()).collect::<Vec<_>>();
        Value::Sequence(vals)
    }
}

impl<K, V> ToAutomerge for HashMap<K, V>
where
    K: ToString,
    V: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let mut hm = HashMap::new();
        for (k, v) in self {
            hm.insert(k.to_string(), v.to_automerge());
        }
        Value::Map(hm, MapType::Map)
    }
}

impl<K, V> ToAutomerge for BTreeMap<K, V>
where
    K: ToString,
    V: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let mut hm = HashMap::new();
        for (k, v) in self {
            hm.insert(k.to_string(), v.to_automerge());
        }
        Value::Map(hm, MapType::Map)
    }
}

impl ToAutomerge for String {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Str(self.to_owned()))
    }
}

impl ToAutomerge for f64 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::F64(*self))
    }
}

impl ToAutomerge for f32 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::F32(*self))
    }
}

impl ToAutomerge for bool {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Boolean(*self))
    }
}

impl<T> ToAutomerge for Option<T>
where
    T: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        if let Some(v) = self {
            v.to_automerge()
        } else {
            Value::Primitive(Primitive::Null)
        }
    }
}

#[cfg(feature = "std")]
impl ToAutomerge for std::time::SystemTime {
    fn to_automerge(&self) -> Value {
        let ts = self
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards");
        Value::Primitive(Primitive::Timestamp(ts.as_secs().try_into().unwrap()))
    }
}

macro_rules! as_i64_to_automerge {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl ToAutomerge for $x {
            fn to_automerge(&self) -> Value {
                (*self as i64).to_automerge()
            }
        })*
    };
}

as_i64_to_automerge! {
    i8,
    i16,
    i32,
}

impl ToAutomerge for i64 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Int(*self))
    }
}

impl ToAutomerge for isize {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Int((*self).try_into().unwrap()))
    }
}

impl ToAutomerge for i128 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Int((*self).try_into().unwrap()))
    }
}

macro_rules! as_u64_to_automerge {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl ToAutomerge for $x {
            fn to_automerge(&self) -> Value {
                (*self as u64).to_automerge()
            }
        })*
    };
}

as_u64_to_automerge! {
    u8,
    u16,
    u32,
}

impl ToAutomerge for u64 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Uint(*self))
    }
}

impl ToAutomerge for usize {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Uint((*self).try_into().unwrap()))
    }
}

impl ToAutomerge for u128 {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Uint((*self).try_into().unwrap()))
    }
}

macro_rules! nonzero_to_automerge {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl ToAutomerge for $x {
            fn to_automerge(&self) -> Value {
                self.get().to_automerge()
            }
        })*
    };
}

nonzero_to_automerge! {
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
}

impl ToAutomerge for serde_json::Value {
    fn to_automerge(&self) -> Value {
        match self {
            serde_json::Value::Null => automerge::Value::Primitive(Primitive::Null),
            serde_json::Value::Bool(b) => automerge::Value::Primitive(Primitive::Boolean(*b)),
            serde_json::Value::Number(n) => {
                if n.is_f64() {
                    automerge::Value::Primitive(Primitive::F64(n.as_f64().unwrap()))
                } else if n.is_i64() {
                    Value::Primitive(Primitive::Int(n.as_i64().unwrap()))
                } else {
                    Value::Primitive(Primitive::Uint(n.as_u64().unwrap()))
                }
            }
            serde_json::Value::String(s) => Value::Primitive(Primitive::Str(s.clone())),
            serde_json::Value::Array(a) => {
                Value::Sequence(a.iter().map(|i| i.to_automerge()).collect::<Vec<_>>())
            }
            serde_json::Value::Object(m) => Value::Map(
                m.iter()
                    .map(|(k, v)| (k.clone(), v.to_automerge()))
                    .collect::<HashMap<_, _>>(),
                MapType::Map,
            ),
        }
    }
}
