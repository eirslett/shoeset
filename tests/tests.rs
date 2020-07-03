extern crate wasm_bindgen_test;
extern crate sjuz;

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn pass() {
    let bytes = include_bytes!("foobar.7z");

    let result = sjuz::decompress(bytes);
    assert_eq!(result.expect("Should be success").id(), "Test archive");
}
