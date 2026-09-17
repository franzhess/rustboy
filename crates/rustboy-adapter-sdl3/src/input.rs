use ::sdl3::event::Event;
use ::sdl3::keyboard::Keycode;
use ::sdl3::{EventPump, Sdl};
use rustboy_application::RunState;
use rustboy_core::{Button, ButtonEvent, ButtonState};

pub struct Input {
    event_pump: EventPump,
    events: Vec<ButtonEvent>,
}

impl Input {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        Ok(Self {
            event_pump: sdl.event_pump().map_err(|error| error.to_string())?,
            events: Vec::new(),
        })
    }

    pub fn poll(&mut self) -> RunState {
        self.events.clear();
        for event in self.event_pump.poll_iter() {
            let (state, keycode) = match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return RunState::Quit,
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => (ButtonState::Released, keycode),
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => (ButtonState::Pressed, keycode),
                _ => continue,
            };
            let button = match keycode {
                Keycode::Up => Button::Up,
                Keycode::Down => Button::Down,
                Keycode::Left => Button::Left,
                Keycode::Right => Button::Right,
                Keycode::A => Button::A,
                Keycode::S => Button::B,
                Keycode::Space => Button::Select,
                Keycode::Return => Button::Start,
                _ => continue,
            };
            self.events.push(ButtonEvent { button, state });
        }
        RunState::Running
    }

    pub fn drain(&mut self) -> Vec<ButtonEvent> {
        std::mem::take(&mut self.events)
    }
}
