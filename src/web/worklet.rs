use bincode::deserialize;
use js_sys::{global, Array, Float32Array, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    AudioWorkletGlobalScope, Blob, BlobPropertyBag, MessageEvent, Url,
};

use super::Result;

#[wasm_bindgen]
#[derive(Default)]
pub struct Processor {
    inner: crate::core::Processor,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Processor {
        Self::default()
    }

    #[wasm_bindgen]
    pub fn message(&mut self, event: MessageEvent) -> Result<()> {
        let data: Uint8Array = event.data().dyn_into()?;
        let message = deserialize(data.to_vec().as_ref())
            .map_err(|e| format!("malformed message received: {}", e))?;
        self.inner.message(message);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn process(&mut self, args: Array) -> bool {
        let output = args
            .get(1)
            .unchecked_into::<Array>()
            .get(0)
            .unchecked_into::<Array>();

        let channels: Vec<Float32Array> =
            output.iter().map(JsValue::unchecked_into).collect();

        let n_samples = match channels.first() {
            Some(b) => b.length() as usize,
            None => return false,
        };

        let scope: AudioWorkletGlobalScope = JsValue::from(global()).into();
        let rate = scope.sample_rate();
        let buf = self.inner.process(channels.len(), n_samples, rate);

        buf.chunks_exact(n_samples)
            .zip(channels)
            .for_each(|(b, a)| a.copy_from(b));

        true
    }
}

pub struct WorkletUrl(String);

impl WorkletUrl {
    pub fn create() -> Result<Self> {
        static CODER_JS: &[u8] = include_bytes!("coder.js");
        static INDEX_JS: &[u8] = include_bytes!("index.js");

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

impl std::ops::Deref for WorkletUrl {
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
