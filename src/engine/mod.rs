#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

use crate::{Result, message::Message};

pub trait Engine {
    fn is_running(&mut self) -> bool;

    fn sample_rate(&mut self) -> f64;

    fn signal(&mut self, message: Message) -> Result<()>;

    fn run(&mut self);

    fn stop(&mut self);
}
