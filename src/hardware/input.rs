use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;
use crate::cpu::mmu::joypad::GBKeyCode;

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
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => { self.keys[GBKeyCode::Up as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Up), .. } => { self.keys[GBKeyCode::Up as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => { self.keys[GBKeyCode::Down as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Down), .. } => { self.keys[GBKeyCode::Down as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => { self.keys[GBKeyCode::Left as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Left), .. } => { self.keys[GBKeyCode::Left as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => { self.keys[GBKeyCode::Right as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Right), .. } => { self.keys[GBKeyCode::Right as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::A), .. } => { self.keys[GBKeyCode::A as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::A), .. } => { self.keys[GBKeyCode::A as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::S), .. } => { self.keys[GBKeyCode::B as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::S), .. } => { self.keys[GBKeyCode::B as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::Return), .. } => { self.keys[GBKeyCode::Start as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Return), .. } => { self.keys[GBKeyCode::Start as usize] = false; },
        Event::KeyDown { keycode: Some(Keycode::Space), .. } => { self.keys[GBKeyCode::Select as usize] = true; },
        Event::KeyUp { keycode: Some(Keycode::Space), .. } => { self.keys[GBKeyCode::Select as usize] = false; },
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return Err("Esc"),
        _ => {}
      }
    }

    Ok(self.keys.clone())
  }
}