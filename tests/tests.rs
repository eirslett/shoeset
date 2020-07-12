extern crate js_sys;
extern crate wasm_bindgen_test;
extern crate shoeset;
extern crate wasm_bindgen;

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn pass_tests() {
    use js_sys;

    let bytes = include_bytes!("foobar.7z");

    let result = shoeset::decompress(bytes).expect("Should be success");
    let files: js_sys::Array = result.files();

    assert_eq!(files.length(), 2);
}
