mod input;
mod display;
mod sound;

use std::sync::mpsc::Sender;
use core::GBEvent;
use crate::sdl::input::Input;
use crate::sdl::display::Display;
use crate::sdl::sound::Sound;

pub fn init_hardware(width:u32, height: u32, input_sender: Sender<GBEvent>) -> (Input, Display, Sound) {
  let sdl_context = sdl2::init().expect("Failed to init SDL2!");

  (
    Input::new(&sdl_context, input_sender),
    Display::new(&sdl_context, width, height),
    Sound::new(&sdl_context),
  )
}