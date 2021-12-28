use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    SetParams(usize, ChannelParams),
    Reset,
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
