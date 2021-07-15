use std::{
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    error::Error,
    hash::Hash,
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use automerge::{Primitive, Value};
use serde_json::Number;
use smol_str::SmolStr;

/// Require a method to convert to a value from an automerge value.
pub trait FromAutomerge: Sized {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError>;
}

/// A failure converting from an automerge value to Rust type.
#[derive(thiserror::Error, Debug)]
pub enum FromAutomergeError {
    #[error("found the wrong type")]
    WrongType {
        found: automerge::Value,
        expected: String,
    },
    #[error("failed converting from automerge")]
    FailedTryFrom,
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn Error + Send + Sync>),
}

impl FromAutomerge for Value {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        Ok(value.clone())
    }
}

impl FromAutomerge for () {
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Null) = value {
            Ok(())
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive null".to_owned(),
            })
        }
    }
}

impl FromAutomerge for String {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Str(s)) = value {
            Ok(s.to_string())
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive string".to_owned(),
            })
        }
    }
}

impl FromAutomerge for char {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Str(s)) = value {
            if s.chars().count() == 1 {
                Ok(s.chars().next().unwrap())
            } else {
                Err(FromAutomergeError::WrongType {
                    found: value.clone(),
                    expected: "a primitive string".to_owned(),
                })
            }
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive string".to_owned(),
            })
        }
    }
}

impl<T> FromAutomerge for Vec<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::List(vec) = value {
            let mut v = Self::with_capacity(vec.len());
            for val in vec {
                v.push(T::from_automerge(val)?)
            }
            Ok(v)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a list".to_owned(),
            })
        }
    }
}

/// A new-type struct for working with the automerge Text value type.
pub struct Text(pub Vec<SmolStr>);

impl FromAutomerge for Text {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Text(vec) = value {
            Ok(Self(vec.clone()))
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "some text".to_owned(),
            })
        }
    }
}

// impl<T> FromAutomerge for HashSet<T>
// where
//     T: FromAutomerge + Clone + Eq + Hash,
// {
//     fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
//         if let Value::List(vec) = value {
//             let mut v = Vec::with_capacity(vec.len());
//             for val in vec {
//                 v.push(T::from_automerge(val)?)
//             }
//             Ok(v.iter().cloned().collect::<HashSet<_>>())
//         } else {
//             Err(FromAutomergeError::WrongType {
//                 found: value.clone(),
//                 expected: "a list".to_owned(),
//             })
//         }
//     }
// }

impl<K, V> FromAutomerge for HashMap<K, V>
where
    K: FromStr + Eq + Hash,
    V: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Map(map) = value {
            let mut m = Self::with_capacity(map.len());
            for (k, v) in map {
                if let Ok(k) = K::from_str(k) {
                    m.insert(k, V::from_automerge(v)?);
                } else {
                    return Err(FromAutomergeError::FailedTryFrom);
                }
            }
            Ok(m)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a map".to_owned(),
            })
        }
    }
}

impl<K, V> FromAutomerge for BTreeMap<K, V>
where
    K: FromStr + Eq + Ord,
    V: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Map(map) = value {
            let mut m = Self::new();
            for (k, v) in map {
                if let Ok(k) = K::from_str(k) {
                    m.insert(k, V::from_automerge(v)?);
                } else {
                    return Err(FromAutomergeError::FailedTryFrom);
                }
            }
            Ok(m)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a map".to_owned(),
            })
        }
    }
}

impl<T> FromAutomerge for Option<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Null) = value {
            Ok(None)
        } else {
            Ok(Some(T::from_automerge(value)?))
        }
    }
}

#[cfg(feature = "std")]
impl FromAutomerge for std::time::SystemTime {
    fn from_automerge(value: &automerge::Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Timestamp(t)) = value {
            let duration = std::time::Duration::from_secs((*t).try_into().unwrap());
            Ok(Self::UNIX_EPOCH + duration)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive timestamp".to_owned(),
            })
        }
    }
}

impl FromAutomerge for bool {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Boolean(b)) = value {
            Ok(*b)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive boolean".to_owned(),
            })
        }
    }
}

impl FromAutomerge for i64 {
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Int(i)) = value {
            Ok(*i)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive int".to_owned(),
            })
        }
    }
}

macro_rules! as_i64_from_automerge {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl FromAutomerge for $x {
            fn from_automerge(value: &automerge::Value) -> Result<Self, FromAutomergeError>{
                i64::from_automerge(value).map(|i| i as $x)
            }
        })*
    };
}

as_i64_from_automerge! {
    i8,i16,i32
}

impl FromAutomerge for isize {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        i64::from_automerge(value).map(|i| i as Self)
    }
}

impl FromAutomerge for i128 {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        i64::from_automerge(value).map(|i| i as Self)
    }
}

impl FromAutomerge for u64 {
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Uint(u)) = value {
            Ok(*u)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive uint".to_owned(),
            })
        }
    }
}

macro_rules! as_u64_from_automerge {
    ( $( $x:ty ),* $(,)? ) => {
        $(
        impl FromAutomerge for $x {
            fn from_automerge(value: &automerge::Value) -> Result<Self, FromAutomergeError>{
                u64::from_automerge(value).map(|u| u as $x)
            }
        })*
    };
}

as_u64_from_automerge! {
    u8,u16,u32
}

impl FromAutomerge for usize {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        u64::from_automerge(value).map(|u| u as Self)
    }
}

impl FromAutomerge for u128 {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        u64::from_automerge(value).map(|u| u as Self)
    }
}

impl FromAutomerge for f64 {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::F64(f)) = value {
            Ok(*f)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive f64".to_owned(),
            })
        }
    }
}

impl FromAutomerge for serde_json::Value {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        let var_name = match value {
            Value::Map(map) => Ok(Self::Object(
                map.iter()
                    .map(|(k, v)| (k.to_string(), Self::from_automerge(v).unwrap()))
                    .collect(),
            )),
            Value::Table(map) => Ok(Self::Object(
                map.iter()
                    .map(|(k, v)| (k.to_string(), Self::from_automerge(v).unwrap()))
                    .collect(),
            )),
            Value::List(v) => Ok(Self::Array(
                v.iter()
                    .map(|i| Self::from_automerge(i).unwrap())
                    .collect::<Vec<_>>(),
            )),
            Value::Text(v) => Ok(Self::String(v.concat())),
            Value::Primitive(p) => match p {
                Primitive::Bytes(b) => Ok(Self::Array(
                    b.iter().map(|b| Self::Number(Number::from(*b))).collect(),
                )),
                Primitive::Str(s) => Ok(Self::String(s.to_string())),
                Primitive::Int(i) | Primitive::Counter(i) => Ok(Self::Number(Number::from(*i))),
                Primitive::Uint(u) => Ok(Self::Number(Number::from(*u))),
                Primitive::F64(f) => Ok(Self::Number(Number::from_f64(*f).unwrap())),
                Primitive::Timestamp(i) => Ok(Self::Number(Number::from(*i))),
                Primitive::Boolean(b) => Ok(Self::Bool(*b)),
                Primitive::Cursor(_) => {
                    panic!("cursor is unsupported")
                }
                Primitive::Null => Ok(Self::Null),
            },
        };
        var_name
    }
}

macro_rules! nonzero_to_automerge_unsigned {
    ( $( ($x:ty, $y:ty) ),* $(,)? ) => {
        $(
        impl FromAutomerge for $x {
            fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
                if let Value::Primitive(Primitive::Uint(u)) = value {
                    let val = <$x>::new(*u as $y);
                    if let Some(val) = val {
                        Ok(val)
                    } else {
                        Err(FromAutomergeError::FailedTryFrom)
                    }
                } else {
                    Err(FromAutomergeError::WrongType {
                        found: value.clone(),
                        expected: "a primitive uint".to_owned(),
                    })
                }
            }
        })*
    };
}

nonzero_to_automerge_unsigned! {
    (std::num::NonZeroU8, u8),
    (std::num::NonZeroU16, u16),
    (std::num::NonZeroU32, u32),
    (std::num::NonZeroU64, u64),
    (std::num::NonZeroU128, u128),
    (std::num::NonZeroUsize, usize),
}

macro_rules! nonzero_to_automerge_signed {
    ( $( ($x:ty, $y:ty) ),* $(,)? ) => {
        $(
        impl FromAutomerge for $x {
            fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
                if let Value::Primitive(Primitive::Int(i)) = value {
                    let val = <$x>::new(*i as $y);
                    if let Some(val) = val {
                        Ok(val)
                    } else {
                        Err(FromAutomergeError::FailedTryFrom)
                    }
                } else {
                    Err(FromAutomergeError::WrongType {
                        found: value.clone(),
                        expected: "a primitive int".to_owned(),
                    })
                }
            }
        })*
    };
}

nonzero_to_automerge_signed! {
    (std::num::NonZeroI8, i8),
    (std::num::NonZeroI16, i16),
    (std::num::NonZeroI32, i32),
    (std::num::NonZeroI64, i64),
    (std::num::NonZeroI128, i128),
    (std::num::NonZeroIsize, isize),
}

impl<T> FromAutomerge for Box<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        T::from_automerge(value).map(Self::new)
    }
}

impl<T> FromAutomerge for Rc<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        T::from_automerge(value).map(Self::new)
    }
}

impl<T> FromAutomerge for Arc<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        T::from_automerge(value).map(Self::new)
    }
}
