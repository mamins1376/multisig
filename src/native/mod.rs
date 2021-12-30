use std::{
    ffi::c_void,
    ptr::{null, NonNull},
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
    thread::spawn,
};

use crate::{
    app::App,
    core::{Message, Processor},
};

extern "C" {
    fn multisig_pw_main(inner: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn multisig_process(
    inner: NonNull<c_void>,
    n_channels: usize,
    n_samples: usize,
    rate: u32,
) -> *const f32 {
    let inner = inner.cast::<EngineInner>().as_mut();
    inner.process(n_channels, n_samples, rate)
}

pub fn main() {
    let (app, options): (Box<App>, _) = Default::default();
    eframe::run_native(app, options)
}

#[derive(Default)]
pub struct Engine {
    port: Option<SyncSender<Message>>,
    rate: Arc<AtomicU32>,
}

impl Engine {
    pub fn is_running(&mut self) -> bool {
        self.port.is_some()
    }

    pub fn sample_rate(&mut self) -> f64 {
        self.rate.load(Ordering::Relaxed) as _
    }

    pub fn signal(&mut self, message: Message) {
        self.port
            .as_mut()
            .map(|p| p.send(message).unwrap())
            .unwrap_or(())
    }

    pub fn run(&mut self) {
        if self.is_running() {
            return;
        }

        let (sender, port) = sync_channel(16);
        let rate = self.rate.clone();
        spawn(move || {
            let processor = Default::default();
            let mut inner = EngineInner {
                rate,
                port,
                processor,
            };
            let inner: *mut EngineInner = &mut inner;
            unsafe { multisig_pw_main(inner.cast()) }
        });

        self.port = Some(sender)
    }

    pub fn stop(&mut self) {
        self.port = None
    }
}

struct EngineInner {
    rate: Arc<AtomicU32>,
    port: Receiver<Message>,
    processor: Processor,
}

impl EngineInner {
    fn process(
        &mut self,
        n_channels: usize,
        n_samples: usize,
        rate: u32,
    ) -> *const f32 {
        loop {
            use std::sync::mpsc::TryRecvError::*;

            match self.port.try_recv() {
                Ok(m) => self.processor.message(m),
                Err(Empty) => break,
                Err(Disconnected) => return null(),
            }
        }

        self.rate.store(rate, Ordering::Relaxed);
        let rate = rate as f32;
        self.processor.process(n_channels, n_samples, rate).as_ptr()
    }
}
