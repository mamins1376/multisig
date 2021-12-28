#![cfg_attr(target_arch = "wasm32", no_main)]

mod app;
mod message;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native.rs"]
mod platform;

#[cfg(target_arch = "wasm32")]
#[path = "web/mod.rs"]
mod platform;

// bummer: main() must be in main.rs (otherwise E0601)
#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn main() {
    platform::main()
}
