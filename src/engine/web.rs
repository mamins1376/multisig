use std::{
    cell::{Cell, Ref, RefCell},
    future::Future,
    rc::Rc,
};

use js_sys::{global, Array, Reflect};
use wasm_bindgen::throw_val;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    AudioContext, AudioContextState, AudioWorkletNode, AudioWorkletNodeOptions,
};

use crate::{message::Message, worklet::WorkletUrl, Result};

#[derive(Default)]
pub struct Engine {
    inner: Rc<EngineInner>,
}

impl Engine {
    pub fn is_running(&mut self) -> bool {
        use AudioContextState::Running;
        self.context_map(|x| x.state() == Running, false)
    }

    pub fn sample_rate(&mut self) -> f64 {
        self.context_map(|x| x.sample_rate(), 48000.) as _
    }

    pub fn signal(&mut self, message: Message) -> Result<()> {
        message.send(&self.inner.worklet()?.port()?)
    }

    pub fn run(&mut self) {
        if !self.is_running() {
            self.spawn(|i| async move { i.run().await })
        }
    }

    pub fn stop(&mut self) {
        if self.is_running() {
            self.spawn(|e| async move { e.stop().await })
        }
    }

    fn context_map<T, F>(&self, f: F, default: T) -> T
    where
        F: FnOnce(&AudioContext) -> T,
    {
        self.inner
            .context
            .borrow()
            .as_ref()
            .map(f)
            .unwrap_or(default)
    }

    fn spawn<C, F>(&self, apply: C)
    where
        C: FnOnce(Rc<EngineInner>) -> F + 'static,
        F: Future<Output = Result<()>>,
    {
        let inner = self.inner.clone();
        spawn_local(async move {
            apply(inner).await.map_err(throw_val).unwrap_or_else(|i| i)
        })
    }
}

#[derive(Default)]
struct EngineInner {
    context: RefCell<Option<AudioContext>>,
    worklet: RefCell<Option<AudioWorkletNode>>,
    loaded: Cell<bool>,
}

impl EngineInner {
    async fn run(&self) -> Result<()> {
        self.load().await?;

        let context = self.context()?;
        JsFuture::from(context.resume()?).await?;

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
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
