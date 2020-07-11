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
    id: String,
    files: js_sys::Array
}

#[wasm_bindgen]
impl Archive {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

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
        // let data = unsafe { js_sys::Uint8Array::view(&file.data) };
        let buf: &[u8] = &file.data;
        let data = Uint8Array::from(buf);
        let f = File {
            name: file.name,
            data
        };

        /*
        let obj = js_sys::Object::new();
        js_sys::Object::define_property(&obj, &JsValue::from_str("foo"), "bar");
        */

        // obj.set("name", &JsValue::from_str(&file.name.to_string()));
        files.push(&JsValue::from(f));
    }


    Ok(Archive{
        id: res.id,
        files,
    })

        /*
    return match result {

        Err(e) => Err(JsValue::from_str(&e.message))
    };
    */

}
