use eframe::{
    egui::{CentralPanel, ComboBox, CtxRef, Slider},
    epi::Frame,
};

use crate::{
    core::{ChannelParams, Message, WaveShape},
    platform::Engine,
};

#[derive(Default)]
pub struct App {
    engine: Engine,
    params: [ChannelParams; 2],
}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        "App"
    }

    fn update(&mut self, ctx: &CtxRef, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let rate = self.engine.sample_rate();
            let nyquist = rate / 2.;

            ui.heading("Multi-Channel Signal Generator");

            for (i, param) in self.params.iter_mut().enumerate() {
                let mut set = false;

                let slider = Slider::new(&mut param.frequency, 0.0..=nyquist)
                    //.logarithmic(true)
                    .suffix("Hz")
                    .text(format!("Channel #{} Freq", i + 1));
                set |= ui.add(slider).changed();

                let slider = Slider::new(&mut param.phase_degrees, 0.0..=360.)
                    .suffix("°")
                    .text(format!("Channel #{} Freq", i + 1));
                set |= ui.add(slider).changed();

                ComboBox::from_label(format!("Channel #{} WaveShape", i + 1))
                    .selected_text(param.shape.name())
                    .show_ui(ui, |ui| {
                        let mut f = |v: WaveShape| {
                            let t = v.name();
                            set |= ui
                                .selectable_value(&mut param.shape, v, t)
                                .changed()
                        };

                        f(WaveShape::Sine);
                        f(WaveShape::Triangle);
                        f(WaveShape::Square(0.5));
                        f(WaveShape::Sawtooth);
                    });

                if set {
                    self.engine.signal(Message::SetParams(i, param.clone()))
                }
            }

            let (label, act) = match self.engine.is_running() {
                false => ("Run", Engine::run as _),
                true => ("Stop", Engine::stop as fn(&mut Engine)),
            };
            if ui.button(label).clicked() {
                act(&mut self.engine)
            }
        });
    }
}
