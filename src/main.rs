#![cfg_attr(target_arch = "wasm32", no_main)]

mod app;
mod engine;
mod message;
mod worklet;

use app::App;

#[cfg(target_arch = "wasm32")]
type Error = wasm_bindgen::JsValue;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum Error {}

type Result<T> = std::result::Result<T, Error>;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount() -> Result<()> {
    console_error_panic_hook::set_once();
    let app = Box::new(App::default());
    eframe::start_web("app", app)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
