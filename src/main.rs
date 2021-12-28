#![recursion_limit = "1024"]

#![cfg_attr(target_arch = "wasm32", no_main)]

mod app;
mod message;
mod worklet;

pub(crate) type Result<T> = std::result::Result<T, wasm_bindgen::JsValue>;
