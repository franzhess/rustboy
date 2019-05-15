
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;
use core::GBEvent;
use std::sync::mpsc::Sender;
use core::GBKeyEvent;
use core::GBKeyState;
use core::GBKeyCode;

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
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { //match keydown escape first so it has priority
          self.input_sender.send(GBEvent::Quit).expect("failed to send input to emulator");
          return false
        },
        Event::KeyUp { keycode:Some(Keycode::Up), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Up })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Up), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Up })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::Down), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Down })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Down), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Down })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::Left), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Left })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Left), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Left })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::Right), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Right })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Right), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Right })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::A), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::A })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::A), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::A })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::S), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::B })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::S), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::B })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::Space), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Select })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Space), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Select })).unwrap(),
        Event::KeyUp { keycode:Some(Keycode::Return), .. } =>  self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyUp, key_code: GBKeyCode::Start })).unwrap(),
        Event::KeyDown { keycode:Some(Keycode::Return), .. } => self.input_sender.send(GBEvent::KeyEvent(GBKeyEvent { state: GBKeyState::KeyDown, key_code: GBKeyCode::Start })).unwrap(),
        _ => {}
      }
    }

    true
  }
}