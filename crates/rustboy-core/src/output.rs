//! Owned output formats shared by the deterministic core and host adapters.
//!
//! Conversion validates the format while retaining the original Vec allocation.
//! Only immutable slices are exposed, so consumers cannot invalidate the format.

use crate::{AUDIO_CHANNELS, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::error::Error;
use std::fmt;

/// A complete DMG frame: 160×144 row-major shade indices, top-left first.
/// Each byte is a shade from 0 (lightest) to 3 (darkest), not an RGB value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame(Vec<u8>);

impl Frame {
    pub const PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

    /// Pixel (x, y) is at `y * SCREEN_WIDTH + x`.
    pub fn pixels(&self) -> &[u8] {
        &self.0
    }

    pub fn into_pixels(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<Vec<u8>> for Frame {
    type Error = FrameError;

    fn try_from(pixels: Vec<u8>) -> Result<Self, Self::Error> {
        if pixels.len() != Self::PIXEL_COUNT {
            return Err(FrameError::InvalidPixelCount {
                actual: pixels.len(),
            });
        }
        if let Some((index, &shade)) = pixels.iter().enumerate().find(|(_, shade)| **shade > 3) {
            return Err(FrameError::InvalidShade { index, shade });
        }
        Ok(Self(pixels))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    InvalidPixelCount { actual: usize },
    InvalidShade { index: usize, shade: u8 },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPixelCount { actual } => write!(
                f,
                "frame has {actual} pixels; expected {}",
                Frame::PIXEL_COUNT
            ),
            Self::InvalidShade { index, shade } => {
                write!(f, "frame pixel {index} has shade {shade}; expected 0..=3")
            }
        }
    }
}

impl Error for FrameError {}

/// Signed 16-bit PCM at AUDIO_OUTPUT_FREQUENCY stereo frames/second.
/// Samples are interleaved `[left, right, left, right, ...]`; empty buffers are
/// valid, but partial stereo frames are not. Values are i16 samples, not bytes
/// with a prescribed byte order. Buffer length is independent of video frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBuffer(Vec<i16>);

impl AudioBuffer {
    pub fn samples(&self) -> &[i16] {
        &self.0
    }

    /// Number of complete left/right pairs, not the number of individual samples.
    pub fn frame_count(&self) -> usize {
        self.0.len() / AUDIO_CHANNELS
    }

    pub fn into_samples(self) -> Vec<i16> {
        self.0
    }
}

impl TryFrom<Vec<i16>> for AudioBuffer {
    type Error = AudioBufferError;

    fn try_from(samples: Vec<i16>) -> Result<Self, Self::Error> {
        if samples.len() % AUDIO_CHANNELS != 0 {
            return Err(AudioBufferError {
                sample_count: samples.len(),
            });
        }
        Ok(Self(samples))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioBufferError {
    pub sample_count: usize,
}

impl fmt::Display for AudioBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "stereo audio requires an even sample count; got {}",
            self.sample_count
        )
    }
}

impl Error for AudioBufferError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_requires_exactly_one_screen_of_pixels() {
        for length in [0, Frame::PIXEL_COUNT - 1, Frame::PIXEL_COUNT + 1] {
            assert_eq!(
                Frame::try_from(vec![0; length]),
                Err(FrameError::InvalidPixelCount { actual: length })
            );
        }
    }

    #[test]
    fn frame_rejects_out_of_range_shades_with_their_location() {
        for shade in 4..=u8::MAX {
            let mut pixels = vec![0; Frame::PIXEL_COUNT];
            pixels[SCREEN_WIDTH + 3] = shade;
            assert_eq!(
                Frame::try_from(pixels),
                Err(FrameError::InvalidShade {
                    index: SCREEN_WIDTH + 3,
                    shade
                })
            );
        }
    }

    #[test]
    fn valid_frame_preserves_pixel_order_and_vec_storage() {
        let mut pixels = Vec::with_capacity(Frame::PIXEL_COUNT + 16);
        pixels.extend((0..Frame::PIXEL_COUNT).map(|index| (index % 4) as u8));
        let pointer = pixels.as_ptr();
        let capacity = pixels.capacity();
        let frame = Frame::try_from(pixels).expect("valid shades and dimensions");
        assert_eq!(frame.pixels().as_ptr(), pointer);
        assert_eq!(frame.pixels().len(), Frame::PIXEL_COUNT);
        for (index, &shade) in frame.pixels().iter().enumerate() {
            assert_eq!(shade, (index % 4) as u8);
        }
        let pixels = frame.into_pixels();
        assert_eq!(pixels.as_ptr(), pointer);
        assert_eq!(pixels.capacity(), capacity);
    }

    #[test]
    fn audio_rejects_incomplete_stereo_frames() {
        for sample_count in [1, 3, 1599] {
            assert_eq!(
                AudioBuffer::try_from(vec![0; sample_count]),
                Err(AudioBufferError { sample_count })
            );
        }
    }

    #[test]
    fn audio_preserves_signed_samples_channel_order_and_vec_storage() {
        let mut samples = Vec::with_capacity(16);
        samples.extend([i16::MIN, i16::MAX, -1, 1]);
        let pointer = samples.as_ptr();
        let capacity = samples.capacity();
        let buffer = AudioBuffer::try_from(samples).expect("two stereo frames");
        assert_eq!(buffer.frame_count(), 2);
        assert_eq!(buffer.samples(), [i16::MIN, i16::MAX, -1, 1]);
        assert_eq!(buffer.samples().as_ptr(), pointer);
        let samples = buffer.into_samples();
        assert_eq!(samples.as_ptr(), pointer);
        assert_eq!(samples.capacity(), capacity);
    }

    #[test]
    fn empty_audio_is_zero_complete_frames() {
        let buffer = AudioBuffer::try_from(Vec::new()).expect("empty stereo buffer");
        assert_eq!(buffer.frame_count(), 0);
        assert!(buffer.samples().is_empty());
    }
}
