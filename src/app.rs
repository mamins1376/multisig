use std::{
    cell::{Cell, Ref, RefCell},
    future::Future,
    rc::Rc,
};

use eframe::{
    egui::{CentralPanel, ComboBox, CtxRef, Slider, Ui},
    epi::Frame,
};
use js_sys::{global, Array, Reflect};
use wasm_bindgen::{prelude::*, throw_val};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    AudioContext, AudioContextState, AudioWorkletNode, AudioWorkletNodeOptions,
};

use crate::{
    message::{ChannelParams, Message, WaveShape},
    worklet::WorkletUrl,
    Result,
};

#[wasm_bindgen]
pub fn mount() -> Result<()> {
    console_error_panic_hook::set_once();

    let app = Box::new(App::default());
    eframe::start_web("app", app)?;

    Ok(())
}

#[derive(Default)]
struct App {
    engine: Rc<Engine>,
    params: [ChannelParams; 2],
}

impl App {
    fn draw(&mut self, ui: &mut Ui) {
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
                self.engine
                    .signal(Message::SetParams(i, param.clone()))
                    .unwrap()
            }
        }

        if self.engine.is_running() {
            self.engine_button(ui, "Stop", |e| async move { e.stop().await })
        } else {
            self.engine_button(ui, "Start", |e| async move { e.run().await })
        }
    }

    fn engine_button<C, F>(&self, ui: &mut Ui, label: &str, clicked: C)
    where
        C: FnOnce(Rc<Engine>) -> F + 'static,
        F: Future<Output = Result<()>>,
    {
        if ui.button(label).clicked() {
            let engine = self.engine.clone();
            spawn_local(async move {
                clicked(engine)
                    .await
                    .map_err(throw_val)
                    .unwrap_or_else(|i| i)
            })
        }
    }
}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        "App"
    }

    fn update(&mut self, ctx: &CtxRef, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| self.draw(ui));
    }
}

#[derive(Default)]
struct Engine {
    context: RefCell<Option<AudioContext>>,
    worklet: RefCell<Option<AudioWorkletNode>>,
    loaded: Cell<bool>,
}

impl Engine {
    fn is_running(&self) -> bool {
        self.context
            .borrow()
            .as_ref()
            .map(|x| x.state() == AudioContextState::Running)
            .unwrap_or(false)
    }

    fn sample_rate(&self) -> f64 {
        self.context
            .borrow()
            .as_ref()
            .map(|x| x.sample_rate() as f64)
            .unwrap_or(48000.)
    }

    fn signal(&self, message: Message) -> Result<()> {
        if self.is_running() {
            message.send(&self.worklet()?.port()?)
        } else {
            Ok(())
        }
    }

    async fn run(&self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        self.load().await?;

        let context = self.context()?;
        JsFuture::from(context.resume()?).await?;

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        let context = self.context()?;
        let suspended = context.suspend()?;
        JsFuture::from(suspended).await?;

        Ok(())
    }

    async fn load(&self) -> Result<()> {
        if self.loaded.get() {
            return Ok(());
        }

        let context = self.context()?;
        let worklet = context.audio_worklet()?;

        let url = WorkletUrl::create()?;
        let loaded = worklet.add_module(&url)?;
        JsFuture::from(loaded).await?;

        self.loaded.set(true);

        let wasm = Reflect::get(&global(), &"wasm".into())?;
        let port = self.worklet()?.port()?;
        port.post_message(&wasm)?;

        Ok(())
    }

    fn context(&self) -> Result<Ref<AudioContext>> {
        let mut context = self.context.borrow();

        if context.is_none() {
            drop(context);
            *self.context.borrow_mut() = Some(AudioContext::new()?);
            context = self.context.borrow();
        }

        Ok(Ref::map(context, |c| c.as_ref().unwrap()))
    }

    fn worklet(&self) -> Result<Ref<AudioWorkletNode>> {
        let mut worklet = self.worklet.borrow();

        if worklet.is_none() {
            drop(worklet);

            assert!(self.loaded.get());

            let context = self.context()?;
            let speakers = context.destination();

            {
                let count = speakers.max_channel_count();
                let array = Array::of1(&count.into());

                let mut options = AudioWorkletNodeOptions::new();
                options.output_channel_count(&array);

                *self.worklet.borrow_mut() =
                    AudioWorkletNode::new_with_options(
                        &context,
                        "wasm-processor",
                        &options,
                    )?
                    .into();
            }

            worklet = self.worklet.borrow();
            worklet
                .as_ref()
                .unwrap()
                .connect_with_audio_node(&speakers)?;
        }

        Ok(Ref::map(worklet, |c| c.as_ref().unwrap()))
    }
}
