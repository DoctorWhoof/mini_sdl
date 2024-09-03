use sdl2::audio::AudioCallback;
use std::collections::VecDeque;


/// A single audio sample, with left and right channels.
#[derive(Debug, Default, Clone, Copy)]
pub struct StereoFrame {
    pub left: i16,
    pub right: i16,
}

/// Used internally to push audio samples to the audio device. Although you can access it directly,
/// it requires locking the audio device so it's easier to use 'App::audio_push_samples' instead.
pub struct AudioInput {
    buffer:  VecDeque<StereoFrame>,
    last_frame: StereoFrame,
    frame_count: usize,
}

impl AudioInput {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::default(),
            last_frame: StereoFrame::default(),
            frame_count: 0,
        }
    }

    /// For debugging purposes, returns lengh of internal buffer
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Push a slice of StereoFrames. Ideally you should call this only once per frame,
    /// with all the samples that you need for that frame.
    #[inline(always)]
    pub fn push_samples(&mut self, frames: &[StereoFrame]) {
        for sample in frames.iter(){
            self.buffer.push_back(*sample);
        }
    }
}

impl<'a> AudioCallback for AudioInput {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        for x in out.iter_mut() {
            if self.frame_count % 2 == 0 {
                self.last_frame = self.buffer.pop_front().unwrap_or_default();
                *x = self.last_frame.left;
            } else {
                *x = self.last_frame.right;
            }
            self.frame_count += 1;
        }
    }
}
