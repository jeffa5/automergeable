use automergeable::{self, automerge, Automergeable};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn make_change() {
    #[derive(Debug, Default, Clone, Automergeable)]
    pub struct DocumentInner {
        tasks: u8,
    }

    let mut doc = automergeable::Document::<DocumentInner>::new_with_timestamper(Box::new(|| None));
    let _change_result = doc
        .change::<_, automerge::InvalidChangeRequest>(|_d| Ok(()))
        .unwrap();
}
