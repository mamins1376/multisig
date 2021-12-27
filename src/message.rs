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
    pub shape: WaveShape,
    pub amplitude_db: f64,
    pub frequency: f64,
    pub phase_degrees: f64,
}

impl Default for ChannelParams {
    fn default() -> Self {
        let frequency = 1e3;
        let (shape, amplitude_db, phase_degrees) = Default::default();
        Self {
            shape,
            amplitude_db,
            frequency,
            phase_degrees,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WaveShape {
    Sine,
    Triangle,
    Square(f64),
    Sawtooth,
}

impl WaveShape {
    pub fn name(&self) -> &'static str {
        match self {
            WaveShape::Sine => "Sine",
            WaveShape::Triangle => "Triangle",
            WaveShape::Square(_) => "Square",
            WaveShape::Sawtooth => "Sawtooth",
        }
    }
}

impl Default for WaveShape {
    fn default() -> Self {
        Self::Sine
    }
}
