use std::convert::TryInto;

use automerge::{LocalChange, Path, ScalarValue, Value};

use crate::ToAutomerge;

/// Calculate the `LocalChange`s between the two types that implement `ToAutomerge`.
///
/// Recursively works from the root.
pub fn diff<T>(new: &T, old: &T) -> Vec<LocalChange>
where
    T: ToAutomerge,
{
    diff_values(&new.to_automerge(), &old.to_automerge())
}

/// Calculate the `LocalChange`s between the two values.
///
/// Recursively works from the root.
pub fn diff_values(new: &Value, old: &Value) -> Vec<LocalChange> {
    diff_with_path(new, old, Path::root())
}

fn diff_with_path(new: &Value, old: &Value, path: Path) -> Vec<LocalChange> {
    match (new, old) {
        (Value::Map(new_map, mt1), Value::Map(old_map, mt2)) if mt1 == mt2 => {
            let mut changes = Vec::new();
            for (k, v) in new_map {
                if let Some(old_v) = old_map.get(k) {
                    // changed
                    changes.append(&mut diff_with_path(v, old_v, path.clone().key(k)))
                } else {
                    // new
                    changes.push(LocalChange::insert(path.clone().key(k), v.clone()))
                }
            }
            for k in old_map.keys() {
                if !new_map.contains_key(k) {
                    // removed
                    changes.push(LocalChange::delete(path.clone().key(k)))
                }
            }
            changes
        }
        (Value::Sequence(new_vec), Value::Sequence(old_vec)) => {
            let mut changes = Vec::new();
            // naive
            for (i, v) in new_vec.iter().enumerate() {
                if let Some(old_v) = old_vec.get(i) {
                    // changed
                    changes.append(&mut diff_with_path(
                        v,
                        old_v,
                        path.clone().index(i.try_into().unwrap()),
                    ))
                } else {
                    // new
                    changes.push(LocalChange::insert(
                        path.clone().index(i.try_into().unwrap()),
                        v.clone(),
                    ))
                }
            }
            for i in new_vec.len()..old_vec.len() {
                // removed
                changes.push(LocalChange::delete(
                    path.clone().index(i.try_into().unwrap()),
                ))
            }
            changes
        }
        (Value::Text(new_vec), Value::Text(old_vec)) => {
            let mut changes = Vec::new();
            // naive
            for (i, v) in new_vec.iter().enumerate() {
                if i < old_vec.len() {
                    // changed
                    changes.push(LocalChange::set(
                        path.clone().index(i.try_into().unwrap()),
                        Value::Primitive(ScalarValue::Str(v.to_string())),
                    ))
                } else {
                    // new
                    changes.push(LocalChange::insert(
                        path.clone().index(i.try_into().unwrap()),
                        Value::Primitive(ScalarValue::Str(v.to_string())),
                    ))
                }
            }
            for i in new_vec.len()..old_vec.len() {
                // removed
                changes.push(LocalChange::delete(
                    path.clone().index(i.try_into().unwrap()),
                ))
            }
            changes
        }
        (
            Value::Primitive(ScalarValue::Str(new_string)),
            Value::Primitive(ScalarValue::Str(_old_string)),
        ) => {
            // just set this, we can't address the characters so this may be a thing such as an enum
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Str(new_string.to_owned())),
            )]
        }
        (
            Value::Primitive(ScalarValue::Int(new_int)),
            Value::Primitive(ScalarValue::Int(_old_int)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Int(*new_int)),
            )]
        }
        (
            Value::Primitive(ScalarValue::Uint(new_int)),
            Value::Primitive(ScalarValue::Uint(_old_int)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Uint(*new_int)),
            )]
        }
        (
            Value::Primitive(ScalarValue::F64(new_int)),
            Value::Primitive(ScalarValue::F64(_old_int)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::F64(*new_int)),
            )]
        }
        (
            Value::Primitive(ScalarValue::F32(new_int)),
            Value::Primitive(ScalarValue::F32(_old_int)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::F32(*new_int)),
            )]
        }
        (
            Value::Primitive(ScalarValue::Counter(new_int)),
            Value::Primitive(ScalarValue::Counter(old_int)),
        ) => {
            // naive
            vec![LocalChange::increment_by(
                path,
                (new_int - old_int).try_into().unwrap(),
            )]
        }
        (
            Value::Primitive(ScalarValue::Timestamp(new_int)),
            Value::Primitive(ScalarValue::Timestamp(_old_int)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Timestamp(*new_int)),
            )]
        }
        (
            Value::Primitive(ScalarValue::Cursor(new_cursor)),
            Value::Primitive(ScalarValue::Cursor(_old_cursor)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Cursor(new_cursor.clone())),
            )]
        }
        (
            Value::Primitive(ScalarValue::Boolean(new_bool)),
            Value::Primitive(ScalarValue::Boolean(_old_bool)),
        ) => {
            // naive
            vec![LocalChange::set(
                path,
                Value::Primitive(ScalarValue::Boolean(*new_bool)),
            )]
        }
        (Value::Primitive(ScalarValue::Null), _) => {
            vec![LocalChange::set(path, Value::Primitive(ScalarValue::Null))]
        }
        (v, Value::Primitive(ScalarValue::Null)) => {
            vec![LocalChange::set(path, v.clone())]
        }
        (n, o) => panic!("unhandled diff case: {:?} {:?}", n, o),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use automerge::MapType;
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn diff_maps() {
        let mut old_map = HashMap::new();
        let mut new_map = HashMap::new();

        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @"[]");

        new_map.insert(
            "abc".to_owned(),
            ScalarValue::Str("some val".to_owned()).into(),
        );
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Key(
                            "abc",
                        ),
                    ],
                ),
                operation: Insert(
                    Primitive(
                        Str(
                            "some val",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_map = new_map.clone();
        new_map.insert(
            "abc".to_owned(),
            ScalarValue::Str("some newer val".to_owned()).into(),
        );
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Key(
                            "abc",
                        ),
                    ],
                ),
                operation: Set(
                    Primitive(
                        Str(
                            "some newer val",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_map = new_map.clone();
        new_map.remove("abc");
        assert_debug_snapshot!(diff_values(&Value::Map(new_map, MapType::Map), &Value::Map(old_map, MapType::Map)), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Key(
                            "abc",
                        ),
                    ],
                ),
                operation: Delete,
            },
        ]
        "###);
    }

    #[test]
    fn diff_vecs() {
        let mut old_vec = Vec::new();
        let mut new_vec = Vec::new();

        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone() ), &Value::Sequence(old_vec.clone() )), @"[]");

        new_vec.push(ScalarValue::Str("some val".to_owned()).into());
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone()), &Value::Sequence(old_vec.clone())), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Insert(
                    Primitive(
                        Str(
                            "some val",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_vec = new_vec.clone();
        new_vec[0] = ScalarValue::Str("some newer val".to_owned()).into();
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone() ), &Value::Sequence(old_vec.clone() )), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Set(
                    Primitive(
                        Str(
                            "some newer val",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_vec = new_vec.clone();
        new_vec.pop();
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec), &Value::Sequence(old_vec )), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Delete,
            },
        ]
        "###);
    }

    #[test]
    fn diff_text() {
        let mut old_text = Vec::new();
        let mut new_text = Vec::new();

        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone() ), &Value::Text(old_text.clone() )), @"[]");

        new_text.push('a');
        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone()), &Value::Text(old_text.clone())), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Insert(
                    Primitive(
                        Str(
                            "a",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_text = new_text.clone();
        new_text[0] = 'b';
        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone() ), &Value::Text(old_text.clone() )), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Set(
                    Primitive(
                        Str(
                            "b",
                        ),
                    ),
                ),
            },
        ]
        "###);

        old_text = new_text.clone();
        new_text.pop();
        assert_debug_snapshot!(diff_values(&Value::Text(new_text), &Value::Text(old_text )), @r###"
        [
            LocalChange {
                path: Path(
                    [
                        Index(
                            0,
                        ),
                    ],
                ),
                operation: Delete,
            },
        ]
        "###);
    }
}
