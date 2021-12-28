use crate::message::Message;

use super::Engine;

#[derive(Default)]
pub struct NativeEngine;

impl Engine for NativeEngine {
    fn is_running(&mut self) -> bool {
        todo!()
    }

    fn sample_rate(&mut self) -> f64 {
        todo!()
    }

    fn signal(&mut self, _message: Message) -> crate::Result<()> {
        todo!()
    }

    fn run(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}
