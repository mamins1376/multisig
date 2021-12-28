use crate::{app::App, message::Message};

pub fn main() {
    let (app, options): (Box<App>, _) = Default::default();
    eframe::run_native(app, options)
}

#[derive(Default)]
pub struct Engine;

impl Engine {
    pub fn is_running(&mut self) -> bool {
        todo!()
    }

    pub fn sample_rate(&mut self) -> f64 {
        todo!()
    }

    pub fn signal(&mut self, _message: Message) {
        todo!()
    }

    pub fn run(&mut self) {
        todo!()
    }

    pub fn stop(&mut self) {
        todo!()
    }
}
