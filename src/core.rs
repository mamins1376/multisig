#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
pub struct Processor {
    channels: Vec<Channel>,
    // TODO: write directly to output buffer when feasible
    buffer: Vec<f32>,
}

impl Processor {
    pub fn message(&mut self, message: Message) {
        use Message::*;

        match message {
            SetParams(i, p) => self.channels[i] = p.into(),
            Reset => self.channels.iter_mut().for_each(Channel::reset),
        }
    }

    pub fn process(
        &mut self,
        n_channels: usize,
        n_samples: usize,
        rate: f32,
    ) -> &[f32] {
        self.channels.resize_with(n_channels, Default::default);

        let len = n_channels * n_samples;
        if self.buffer.len() < len {
            self.buffer.resize(len, 0.);
        }

        self.channels
            .iter_mut()
            .zip(self.buffer.chunks_exact_mut(n_samples))
            .for_each(|(c, b)| c.process(b, rate));

        &self.buffer[..len]
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum Message {
    SetParams(usize, ChannelParams),
    Reset,
}

#[derive(Default, Debug)]
pub struct Channel {
    params: ChannelParams,
    state: ChannelState,
}

impl Channel {
    pub fn process(&mut self, buf: &mut [f32], rate: f32) {
        self.state.process(buf, &self.params, rate)
    }

    pub fn reset(&mut self) {
        self.state = Default::default();
    }
}

impl From<ChannelParams> for Channel {
    fn from(params: ChannelParams) -> Self {
        let state = Default::default();
        Self { params, state }
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Default, Debug)]
struct ChannelState {
    t: u32,
}

impl ChannelState {
    fn process(&mut self, buf: &mut [f32], params: &ChannelParams, rate: f32) {
        use std::f64::consts::TAU;

        let rate = rate as f64;
        let amp = 10f64.powf(params.amplitude_db / 20.);
        let theta = params.phase_degrees.to_radians();

        // TODO: optimize this
        let w = params.frequency * TAU / rate;
        for b in buf {
            let p = (self.t as f64).mul_add(w, theta);
            self.t += 1;

            // i hope the compiler moves this out of the loop
            let form = match params.shape {
                WaveShape::Sine => p.sin(),
                WaveShape::Triangle => match (p / TAU).fract() * 4. {
                    p if p < 1. => p,
                    p if p < 3. => 2. - p,
                    p => p - 4.,
                },
                WaveShape::Square(d) => {
                    if (p / TAU).fract() < d {
                        1.
                    } else {
                        -1.
                    }
                }
                WaveShape::Sawtooth => (p / TAU).fract().mul_add(-2., 1.),
            };

            *b = (form * amp) as f32;
        }
    }
}
