use cpal::{
    traits::{EventLoopTrait, HostTrait},
    Format, SampleFormat, SampleRate, StreamData, UnknownTypeOutputBuffer,
};
use sfxr::{Generator, Sample, WaveType};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const LIGHT_PROJECTILE_VOLUME: f32 = 0.25;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const LIGHT_PROJECTILE_BASE_FREQ: f64 = 0.12;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const LIGHT_PROJECTILE_ATTACK_DURATION: f32 = 0.01;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const LIGHT_PROJECTILE_SUSTAIN_DURATION: f32 = 0.005;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const LIGHT_PROJECTILE_DECAY_DURATION: f32 = 0.14;

#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const HEAVY_PROJECTILE_VOLUME: f32 = 1.0;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const HEAVY_PROJECTILE_BASE_FREQ: f64 = 0.15;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const HEAVY_PROJECTILE_ATTACK_DURATION: f32 = 0.01;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const HEAVY_PROJECTILE_SUSTAIN_DURATION: f32 = 0.005;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const HEAVY_PROJECTILE_DECAY_DURATION: f32 = 0.14;

#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const UNIT_HIT_VOLUME: f32 = 0.8;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const UNIT_HIT_BASE_FREQ: f64 = 0.12;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const UNIT_HIT_ATTACK_DURATION: f32 = 0.01;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const UNIT_HIT_SUSTAIN_DURATION: f32 = 0.005;
#[const_tweaker::tweak(min = 0.0, max = 1.0, step = 0.001)]
const UNIT_HIT_DECAY_DURATION: f32 = 0.14;

/// Manages the audio.
#[derive(Default)]
pub struct Audio {
    generator: Arc<Mutex<Option<Generator>>>,
}

impl Audio {
    /// Instantiate a new audio object without a generator.
    pub fn new() -> Self {
        Self {
            generator: Arc::new(Mutex::new(None)),
        }
    }

    /// Play a sound for a light projectile hitting the ground.
    pub fn play_light_projectile(&self) {
        let mut sample = Sample::new();

        sample.wave_type = WaveType::Sine;
        sample.base_freq = *LIGHT_PROJECTILE_BASE_FREQ;
        sample.env_attack = *LIGHT_PROJECTILE_ATTACK_DURATION;
        sample.env_sustain = *LIGHT_PROJECTILE_SUSTAIN_DURATION;
        sample.env_decay = *LIGHT_PROJECTILE_DECAY_DURATION;

        self.play(sample, *LIGHT_PROJECTILE_VOLUME);
    }

    /// Play a sound for a heavy projectile hitting the ground.
    pub fn play_heavy_projectile(&self) {
        let mut sample = Sample::new();

        sample.wave_type = WaveType::Sine;
        sample.base_freq = *HEAVY_PROJECTILE_BASE_FREQ;
        sample.env_attack = *HEAVY_PROJECTILE_ATTACK_DURATION;
        sample.env_sustain = *HEAVY_PROJECTILE_SUSTAIN_DURATION;
        sample.env_decay = *HEAVY_PROJECTILE_DECAY_DURATION;

        self.play(sample, *HEAVY_PROJECTILE_VOLUME);
    }

    /// Play a sound when a unit is hit.
    pub fn play_unit_hit(&self) {
        let mut sample = Sample::new();

        sample.wave_type = WaveType::Sine;
        sample.base_freq = *UNIT_HIT_BASE_FREQ;
        sample.env_attack = *UNIT_HIT_ATTACK_DURATION;
        sample.env_sustain = *UNIT_HIT_SUSTAIN_DURATION;
        sample.env_decay = *UNIT_HIT_DECAY_DURATION;

        self.play(sample, *UNIT_HIT_VOLUME);
    }

    /// Play a sample.
    pub fn play(&self, sample: Sample, volume: f32) {
        let mut new_generator = Generator::new(sample);
        new_generator.volume = volume;

        let mut generator = self.generator.lock().unwrap();
        *generator = Some(new_generator);
    }

    /// Start a thread which will emit the audio.
    pub fn run(&mut self) {
        let generator = self.generator.clone();

        thread::spawn(|| {
            // Setup the audio system
            let host = cpal::default_host();
            let event_loop = host.event_loop();

            let device = host
                .default_output_device()
                .expect("no output device available");

            // This is the only format sfxr supports
            let format = Format {
                channels: 1,
                sample_rate: SampleRate(44_100),
                data_type: SampleFormat::F32,
            };

            let stream_id = event_loop
                .build_output_stream(&device, &format)
                .expect("could not build output stream");

            event_loop
                .play_stream(stream_id)
                .expect("could not play stream");

            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };

                match stream_data {
                    StreamData::Output {
                        buffer: UnknownTypeOutputBuffer::F32(mut buffer),
                    } => match *generator.lock().unwrap() {
                        Some(ref mut generator) => generator.generate(&mut buffer),
                        None => {
                            for elem in buffer.iter_mut() {
                                *elem = 0.0;
                            }
                        }
                    },
                    _ => panic!("output type buffer can not be used"),
                }
            });
        });
    }
}
