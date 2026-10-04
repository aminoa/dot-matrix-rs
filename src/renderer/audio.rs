use std::eprintln;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::*, HeapCons, HeapProd, HeapRb};

pub struct AudioRenderer {
    pub stream: cpal::Stream,
    pub sample_rate: f32,
}

impl AudioRenderer {
    pub fn new() -> Result<(AudioRenderer, HeapProd<f32>), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let Some(device) = host.default_output_device() else {
            return Err("Error: no output device".into());
        };
        let config: cpal::StreamConfig = match device.default_output_config() {
            Ok(config) => config.into(),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        };

        let channels = config.channels as usize;

        // 1 second of buffer
        let rb = HeapRb::<f32>::new(config.sample_rate as usize / 5);
        let (producer, mut consumer): (HeapProd<f32>, HeapCons<f32>) = rb.split();

        let stream = device
            .build_output_stream(
                config,
                move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // interleaved LR sample buffer (each frame is one LR)
                    for frame in out.chunks_mut(channels) {
                        // writing samples into left/right channels (only mono output currently)
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

        let sample_rate = config.sample_rate as f32;
        Ok((AudioRenderer { stream, sample_rate }, producer))
    }
}
