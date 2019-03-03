use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;
use core::joypad::{GBKeyCode, GBKeyState, GBKeyEvent};
use core::GBEvent;
use std::sync::mpsc::Sender;

pub struct Input {
  event_pump: EventPump,
  input_sender: Sender<GBEvent>
}

impl Input {
  pub fn new(sdl: &Sdl, input_sender: Sender<GBEvent>) -> Input {
    Input {
      event_pump: sdl.event_pump().unwrap(),
      input_sender
    }
  }

  pub fn process_input(&mut self) -> bool {
    for event in self.event_pump.poll_iter() {
      match event {
        /*Event::KeyDown { keycode: Some(Keycode::Up), .. } => { self.keys[GBKeyCode::Up as usize] = true; },
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
        Event::KeyUp { keycode: Some(Keycode::Space), .. } => { self.keys[GBKeyCode::Select as usize] = false; },*/
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          self.input_sender.send(GBEvent::Quit);
          return false
        },
        Event::KeyUp { keycode:Some(keycode), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: key_mapping(keycode) })).unwrap(),
        Event::KeyDown { keycode:Some(keycode), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: key_mapping(keycode) })).unwrap(),
        _ => {}
      }
    }

    true
  }
}

fn key_mapping(keycode: Keycode) -> GBKeyCode {
  match keycode {
    Keycode::Up => GBKeyCode::Up,
    Keycode::Down => GBKeyCode::Down,
    Keycode::Left => GBKeyCode::Left,
    Keycode::Right => GBKeyCode::Right,
    Keycode::A => GBKeyCode::A,
    Keycode::B => GBKeyCode::B,
    Keycode::Space => GBKeyCode::Select,
    _ => GBKeyCode::Start,
  }
}