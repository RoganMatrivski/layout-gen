use layout_gen::layout::parse_layout;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn basic() {
    parse_layout(r#"<layout></layout>"#).unwrap();
}

#[wasm_bindgen_test]
fn flex() {
    parse_layout(r#"<layout><flex grow="1"></flex></layout>"#).unwrap();
}
