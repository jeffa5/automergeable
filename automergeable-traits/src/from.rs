use std::{collections::HashMap, convert::TryInto, error::Error, time::SystemTime};

use automerge::{ScalarValue, Value};

/// Require a method to convert to a value from an automerge value.
pub trait FromAutomerge: Sized {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError>;
}

#[derive(thiserror::Error, Debug)]
pub enum FromAutomergeError {
    #[error("found the wrong type")]
    WrongType,
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn Error>),
}

impl FromAutomerge for String {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(ScalarValue::Str(s)) = value {
            Ok(s.to_owned())
        } else {
            Err(FromAutomergeError::WrongType)
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
            Err(FromAutomergeError::WrongType)
        }
    }
}

impl FromAutomerge for Vec<char> {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Text(vec) = value {
            Ok(vec.to_vec())
        } else {
            Err(FromAutomergeError::WrongType)
        }
    }
}

impl<V> FromAutomerge for HashMap<String, V>
where
    V: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Map(map, automerge::MapType::Map) = value {
            let mut m = HashMap::new();
            for (k, v) in map {
                m.insert(k.clone(), V::from_automerge(v)?);
            }
            Ok(m)
        } else {
            Err(FromAutomergeError::WrongType)
        }
    }
}

impl<T> FromAutomerge for Option<T>
where
    T: FromAutomerge,
{
    fn from_automerge(value: &automerge::Value) -> std::result::Result<Self, FromAutomergeError> {
        if let Value::Primitive(ScalarValue::Null) = value {
            Ok(None)
        } else {
            Ok(Some(T::from_automerge(value)?))
        }
    }
}

impl FromAutomerge for SystemTime {
    fn from_automerge(value: &automerge::Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(ScalarValue::Timestamp(t)) = value {
            let duration = std::time::Duration::from_secs((*t).try_into().unwrap());
            Ok(SystemTime::UNIX_EPOCH + duration)
        } else {
            Err(FromAutomergeError::WrongType)
        }
    }
}

impl FromAutomerge for bool {
    fn from_automerge(value: &Value) -> Result<Self, FromAutomergeError> {
        if let Value::Primitive(ScalarValue::Boolean(b)) = value {
            Ok(*b)
        } else {
            Err(FromAutomergeError::WrongType)
        }
    }
}
