use std::{
    collections::{BTreeMap, HashMap},
    convert::{TryFrom, TryInto},
    error::Error,
    hash::Hash,
    time::SystemTime,
};

use automerge::{Primitive, Value};
use serde_json::Number;

/// Require a method to convert to a value from an automerge value.
pub trait FromAutomerge: Sized {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError>;
}

#[derive(thiserror::Error, Debug)]
pub enum FromAutomergeError {
    #[error("found the wrong type")]
    WrongType { found: automerge::Value },
    #[error("failed converting from automerge")]
    FailedTryFrom,
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn Error>),
}

impl FromAutomerge for String {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Str(s)) = value {
            Ok(s.to_owned())
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
            })
        }
    }
}

impl<T> FromAutomerge for Vec<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Sequence(vec) = value {
            let mut v = Vec::new();
            for val in vec {
                v.push(T::from_automerge(val)?)
            }
            Ok(v)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
            })
        }
    }
}

impl FromAutomerge for Vec<char> {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Text(vec) = value {
            Ok(vec.to_vec())
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
            })
        }
    }
}

impl<K, V> FromAutomerge for HashMap<K, V>
where
    K: TryFrom<String> + Eq + Hash,
    V: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Map(map, automerge::MapType::Map) = value {
            let mut m = HashMap::new();
            for (k, v) in map {
                if let Ok(k) = K::try_from(k.clone()) {
                    m.insert(k, V::from_automerge(v)?);
                } else {
                    return Err(FromAutomergeError::FailedTryFrom);
                }
            }
            Ok(m)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
            })
        }
    }
}

impl<K, V> FromAutomerge for BTreeMap<K, V>
where
    K: TryFrom<String> + Eq + Ord,
    V: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Map(map, automerge::MapType::Map) = value {
            let mut m = BTreeMap::new();
            for (k, v) in map {
                if let Ok(k) = K::try_from(k.clone()) {
                    m.insert(k, V::from_automerge(v)?);
                } else {
                    return Err(FromAutomergeError::FailedTryFrom);
                }
            }
            Ok(m)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
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

impl FromAutomerge for SystemTime {
    fn from_automerge(value: &automerge::Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Timestamp(t)) = value {
            let duration = std::time::Duration::from_secs((*t).try_into().unwrap());
            Ok(SystemTime::UNIX_EPOCH + duration)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
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
        i64::from_automerge(value).map(|i| i as isize)
    }
}

impl FromAutomerge for i128 {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        i64::from_automerge(value).map(|i| i as i128)
    }
}

impl FromAutomerge for u64 {
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(Primitive::Uint(u)) = value {
            Ok(*u)
        } else {
            Err(FromAutomergeError::WrongType {
                found: value.clone(),
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
        u64::from_automerge(value).map(|u| u as usize)
    }
}

impl FromAutomerge for u128 {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        u64::from_automerge(value).map(|u| u as u128)
    }
}

impl FromAutomerge for serde_json::Value {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        let var_name = match value {
            Value::Map(map, _) => Ok(serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::from_automerge(v).unwrap()))
                    .collect(),
            )),
            Value::Sequence(v) => Ok(serde_json::Value::Array(
                v.iter()
                    .map(|i| serde_json::Value::from_automerge(i).unwrap())
                    .collect::<Vec<_>>(),
            )),
            Value::Text(v) => Ok(serde_json::Value::String(v.iter().collect())),
            Value::Primitive(p) => match p {
                Primitive::Str(s) => Ok(serde_json::Value::String(s.clone())),
                Primitive::Int(i) => Ok(serde_json::Value::Number(Number::from(*i))),
                Primitive::Uint(u) => Ok(serde_json::Value::Number(Number::from(*u))),
                Primitive::F64(f) => Ok(serde_json::Value::Number(Number::from_f64(*f).unwrap())),
                Primitive::F32(f) => Ok(serde_json::Value::Number(
                    Number::from_f64((*f).into()).unwrap(),
                )),
                Primitive::Counter(i) => Ok(serde_json::Value::Number(Number::from(*i))),
                Primitive::Timestamp(i) => Ok(serde_json::Value::Number(Number::from(*i))),
                Primitive::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
                Primitive::Cursor(_) => {
                    panic!("cursor is unsupported")
                }
                Primitive::Null => Ok(serde_json::Value::Null),
            },
        };
        var_name
    }
}
