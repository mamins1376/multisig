class ProcessorWrapper extends AudioWorkletProcessor {
  constructor() {
    super();

    this.port.onmessage = this.message.bind(this);
    this.port.onmessageerror = raise;

    this.queue = [];
  }

  message(event) {
    if (!this.inner) {
      if (!wasm) {
        if (initing) {
          this.queue.push(event);
        } else {
          init(event.data)
            .then(_ => this.message())
            .catch(raise);
          initing = true;
        }
        return;
      }

      this.inner = new Processor();
    }

    this.queue.push(event);
    while (event = this.queue.shift())
      this.inner.message(event);
  }

  process(...a) {
    return this.inner ? this.inner.process(a) : true;
  }

  free() {
    if (this.inner) this.inner.free();
  }
}

const raise = err => { throw err; };

let initing = false;

registerProcessor("wasm-processor", ProcessorWrapper);
