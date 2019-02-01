use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;

pub struct Input {
  event_pump: EventPump,
  keys: [bool; 8],
}

impl Input {
  pub fn new(sdl: &Sdl) -> Input {
    Input {
      event_pump: sdl.event_pump().unwrap(),
      keys: [false; 8],
    }
  }

  pub fn process_input(&mut self) -> Result<[bool; 8], &str> {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return Err("Esc"),
        _ => {}
      }
    }

    Ok(self.keys.clone())
  }
}