mod input;
mod display;
mod sound;

use crate::hardware::input::Input;
use crate::hardware::display::Display;
use std::sync::mpsc::Sender;
use core::GBEvent;

pub fn init_hardware(width:u32, height: u32, input_sender: Sender<GBEvent>) -> (Input, Display) {
  let sdl_context = sdl2::init().expect("Failed to init SDL2!");

  (
    Input::new(&sdl_context, input_sender),
    Display::new(&sdl_context, width, height)
  )
}