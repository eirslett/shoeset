extern crate js_sys;
extern crate wasm_bindgen_test;
extern crate sjuz;

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn pass_tests() {
    let bytes = include_bytes!("foobar.7z");

    let result = sjuz::decompress(bytes).expect("Should be success");
    assert_eq!(result.id(), "Test archive");
}
