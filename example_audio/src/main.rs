use mini_sdl::*;
use std::io::{Error, ErrorKind::NotFound};
use std::{f64::consts::TAU, i16};

fn main() -> SdlResult<()> {
    let mut app = mini_sdl::App::new(
        "audio-test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
        Some(44100),
    )?;

    let Some(sample_rate) = app.audio_mixrate() else {
        return Err(Box::new(Error::new(NotFound, "No Audio device")));
    };

    let rate = sample_rate as f64;
    let frequency = 440.0; // 440Hz, frequency of note A4
    let period = 1.0 / frequency;
    let time_per_sample = 1.0 / rate; // Time duration of one sample
    let mut accumulated_phase = 0.0;

    app.audio_start()?;

    while !app.quit_requested {
        app.frame_start()?;
        // Process audio
        let mut samples = Vec::new();
        if let Some(samples_per_frame) = app.audio_samples_per_frame() {
            // Sine wave generation
            for _ in 0..samples_per_frame {
                let phase = (accumulated_phase % period) / period;
                let sine_value = (TAU * phase).sin();
                // Push stereo samples
                let value = (sine_value * 0.25 * i16::MAX as f64) as i16;
                samples.push(value); // left
                samples.push(value); // right
                accumulated_phase += time_per_sample;
            }
            // Copy new samples to app's audio buffer and reset samples container
            app.audio_push_samples(samples.as_slice())?;
        };
        // Always call "start" and "finish" frame!
        app.frame_finish()?;
    }
    Ok(())
}
