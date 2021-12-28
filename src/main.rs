#![cfg_attr(target_arch = "wasm32", no_main)]

mod app;
mod engine;
mod message;

#[cfg(target_arch = "wasm32")]
type Error = wasm_bindgen::JsValue;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount() -> Result<()> {
    console_error_panic_hook::set_once();

    type App = Box<app::App<engine::web::WebEngine>>;
    eframe::start_web("app", App::default())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    type App = Box<app::App<engine::native::NativeEngine>>;
    eframe::run_native(App::default(), Default::default());
}
