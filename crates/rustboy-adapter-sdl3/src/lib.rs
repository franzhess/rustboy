mod display;
mod input;
mod sound;

use crate::display::Display;
use crate::input::Input;
use crate::sound::Sound;
use rustboy_application::{AudioSink, FrameSink, InputSource, RunState};
use rustboy_core::ButtonEvent;

pub struct Sdl3Adapter {
    input: Input,
    display: Display,
    sound: Sound,
}

impl Sdl3Adapter {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let sdl = ::sdl3::init().map_err(|error| error.to_string())?;
        Ok(Self {
            input: Input::new(&sdl)?,
            display: Display::new(&sdl, width, height)?,
            sound: Sound::new(&sdl)?,
        })
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.sound.play()
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.sound.stop()
    }
}

impl InputSource for Sdl3Adapter {
    fn poll_input(&mut self) -> Result<RunState, String> {
        Ok(self.input.poll())
    }

    fn drain_input(&mut self) -> Vec<ButtonEvent> {
        self.input.drain()
    }
}

impl FrameSink for Sdl3Adapter {
    fn present_frame(&mut self, frame: Vec<u8>) -> Result<(), String> {
        self.display.draw_screen(frame)
    }
}

impl AudioSink for Sdl3Adapter {
    fn queue_audio(&mut self, samples: Vec<i16>) -> Result<(), String> {
        self.sound.queue(samples)
    }
}
