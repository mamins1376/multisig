use bincode::{deserialize, serialize};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, MessagePort};

use super::Result;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    SetParams(usize, ChannelParams),
    Reset,
}

impl Message {
    pub fn receive(event: MessageEvent) -> Result<Self> {
        let data: Uint8Array = event.data().dyn_into()?;
        let message = deserialize(data.to_vec().as_ref())
            .map_err(|e| format!("malformed message received: {}", e))?;
        Ok(message)
    }

    pub fn send(&self, port: &MessagePort) -> Result<()> {
        let bytes = serialize(self)
            .map_err(|e| format!("serializing message failed: {}", e))?;
        let bytes = Uint8Array::from(bytes.as_slice());
        port.post_message(&bytes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelParams {
    pub frequency: f32,
    pub phase: f32,
}

impl Default for ChannelParams {
    fn default() -> Self {
        Self {
            frequency: 440.,
            phase: 0.,
        }
    }
}
