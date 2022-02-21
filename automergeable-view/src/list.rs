mod historical;
mod immutable;
mod mutable;

pub use historical::HistoricalListView;
pub use immutable::ListView;
pub use mutable::MutableListView;

#[cfg(test)]
mod tests {
    use automerge::{Automerge, ScalarValue};
    use serde_json::json;

    use crate::{automerge_doc, View, ViewableDoc};

    #[test]
    fn test_list() {
        let doc = automerge_doc(json!({
            "a": [1, 2],
        }))
        .unwrap();

        let list = doc.view().get("a").unwrap().into_list().unwrap();

        assert_eq!(list.get(0), Some(View::Scalar(ScalarValue::Uint(1))));

        assert_eq!(list.len(), 2);

        assert!(!list.is_empty());

        let v: Vec<View<Automerge>> = vec![1u64.into(), 2u64.into()];
        assert_eq!(list.iter().collect::<Vec<_>>(), v);
    }
}
