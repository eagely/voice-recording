use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::transcription_client::TranscriptionClient;

pub struct Recorder {
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    transcription_client: TranscriptionClient,
}

impl Recorder {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .input_devices()
            .map_err(|_| Error::NoInputDevice)?
            .find(|d| d.name().map(|n| n == "pipewire").unwrap_or(false))
            .ok_or(Error::NoInputDevice)?;

        let sample_rate = 16000;
        let channels = 1;

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buffer) = buffer_clone.lock() {
                    buffer.extend_from_slice(data);
                }
            },
            move |err| {
                eprintln!("An error occurred on stream: {}", err);
            },
            None,
        )?;

        let transcription_client = TranscriptionClient::default();

        Ok(Self {
            stream,
            buffer,
            sample_rate,
            channels,
            transcription_client,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.stream.play()?;
        Ok(())
    }

    fn create_wav_data(&self) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let recorded_data = self.buffer.lock().unwrap().clone();
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for &sample in &recorded_data {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        Ok(cursor.into_inner())
    }

    pub fn stop(&mut self) -> Result<String> {
        let wav_data = self.create_wav_data()?;
        self.transcription_client.transcribe(wav_data)
    }
}
