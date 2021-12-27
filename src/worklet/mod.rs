use js_sys::{global, Array, Float32Array, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    AudioWorkletGlobalScope, Blob, BlobPropertyBag, MessageEvent, Url,
};

use std::{
    f32::consts::TAU,
    ops::{Add, Deref},
};

use crate::{
    message::{ChannelParams, Message},
    Result,
};

// TODO: sed s,MessagePort,SharedArrayBuffer,g
#[wasm_bindgen]
pub struct Processor {
    channels: Vec<Channel>,
    buffer: [f32; 128],
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Processor {
        let channels = Default::default();
        let buffer = [0f32; 128];
        Processor { channels, buffer }
    }

    #[wasm_bindgen]
    pub fn message(&mut self, message: MessageEvent) -> Result<()> {
        use Message::*;

        match Message::receive(message)? {
            SetParams(i, p) => self.channels[i] = p.into(),
            Reset => self.channels.iter_mut().for_each(Channel::reset),
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn process(&mut self, args: Array) -> bool {
        let scope: AudioWorkletGlobalScope = JsValue::from(global()).into();
        let dp = TAU / scope.sample_rate();

        let mut count = 0;

        for output in args.get(1).unchecked_into::<Array>().iter() {
            for buffer in output.unchecked_into::<Array>().iter() {
                if self.channels.len() == count {
                    self.channels.push(Default::default());
                }

                self.channels[count].process(&mut self.buffer, dp);

                buffer
                    .unchecked_into::<Float32Array>()
                    .copy_from(&self.buffer);

                count += 1
            }
        }

        self.channels.truncate(count);

        count != 0
    }
}

#[derive(Default)]
struct Channel {
    params: ChannelParams,
    state: ChannelState,
}

impl Channel {
    fn process(&mut self, buf: &mut [f32], dp: f32) {
        self.state.process(buf, &self.params, dp)
    }

    fn reset(&mut self) {
        self.state = Default::default();
    }
}

impl From<ChannelParams> for Channel {
    fn from(params: ChannelParams) -> Self {
        let state = Default::default();
        Self { params, state }
    }
}

#[derive(Default)]
struct ChannelState {
    phase: f32,
}

impl ChannelState {
    fn process(&mut self, buf: &mut [f32], params: &ChannelParams, dp: f32) {
        let dp = params.frequency * dp;
        let offset = params.phase.to_radians();
        for b in buf {
            *b = self.phase.add(offset).sin();
            self.phase += dp;
        }
    }
}

pub struct WorkletUrl(String);

impl WorkletUrl {
    pub fn create() -> Result<Self> {
        static CODER_JS: &'static [u8] = include_bytes!("coder.js");
        static INDEX_JS: &'static [u8] = include_bytes!("index.js");

        let blob = {
            // SAFETY: we never mutate the buffer, and it's 'static as well.
            let parts = unsafe {
                let coder = Uint8Array::view(CODER_JS);
                let glue = Reflect::get(&global(), &"glue".into())?;
                let index = Uint8Array::view(INDEX_JS);
                Array::of3(&coder, &glue, &index)
            };

            let mut props = BlobPropertyBag::new();
            props.type_("application/javascript");
            Blob::new_with_u8_array_sequence_and_options(&parts, &props)?
        };

        Ok(Self(Url::create_object_url_with_blob(&blob)?))
    }
}

impl Deref for WorkletUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for WorkletUrl {
    fn drop(&mut self) {
        let _ = Url::revoke_object_url(&self.0);
    }
}
