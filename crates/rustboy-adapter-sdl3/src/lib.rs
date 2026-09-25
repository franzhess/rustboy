mod display;
mod error;
mod input;
mod sound;

use crate::display::Display;
use crate::input::Input;
use crate::sound::Sound;
pub use error::InitError;
use rustboy_application::{AudioSink, FrameSink, InputSource, RunState};
use rustboy_core::{AudioBuffer, ButtonEvent, Frame};

pub struct Sdl3Adapter {
    input: Input,
    display: Display,
    sound: Sound,
}

impl Sdl3Adapter {
    pub fn new(width: u32, height: u32) -> Result<Self, InitError> {
        let sdl = ::sdl3::init()?;
        Ok(Self {
            input: Input::new(&sdl)?,
            display: Display::new(&sdl, width, height)?,
            sound: Sound::new(&sdl)?,
        })
    }

    pub fn play(&mut self) -> Result<(), sdl3::Error> {
        self.sound.play()
    }

    pub fn stop(&mut self) -> Result<(), sdl3::Error> {
        self.sound.stop()
    }
}

impl InputSource for Sdl3Adapter {
    type Error = std::convert::Infallible;

    fn poll_input(&mut self) -> Result<RunState, Self::Error> {
        Ok(self.input.poll())
    }

    fn drain_input(&mut self) -> Vec<ButtonEvent> {
        self.input.drain()
    }
}

impl FrameSink for Sdl3Adapter {
    type Error = sdl3::Error;

    fn present_frame(&mut self, frame: &Frame) -> Result<(), Self::Error> {
        self.display.draw_screen(frame)
    }
}

impl AudioSink for Sdl3Adapter {
    type Error = sdl3::Error;

    fn queue_audio(&mut self, samples: &AudioBuffer) -> Result<(), Self::Error> {
        self.sound.queue(samples)
    }
}
