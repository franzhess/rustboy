mod input;
mod display;
mod sound;

use crate::hardware::input::Input;
use crate::hardware::display::Display;

pub fn init_hardware(width:u32, height: u32) -> (Input, Display) {
  let sdl_context = sdl2::init().expect("Failed to init SDL2!");

  (
    Input::new(&sdl_context),
    Display::new(&sdl_context, width, height)
  )
}