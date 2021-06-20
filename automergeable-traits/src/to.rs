use std::{
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    rc::Rc,
    sync::Arc,
};

use automerge::{Primitive, Value};
use smol_str::SmolStr;

/// Require a method to convert the current value into an automerge value.
pub trait ToAutomerge {
    fn to_automerge(&self) -> Value;
}

impl ToAutomerge for Value {
    fn to_automerge(&self) -> Value {
        self.clone()
    }
}

impl ToAutomerge for () {
    fn to_automerge(&self) -> automerge::Value {
        Value::Primitive(Primitive::Null)
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

// impl<T> ToAutomerge for HashSet<T>
// where
//     T: ToAutomerge,
// {
//     fn to_automerge(&self) -> Value {
//         let vals = self.iter().map(|v| v.to_automerge()).collect::<Vec<_>>();
//         Value::Sequence(vals)
//     }
// }

impl<K, V> ToAutomerge for HashMap<K, V>
where
    K: AsRef<str>,
    V: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let mut hm = HashMap::with_capacity(self.len());
        for (k, v) in self {
            hm.insert(SmolStr::new(k), v.to_automerge());
        }
        Value::Map(hm)
    }
}

impl<K, V> ToAutomerge for BTreeMap<K, V>
where
    K: AsRef<str>,
    V: ToAutomerge,
{
    fn to_automerge(&self) -> Value {
        let mut hm = HashMap::with_capacity(self.len());
        for (k, v) in self {
            hm.insert(SmolStr::new(k), v.to_automerge());
        }
        Value::Map(hm)
    }
}

impl ToAutomerge for String {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Str(SmolStr::new(self)))
    }
}

impl ToAutomerge for char {
    fn to_automerge(&self) -> Value {
        Value::Primitive(Primitive::Str(SmolStr::new(self.to_string())))
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
        self.as_ref()
            .map_or(Value::Primitive(Primitive::Null), |v| v.to_automerge())
    }
}

#[cfg(feature = "std")]
impl ToAutomerge for std::time::SystemTime {
    fn to_automerge(&self) -> Value {
        let ts = self
            .duration_since(Self::UNIX_EPOCH)
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
    // TODO: need to convert back to text and bytes somehow
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
            serde_json::Value::String(s) => Value::Primitive(Primitive::Str(SmolStr::new(s))),
            serde_json::Value::Array(a) => {
                Value::Sequence(a.iter().map(|i| i.to_automerge()).collect::<Vec<_>>())
            }
            serde_json::Value::Object(m) => Value::Map(
                m.iter()
                    .map(|(k, v)| (SmolStr::new(k), v.to_automerge()))
                    .collect::<HashMap<_, _>>(),
            ),
        }
    }
}

macro_rules! refs {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl<T> ToAutomerge for $x
        where
            T: ToAutomerge,
        {
            fn to_automerge(&self) -> Value {
                (**self).to_automerge()
            }
        })*
    };
}

refs! {
    Box<T>,
    Rc<T>,
    Arc<T>,
}
