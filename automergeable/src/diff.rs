use std::convert::TryInto;

use automerge::{InvalidChangeRequest, LocalChange, Path, Primitive, Value};

/// Calculate the [`LocalChange`]s between the two values.
///
/// Recursively works from the root.
pub fn diff_values(new: &Value, old: &Value) -> Result<Vec<LocalChange>, InvalidChangeRequest> {
    diff_with_path(Some(new), Some(old), Path::root())
}

/// Calculate the [`LocalChange`]s between the two values that start from the given path.
pub fn diff_with_path(
    new: Option<&Value>,
    old: Option<&Value>,
    path: Path,
) -> Result<Vec<LocalChange>, InvalidChangeRequest> {
    match (new, old) {
        (None, None) => Ok(Vec::new()),
        (Some(new), None) => Ok(vec![LocalChange::set(path, new.clone())]),
        (None, Some(_)) => Ok(vec![LocalChange::delete(path)]),
        (Some(new), Some(old)) => {
            match (new, old) {
                (Value::Map(new_map), Value::Map(old_map)) => {
                    let mut changes = Vec::new();
                    for (k, v) in new_map {
                        if let Some(old_v) = old_map.get(k) {
                            // changed
                            changes.append(&mut diff_with_path(
                                Some(v),
                                Some(old_v),
                                path.clone().key(k.clone()),
                            )?)
                        } else {
                            // new
                            changes.push(LocalChange::set(path.clone().key(k.clone()), v.clone()))
                        }
                    }
                    for k in old_map.keys() {
                        if !new_map.contains_key(k) {
                            // removed
                            changes.push(LocalChange::delete(path.clone().key(k.clone())))
                        }
                    }
                    Ok(changes)
                }
                (Value::SortedMap(new_map), Value::SortedMap(old_map)) => {
                    let mut changes = Vec::new();
                    for (k, v) in new_map {
                        if let Some(old_v) = old_map.get(k) {
                            // changed
                            changes.append(&mut diff_with_path(
                                Some(v),
                                Some(old_v),
                                path.clone().key(k.clone()),
                            )?)
                        } else {
                            // new
                            changes.push(LocalChange::set(path.clone().key(k.clone()), v.clone()))
                        }
                    }
                    for k in old_map.keys() {
                        if !new_map.contains_key(k) {
                            // removed
                            changes.push(LocalChange::delete(path.clone().key(k.clone())))
                        }
                    }
                    Ok(changes)
                }
                (Value::Table(new_map), Value::Table(old_map)) => {
                    let mut changes = Vec::new();
                    for (k, v) in new_map {
                        if let Some(old_v) = old_map.get(k) {
                            // changed
                            changes.append(&mut diff_with_path(
                                Some(v),
                                Some(old_v),
                                path.clone().key(k.clone()),
                            )?)
                        } else {
                            // new
                            changes.push(LocalChange::set(path.clone().key(k.clone()), v.clone()))
                        }
                    }
                    for k in old_map.keys() {
                        if !new_map.contains_key(k) {
                            // removed
                            changes.push(LocalChange::delete(path.clone().key(k.clone())))
                        }
                    }
                    Ok(changes)
                }
                (Value::List(new_vec), Value::List(old_vec)) => {
                    let mut changes = Vec::new();
                    // naive
                    for (i, v) in new_vec.iter().enumerate() {
                        if let Some(old_v) = old_vec.get(i) {
                            // changed
                            changes.append(&mut diff_with_path(
                                Some(v),
                                Some(old_v),
                                path.clone().index(i.try_into().unwrap()),
                            )?)
                        } else {
                            // new
                            changes.push(LocalChange::insert(
                                path.clone().index(i.try_into().unwrap()),
                                v.clone(),
                            ))
                        }
                    }
                    // reverse so delete from the end
                    for i in (new_vec.len()..old_vec.len()).rev() {
                        // removed
                        changes.push(LocalChange::delete(
                            path.clone().index(i.try_into().unwrap()),
                        ))
                    }
                    Ok(changes)
                }
                (Value::Text(new_vec), Value::Text(old_vec)) => {
                    let mut changes = Vec::new();
                    // naive
                    for (i, v) in new_vec.iter().enumerate() {
                        if let Some(old_v) = old_vec.get(i) {
                            if v != old_v {
                                // changed
                                changes.push(LocalChange::set(
                                    path.clone().index(i.try_into().unwrap()),
                                    Value::Primitive(Primitive::Str(v.clone())),
                                ))
                            }
                        } else {
                            // new
                            changes.push(LocalChange::insert(
                                path.clone().index(i.try_into().unwrap()),
                                Value::Primitive(Primitive::Str(v.clone())),
                            ))
                        }
                    }
                    // reverse so delete from the end
                    for i in (new_vec.len()..old_vec.len()).rev() {
                        // removed
                        changes.push(LocalChange::delete(
                            path.clone().index(i.try_into().unwrap()),
                        ))
                    }
                    Ok(changes)
                }
                (
                    Value::Primitive(Primitive::Str(new_string)),
                    Value::Primitive(Primitive::Str(old_string)),
                ) => {
                    // just set this, we can't address the characters so this may be a thing such as an enum
                    if new_string == old_string {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Str(new_string.clone())),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::Bytes(new)),
                    Value::Primitive(Primitive::Bytes(old)),
                ) => {
                    if new == old {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Bytes(new.clone())),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::Int(new_int)),
                    Value::Primitive(Primitive::Int(old_int)),
                ) => {
                    if new_int == old_int {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Int(*new_int)),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::Uint(new_int)),
                    Value::Primitive(Primitive::Uint(old_int)),
                ) => {
                    if new_int == old_int {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Uint(*new_int)),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::F64(new_int)),
                    Value::Primitive(Primitive::F64(old_int)),
                ) =>
                {
                    #[allow(clippy::float_cmp)]
                    if new_int == old_int {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::F64(*new_int)),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::Counter(new_int)),
                    Value::Primitive(Primitive::Counter(old_int)),
                ) => {
                    if new_int == old_int {
                        Ok(Vec::new())
                    } else {
                        let diff = if let Some(diff) = new_int.checked_sub(*old_int) {
                            diff
                        } else {
                            // TODO: perhaps change this behavior or change error type
                            return Err(InvalidChangeRequest::CannotOverwriteCounter { path });
                        };
                        Ok(vec![LocalChange::increment_by(path, diff)])
                    }
                }
                (
                    Value::Primitive(Primitive::Timestamp(new_int)),
                    Value::Primitive(Primitive::Timestamp(old_int)),
                ) => {
                    if new_int == old_int {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Timestamp(*new_int)),
                        )])
                    }
                }
                (
                    Value::Primitive(Primitive::Cursor(new_cursor)),
                    Value::Primitive(Primitive::Cursor(_old_cursor)),
                ) => {
                    // naive
                    Ok(vec![LocalChange::set(
                        path,
                        Value::Primitive(Primitive::Cursor(new_cursor.clone())),
                    )])
                }
                (
                    Value::Primitive(Primitive::Boolean(new_bool)),
                    Value::Primitive(Primitive::Boolean(old_bool)),
                ) => {
                    if new_bool == old_bool {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![LocalChange::set(
                            path,
                            Value::Primitive(Primitive::Boolean(*new_bool)),
                        )])
                    }
                }
                (Value::Primitive(Primitive::Null), Value::Primitive(Primitive::Null)) => {
                    Ok(Vec::new())
                }
                // handle mismatch combinations
                (_, Value::Primitive(Primitive::Counter(_))) => {
                    Err(InvalidChangeRequest::CannotOverwriteCounter { path })
                }
                (Value::Primitive(Primitive::Null), _) => Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Null),
                )]),
                (v, Value::Primitive(Primitive::Null)) => {
                    Ok(vec![LocalChange::set(path, v.clone())])
                }
                (n, _) => Ok(vec![LocalChange::set(path, n.clone())]),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn diff_maps() {
        let mut old_map = HashMap::new();
        let mut new_map = HashMap::new();

        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), ), &Value::Map(old_map.clone(), )), @r###"
        Ok(
            [],
        )
        "###);

        new_map.insert("abc".into(), Primitive::Str("some val".into()).into());
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), ), &Value::Map(old_map.clone(), )), @r###"
        Ok(
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
                                "some val",
                            ),
                        ),
                    ),
                },
            ],
        )
        "###);

        old_map = new_map.clone();
        new_map.insert("abc".into(), Primitive::Str("some newer val".into()).into());
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), ), &Value::Map(old_map.clone(), )), @r###"
        Ok(
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
            ],
        )
        "###);

        old_map = new_map.clone();
        new_map.remove("abc");
        assert_debug_snapshot!(diff_values(&Value::Map(new_map, ), &Value::Map(old_map, )), @r###"
        Ok(
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
            ],
        )
        "###);
    }

    #[test]
    fn diff_vecs() {
        let mut old_vec = Vec::new();
        let mut new_vec = Vec::new();

        assert_debug_snapshot!(diff_values(&Value::List(new_vec.clone() ), &Value::List(old_vec.clone() )), @r###"
        Ok(
            [],
        )
        "###);

        new_vec.push(Primitive::Str("some val".into()).into());
        assert_debug_snapshot!(diff_values(&Value::List(new_vec.clone()), &Value::List(old_vec.clone())), @r###"
        Ok(
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
            ],
        )
        "###);

        old_vec = new_vec.clone();
        new_vec[0] = Primitive::Str("some newer val".into()).into();
        assert_debug_snapshot!(diff_values(&Value::List(new_vec.clone() ), &Value::List(old_vec.clone() )), @r###"
        Ok(
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
            ],
        )
        "###);

        old_vec = new_vec.clone();
        new_vec.pop();
        assert_debug_snapshot!(diff_values(&Value::List(new_vec), &Value::List(old_vec )), @r###"
        Ok(
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
            ],
        )
        "###);
    }

    #[test]
    fn diff_text() {
        let mut old_text = Vec::new();
        let mut new_text = Vec::new();

        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone() ), &Value::Text(old_text.clone() )), @r###"
        Ok(
            [],
        )
        "###);

        new_text.push("a".into());
        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone()), &Value::Text(old_text.clone())), @r###"
        Ok(
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
            ],
        )
        "###);

        old_text = new_text.clone();
        new_text[0] = "b".into();
        assert_debug_snapshot!(diff_values(&Value::Text(new_text.clone() ), &Value::Text(old_text.clone() )), @r###"
        Ok(
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
            ],
        )
        "###);

        old_text = new_text.clone();
        new_text.pop();
        assert_debug_snapshot!(diff_values(&Value::Text(new_text), &Value::Text(old_text )), @r###"
        Ok(
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
            ],
        )
        "###);
    }

    #[test]
    fn new_and_empty() {
        let old = Value::Primitive(Primitive::Null);
        let mut hm = HashMap::new();
        hm.insert("a".into(), Value::Primitive(Primitive::Uint(2)));
        let new = Value::Map(hm);

        assert_debug_snapshot!(diff_values(&new , &old), @r###"
        Ok(
            [
                LocalChange {
                    path: Path(
                        [],
                    ),
                    operation: Set(
                        Map(
                            {
                                "a": Primitive(
                                    Uint(
                                        2,
                                    ),
                                ),
                            },
                        ),
                    ),
                },
            ],
        )
        "###);
    }

    #[test]
    fn empty_and_new() {
        let new = Value::Primitive(Primitive::Null);
        let mut hm = HashMap::new();
        hm.insert("a".into(), Value::Primitive(Primitive::Uint(2)));
        let old = Value::Map(hm);

        assert_debug_snapshot!(diff_values(&new , &old), @r###"
        Ok(
            [
                LocalChange {
                    path: Path(
                        [],
                    ),
                    operation: Set(
                        Primitive(
                            Null,
                        ),
                    ),
                },
            ],
        )
        "###);
    }
}
