use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::*, HeapCons, HeapProd, HeapRb};

pub const SAMPLE_RATE: u32 = 48000;

pub struct AudioRenderer {
    pub stream: cpal::Stream,
}

impl AudioRenderer {
    pub fn new() -> (AudioRenderer, HeapProd<f32>) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Error: no output device");

        let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
        let channels = config.channels as usize;

        // ~200 ms of slack so UI-thread jitter doesn't starve the callback.
        let rb = HeapRb::<f32>::new(config.sample_rate as usize / 5 * channels.max(1));
        let (producer, mut consumer): (HeapProd<f32>, HeapCons<f32>) = rb.split();

        let stream = device
            .build_output_stream(
                config,
                move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // interleaved LR buffer
                    for frame in out.chunks_mut(channels) {
                        // writing samples into left/right channels
                        let s = consumer.try_pop().unwrap_or(0.0);
                        for slot in frame.iter_mut() {
                            *slot = s;
                        }
                    }
                },
                move |err| eprintln!("Error: audio stream error: {err}"),
                None,
            )
            .expect("Error: failed to build output stream");

        stream.play().expect("Error: failed to start stream");
        (AudioRenderer { stream }, producer)
    }
}
