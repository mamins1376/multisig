#![recursion_limit = "1024"]

mod app;
mod message;
mod worklet;

pub(crate) type Result<T> = std::result::Result<T, wasm_bindgen::JsValue>;
