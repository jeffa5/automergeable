use std::convert::TryInto;

use automerge::{InvalidChangeRequest, LocalChange, Path, Primitive, Value};

/// Calculate the `LocalChange`s between the two values.
///
/// Recursively works from the root.
#[tracing::instrument(skip(new, old))]
pub fn diff_values(new: &Value, old: &Value) -> Result<Vec<LocalChange>, InvalidChangeRequest> {
    diff_with_path(new, old, Path::root())
}

#[tracing::instrument(skip(new, old))]
fn diff_with_path(
    new: &Value,
    old: &Value,
    path: Path,
) -> Result<Vec<LocalChange>, InvalidChangeRequest> {
    match (new, old) {
        (Value::Map(new_map, mt1), Value::Map(old_map, mt2)) if mt1 == mt2 => {
            let mut changes = Vec::new();
            for (k, v) in new_map {
                if let Some(old_v) = old_map.get(k) {
                    // changed
                    changes.append(&mut diff_with_path(v, old_v, path.clone().key(k))?)
                } else {
                    // new
                    changes.push(LocalChange::set(path.clone().key(k), v.clone()))
                }
            }
            for k in old_map.keys() {
                if !new_map.contains_key(k) {
                    // removed
                    changes.push(LocalChange::delete(path.clone().key(k)))
                }
            }
            Ok(changes)
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
                if i < old_vec.len() && *v != old_vec[i] {
                    // changed
                    changes.push(LocalChange::set(
                        path.clone().index(i.try_into().unwrap()),
                        Value::Primitive(Primitive::Str(v.to_string())),
                    ))
                } else {
                    // new
                    changes.push(LocalChange::insert(
                        path.clone().index(i.try_into().unwrap()),
                        Value::Primitive(Primitive::Str(v.to_string())),
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
            if new_string != old_string {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Str(new_string.to_owned())),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (Value::Primitive(Primitive::Int(new_int)), Value::Primitive(Primitive::Int(old_int))) => {
            if new_int != old_int {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Int(*new_int)),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (
            Value::Primitive(Primitive::Uint(new_int)),
            Value::Primitive(Primitive::Uint(old_int)),
        ) => {
            if new_int != old_int {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Uint(*new_int)),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (Value::Primitive(Primitive::F64(new_int)), Value::Primitive(Primitive::F64(old_int))) =>
        {
            #[allow(clippy::float_cmp)]
            if new_int != old_int {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::F64(*new_int)),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (Value::Primitive(Primitive::F32(new_int)), Value::Primitive(Primitive::F32(old_int))) =>
        {
            #[allow(clippy::float_cmp)]
            if new_int != old_int {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::F32(*new_int)),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (
            Value::Primitive(Primitive::Counter(new_int)),
            Value::Primitive(Primitive::Counter(old_int)),
        ) => {
            if new_int != old_int {
                if new_int > old_int {
                    let diff = if let Some(diff) = new_int.checked_sub(*old_int) {
                        diff
                    } else {
                        // TODO: perhaps change this behavior or change error type
                        return Err(InvalidChangeRequest::CannotOverwriteCounter { path });
                    };
                    let diff = diff.try_into();
                    if let Ok(diff) = diff {
                        Ok(vec![LocalChange::increment_by(path, diff)])
                    } else {
                        // TODO: change this once increment_by has larger values
                        Err(InvalidChangeRequest::CannotOverwriteCounter { path })
                    }
                } else {
                    // TODO: change this once counters can be decremented
                    Err(InvalidChangeRequest::CannotOverwriteCounter { path })
                }
            } else {
                Ok(Vec::new())
            }
        }
        (
            Value::Primitive(Primitive::Timestamp(new_int)),
            Value::Primitive(Primitive::Timestamp(old_int)),
        ) => {
            if new_int != old_int {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Timestamp(*new_int)),
                )])
            } else {
                Ok(Vec::new())
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
            if new_bool != old_bool {
                Ok(vec![LocalChange::set(
                    path,
                    Value::Primitive(Primitive::Boolean(*new_bool)),
                )])
            } else {
                Ok(Vec::new())
            }
        }
        (Value::Primitive(Primitive::Null), Value::Primitive(Primitive::Null)) => Ok(Vec::new()),
        // handle mismatch combinations
        (_, Value::Primitive(Primitive::Counter(_))) => {
            Err(InvalidChangeRequest::CannotOverwriteCounter { path })
        }
        (Value::Primitive(Primitive::Null), _) => Ok(vec![LocalChange::set(
            path,
            Value::Primitive(Primitive::Null),
        )]),
        (v, Value::Primitive(Primitive::Null)) => Ok(vec![LocalChange::set(path, v.clone())]),
        (n, _) => Ok(vec![LocalChange::set(path, n.clone())]),
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

        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @r###"
        Ok(
            [],
        )
        "###);

        new_map.insert(
            "abc".to_owned(),
            Primitive::Str("some val".to_owned()).into(),
        );
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @r###"
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
        new_map.insert(
            "abc".to_owned(),
            Primitive::Str("some newer val".to_owned()).into(),
        );
        assert_debug_snapshot!(diff_values(&Value::Map(new_map.clone(), MapType::Map), &Value::Map(old_map.clone(), MapType::Map)), @r###"
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
        assert_debug_snapshot!(diff_values(&Value::Map(new_map, MapType::Map), &Value::Map(old_map, MapType::Map)), @r###"
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

        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone() ), &Value::Sequence(old_vec.clone() )), @r###"
        Ok(
            [],
        )
        "###);

        new_vec.push(Primitive::Str("some val".to_owned()).into());
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone()), &Value::Sequence(old_vec.clone())), @r###"
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
        new_vec[0] = Primitive::Str("some newer val".to_owned()).into();
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec.clone() ), &Value::Sequence(old_vec.clone() )), @r###"
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
        assert_debug_snapshot!(diff_values(&Value::Sequence(new_vec), &Value::Sequence(old_vec )), @r###"
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

        new_text.push("a".to_owned());
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
        new_text[0] = "b".to_owned();
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
        hm.insert("a".to_owned(), Value::Primitive(Primitive::Uint(2)));
        let new = Value::Map(hm, MapType::Map);

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
                            Map,
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
        hm.insert("a".to_owned(), Value::Primitive(Primitive::Uint(2)));
        let old = Value::Map(hm, MapType::Map);

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
