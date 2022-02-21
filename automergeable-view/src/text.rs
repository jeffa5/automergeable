mod historic;
mod immutable;
mod mutable;

pub use historic::HistoricTextView;
pub use immutable::TextView;
pub use mutable::MutableTextView;

#[cfg(test)]
mod tests {
    use crate::MutableDoc;
    use crate::ViewableDoc;
    use automerge::Automerge;
    use automerge::Value;

    #[test]
    fn test_text() {
        let mut doc = Automerge::new();
        let mut tx = doc.transaction();
        tx.view_mut().insert("a", Value::text());
        tx.view_mut().get_mut("a").unwrap().insert(0, "b");
        tx.view_mut().get_mut("a").unwrap().insert(1, "c");
        tx.commit();

        let text = doc.view().get("a").unwrap().into_text().unwrap();

        assert_eq!(text.get(0), Some("b".into()));

        assert_eq!(text.len(), 2);

        assert!(!text.is_empty());

        assert_eq!(
            text.iter().collect::<Vec<_>>(),
            vec!["b".to_string(), "c".to_string()]
        );
    }
}
