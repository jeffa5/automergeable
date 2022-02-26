mod historical;
mod immutable;
mod mutable;

pub use historical::HistoricalMapView;
pub use immutable::MapView;
pub use mutable::MutableMapView;

#[cfg(test)]
mod tests {
    use crate::{MutableDoc, View};
    use automerge::{Automerge, ScalarValue, Value};
    use serde_json::json;

    use crate::{automerge_doc, ViewableDoc};

    #[test]
    fn test_map() {
        let doc = automerge_doc(json!({
            "a": 1,
            "b": 2,
        }))
        .unwrap();

        let root = doc.view();

        assert!(matches!(
            root.get("a"),
            Some(View::Scalar(ScalarValue::Uint(1)))
        ));

        assert!(matches!(
            root.get("b"),
            Some(View::Scalar(ScalarValue::Uint(2)))
        ));

        assert_eq!(root.len(), 2);

        assert!(!root.is_empty());

        assert!(root.contains_key("a"));

        assert!(!root.contains_key("c"));

        assert_eq!(root.keys().collect::<Vec<String>>(), vec!["a", "b"]);

        let v: Vec<View<Automerge>> = vec![1u64.into(), 2u64.into()];
        assert_eq!(root.values().collect::<Vec<_>>(), v);

        assert_eq!(
            root.iter().collect::<Vec<_>>(),
            vec![("a".to_owned(), 1u64.into()), ("b".to_owned(), 2u64.into())]
        );
    }

    #[test]
    fn test_map_mut() {
        let mut doc = automerge_doc(json!({
            "a": 1,
            "b": 2,
        }))
        .unwrap();

        let mut tx = doc.transaction();
        let mut root = tx.view_mut();

        assert!(matches!(
            root.get("a"),
            Some(View::Scalar(ScalarValue::Uint(1)))
        ));

        assert!(matches!(
            root.get("b"),
            Some(View::Scalar(ScalarValue::Uint(2)))
        ));

        assert_eq!(root.len(), 2);

        assert!(!root.is_empty());

        assert!(root.contains_key("a"));

        assert!(!root.contains_key("c"));

        assert_eq!(root.keys().collect::<Vec<String>>(), vec!["a", "b"]);

        let v: Vec<View<Automerge>> = vec![1u64.into(), 2u64.into()];
        assert_eq!(root.values().collect::<Vec<_>>(), v);

        assert_eq!(
            root.iter().collect::<Vec<_>>(),
            vec![("a".to_owned(), 1u64.into()), ("b".to_owned(), 2u64.into())]
        );

        root.insert("c", 5);

        assert_eq!(root.len(), 3);
        assert!(root.contains_key("c"));

        assert!(root.remove("a"));
        assert!(!root.remove("a"));
        assert_eq!(root.len(), 2);

        let imm = root.into_immutable();
        assert!(imm.contains_key("c"));
    }

    #[test]
    fn nested_map() {
        let mut doc = Automerge::new();
        let mut tx = doc.transaction();
        let mut root = tx.view_mut();

        root.insert("a", Value::map());
        let mut a = root.get_mut("a").unwrap().into_map_mut().unwrap();
        a.insert("b", 1);

        assert!(a.contains_key("b"));
    }
}
