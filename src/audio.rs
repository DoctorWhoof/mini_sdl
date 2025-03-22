use sdl3::audio::{AudioCallback, AudioStream};
// use std::collections::VecDeque;

// /// A single audio sample, with left and right channels.
// #[derive(Debug, Default, Clone, Copy)]
pub struct StereoFrame {
    pub left: i16,
    pub right: i16,
}

/// Used internally to push audio samples to the audio device. Although you can access it directly,
/// it requires locking the audio device so it's easier to use 'App::audio_push_samples' instead.
pub struct AudioInput {
    buffer: Vec<i16>,
    // last_frame: StereoFrame,
    // head: usize,
    // device_sample_count: u16,
}

impl AudioInput {
    pub fn new() -> Self {
        Self {
            buffer: Vec::default(),
            // device_sample_count,
        }
    }

    /// For debugging purposes, returns lengh of internal buffer
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Push a single StereoFrame to the buffer.
    #[inline(always)]
    pub fn push_sample(&mut self, frame: StereoFrame) {
        self.buffer.push(frame.left);
        self.buffer.push(frame.right);
    }

    /// Push a slice of StereoFrames. Ideally you should call this only once per frame,
    /// with all the samples that you need for that frame.
    #[inline(always)]
    pub fn push_samples(&mut self, frames: &[StereoFrame]) {
        for sample in frames.iter() {
            self.buffer.push(sample.left);
            self.buffer.push(sample.right);
        }
    }
}

impl<'a> AudioCallback<i16> for AudioInput {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let len = self.buffer.len();
        if len == 0 || requested == 0{
            return;
        }
        // stream
        //     .put_data_i16(&self.buffer.as_slice()[(len - requested as usize)..len])
        //     .unwrap();
        stream
            .put_data_i16(self.buffer.as_slice())
            .unwrap();
        self.buffer.clear();
    }
}
