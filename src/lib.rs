extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

pub mod internal;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Archive{
    id: String
}

#[wasm_bindgen]
impl Archive {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
}

#[wasm_bindgen]
pub fn decompress(data: &[u8]) -> Result<Archive, JsValue> {
    let result = internal::decompress_internal(data);

    return match result {
        Ok(res) => Ok(Archive{
            id: res.id
        }),
        Err(e) => Err(JsValue::from_str(&e.message))
    };
}
