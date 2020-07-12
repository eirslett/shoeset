extern crate wasm_bindgen;
extern crate js_sys;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use js_sys::Array;
use js_sys::Uint8Array;

pub mod internal;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct File {
    name: String,
    data: js_sys::Uint8Array,
}

#[wasm_bindgen]
impl File {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        self.data.clone()
    }
}

#[wasm_bindgen]
pub struct Archive{
    files: js_sys::Array
}

#[wasm_bindgen]
impl Archive {
    #[wasm_bindgen(getter)]
    pub fn files(&self) -> js_sys::Array {
        self.files.clone()
    }
}

#[wasm_bindgen]
pub fn decompress(data: &[u8]) -> Result<Archive, JsValue> {
    let res = internal::decompress(data).or_else(|e| Err(JsValue::from_str(&e.message)))?;

    let files = Array::new();
    for file in res.files {
        let buf: &[u8] = &file.data;
        let data = Uint8Array::from(buf);
        let f = File {
            name: file.name,
            data
        };

        files.push(&JsValue::from(f));
    }

    Ok(Archive{
        files,
    })
}
