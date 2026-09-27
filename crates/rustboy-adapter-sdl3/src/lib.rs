mod display;
mod error;
mod input;
mod sound;

use crate::display::{Display, DisplayContext};
use crate::input::Input;
use crate::sound::Sound;
pub use error::InitError;
use rustboy_application::{AudioSink, FrameSink, InputSource, RunState};
use rustboy_core::{AudioBuffer, ButtonEvent, Frame};

/// Owns SDL and the renderer/texture creator borrowed by a connected adapter.
pub struct Sdl3Context {
    sdl: sdl3::Sdl,
    display: DisplayContext,
}

impl Sdl3Context {
    pub fn new(width: u32, height: u32) -> Result<Self, InitError> {
        let sdl = sdl3::init()?;
        let display = DisplayContext::new(&sdl, width, height)?;
        Ok(Self { sdl, display })
    }
}

pub struct Sdl3Adapter<'a> {
    input: Input,
    display: Display<'a>,
    sound: Sound,
}

impl<'a> Sdl3Adapter<'a> {
    pub fn new(context: &'a mut Sdl3Context) -> Result<Self, InitError> {
        Ok(Self {
            input: Input::new(&context.sdl)?,
            display: context.display.connect()?,
            sound: Sound::new(&context.sdl)?,
        })
    }

    pub fn play(&mut self) -> Result<(), sdl3::Error> {
        self.sound.play()
    }

    pub fn stop(&mut self) -> Result<(), sdl3::Error> {
        self.sound.stop()
    }
}

impl InputSource for Sdl3Adapter<'_> {
    type Error = std::convert::Infallible;

    fn poll_input(&mut self) -> Result<RunState, Self::Error> {
        Ok(self.input.poll())
    }

    fn drain_input(&mut self) -> Vec<ButtonEvent> {
        self.input.drain()
    }
}

impl FrameSink for Sdl3Adapter<'_> {
    type Error = sdl3::Error;

    fn present_frame(&mut self, frame: &Frame) -> Result<(), Self::Error> {
        self.display.draw_screen(frame)
    }
}

impl AudioSink for Sdl3Adapter<'_> {
    type Error = sdl3::Error;

    fn queue_audio(&mut self, samples: &AudioBuffer) -> Result<(), Self::Error> {
        self.sound.queue(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "SDL lifecycle smoke test: use SDL_VIDEO_DRIVER=dummy SDL_AUDIO_DRIVER=dummy"]
    fn context_can_create_and_drop_adapters_with_streaming_output() {
        let mut context = Sdl3Context::new(320, 288).expect("SDL context");
        let frame = Frame::try_from(vec![2; Frame::PIXEL_COUNT]).expect("valid frame");
        let audio = AudioBuffer::try_from(vec![0; 1600]).expect("valid stereo buffer");
        for _ in 0..2 {
            let mut adapter = Sdl3Adapter::new(&mut context).expect("SDL adapter");
            adapter.play().expect("start audio");
            adapter.present_frame(&frame).expect("streaming frame");
            adapter.queue_audio(&audio).expect("queue audio");
            adapter.stop().expect("stop audio");
            // Drop the texture and event pump before borrowing the context again.
        }
    }
}
