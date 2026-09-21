use ::sdl3::event::Event;
use ::sdl3::gamepad::{Button as GamepadButton, Gamepad};
use ::sdl3::joystick::JoystickId;
use ::sdl3::keyboard::Keycode;
use ::sdl3::{EventPump, GamepadSubsystem, Sdl};
use rustboy_application::RunState;
use rustboy_core::{Button, ButtonEvent, ButtonState};

pub struct Input {
    event_pump: EventPump,
    gamepad_subsystem: Option<GamepadSubsystem>,
    // SDL only delivers button events for opened gamepads; retain the handle.
    gamepad: Option<Gamepad>,
    buttons: Buttons,
}

const GAMEPAD_BUTTONS: [(GamepadButton, Button); 8] = [
    (GamepadButton::DPadUp, Button::Up),
    (GamepadButton::DPadDown, Button::Down),
    (GamepadButton::DPadLeft, Button::Left),
    (GamepadButton::DPadRight, Button::Right),
    (GamepadButton::South, Button::A),
    (GamepadButton::East, Button::B),
    (GamepadButton::Back, Button::Select),
    (GamepadButton::Start, Button::Start),
];

impl Input {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let gamepad_subsystem = match sdl.gamepad() {
            Ok(subsystem) => Some(subsystem),
            Err(error) => {
                eprintln!("Gamepad support unavailable; keyboard input remains available: {error}");
                None
            }
        };
        let mut input = Self {
            event_pump: sdl.event_pump().map_err(|error| error.to_string())?,
            gamepad_subsystem,
            gamepad: None,
            buttons: Buttons::default(),
        };
        input.open_gamepad();
        Ok(input)
    }

    fn open_gamepad(&mut self) {
        if self.gamepad.is_some() {
            return;
        }
        let Some(subsystem) = &self.gamepad_subsystem else {
            return;
        };
        if !subsystem.has_gamepad() {
            return;
        }
        let ids = match subsystem.gamepads() {
            Ok(ids) => ids,
            Err(error) => {
                eprintln!("Could not list gamepads: {error}");
                return;
            }
        };
        for id in ids {
            match subsystem.open(id) {
                Ok(gamepad) => {
                    self.buttons.gamepad_id = Some(id);
                    // Include buttons already held when a controller is selected.
                    for (physical, button) in GAMEPAD_BUTTONS {
                        self.buttons
                            .set(Source::Gamepad, button, gamepad.button(physical));
                    }
                    self.gamepad = Some(gamepad);
                    break;
                }
                Err(error) => eprintln!("Could not open gamepad {id}: {error}"),
            }
        }
    }

    pub fn poll(&mut self) -> RunState {
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return RunState::Quit,
                Event::GamepadAdded { .. } => self.open_gamepad(),
                Event::GamepadRemoved { which, .. } => {
                    if self.buttons.disconnect_gamepad(which) {
                        self.gamepad = None;
                        self.open_gamepad();
                    }
                }
                _ => self.buttons.handle_event(event),
            }
        }
        RunState::Running
    }

    pub fn drain(&mut self) -> Vec<ButtonEvent> {
        std::mem::take(&mut self.buttons.events)
    }
}

#[derive(Clone, Copy)]
enum Source {
    Keyboard,
    Gamepad,
}

#[derive(Default)]
struct Buttons {
    keyboard: [bool; 8],
    gamepad: [bool; 8],
    gamepad_id: Option<JoystickId>,
    events: Vec<ButtonEvent>,
}

impl Buttons {
    fn handle_event(&mut self, event: Event) {
        let pressed = matches!(
            event,
            Event::KeyDown { .. } | Event::GamepadButtonDown { .. }
        );
        let (source, button) = match event {
            Event::KeyDown {
                keycode: Some(key), ..
            }
            | Event::KeyUp {
                keycode: Some(key), ..
            } => {
                let button = match key {
                    Keycode::Up => Button::Up,
                    Keycode::Down => Button::Down,
                    Keycode::Left => Button::Left,
                    Keycode::Right => Button::Right,
                    Keycode::A => Button::A,
                    Keycode::S => Button::B,
                    Keycode::Space => Button::Select,
                    Keycode::Return => Button::Start,
                    _ => return,
                };
                (Source::Keyboard, button)
            }
            Event::GamepadButtonDown { which, button, .. }
            | Event::GamepadButtonUp { which, button, .. }
                if self.gamepad_id == Some(which) =>
            {
                let Some((_, button)) = GAMEPAD_BUTTONS
                    .iter()
                    .find(|(physical, _)| *physical == button)
                else {
                    return;
                };
                (Source::Gamepad, *button)
            }
            _ => return,
        };
        self.set(source, button, pressed);
    }

    fn set(&mut self, source: Source, button: Button, pressed: bool) {
        let index = button as usize;
        let before = self.keyboard[index] || self.gamepad[index];
        match source {
            Source::Keyboard => self.keyboard[index] = pressed,
            Source::Gamepad => self.gamepad[index] = pressed,
        }
        // A release from one device must not cancel another device's held button.
        let after = self.keyboard[index] || self.gamepad[index];
        if before != after {
            self.events.push(ButtonEvent {
                button,
                state: if after {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                },
            });
        }
    }

    fn disconnect_gamepad(&mut self, id: JoystickId) -> bool {
        if self.gamepad_id != Some(id) {
            return false;
        }
        self.gamepad_id = None;
        for (_, button) in GAMEPAD_BUTTONS {
            self.set(Source::Gamepad, button, false);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connected() -> Buttons {
        Buttons {
            gamepad_id: Some(JoystickId::new(1)),
            ..Buttons::default()
        }
    }

    fn gamepad_event(id: u32, button: GamepadButton, pressed: bool) -> Event {
        let which = JoystickId::new(id);
        if pressed {
            Event::GamepadButtonDown {
                timestamp: 0,
                which,
                button,
            }
        } else {
            Event::GamepadButtonUp {
                timestamp: 0,
                which,
                button,
            }
        }
    }

    fn keyboard_event(pressed: bool) -> Event {
        if pressed {
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::A),
                scancode: None,
                keymod: ::sdl3::keyboard::Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            }
        } else {
            Event::KeyUp {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::A),
                scancode: None,
                keymod: ::sdl3::keyboard::Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            }
        }
    }

    fn events(buttons: &mut Buttons) -> Vec<(usize, ButtonState)> {
        std::mem::take(&mut buttons.events)
            .into_iter()
            .map(|event| (event.button as usize, event.state))
            .collect()
    }

    #[test]
    fn gamepad_buttons_generate_press_and_release_edges() {
        let mut buttons = connected();
        for (physical, expected) in [
            (GamepadButton::South, Button::A),
            (GamepadButton::East, Button::B),
            (GamepadButton::DPadUp, Button::Up),
            (GamepadButton::DPadDown, Button::Down),
            (GamepadButton::DPadLeft, Button::Left),
            (GamepadButton::DPadRight, Button::Right),
            (GamepadButton::Start, Button::Start),
            (GamepadButton::Back, Button::Select),
        ] {
            buttons.handle_event(gamepad_event(1, physical, true));
            buttons.handle_event(gamepad_event(1, physical, true));
            buttons.handle_event(gamepad_event(1, physical, false));
            assert_eq!(
                events(&mut buttons),
                vec![
                    (expected as usize, ButtonState::Pressed),
                    (expected as usize, ButtonState::Released),
                ]
            );
        }
    }

    #[test]
    fn either_device_can_release_without_cancelling_the_other_hold() {
        for release_keyboard_first in [false, true] {
            let mut buttons = connected();
            buttons.handle_event(keyboard_event(true));
            buttons.handle_event(gamepad_event(1, GamepadButton::South, true));
            assert_eq!(
                events(&mut buttons),
                vec![(Button::A as usize, ButtonState::Pressed)]
            );

            let release_keyboard = keyboard_event(false);
            let release_gamepad = gamepad_event(1, GamepadButton::South, false);
            let (first, last) = if release_keyboard_first {
                (release_keyboard, release_gamepad)
            } else {
                (release_gamepad, release_keyboard)
            };
            buttons.handle_event(first);
            assert!(events(&mut buttons).is_empty());
            buttons.handle_event(last);
            assert_eq!(
                events(&mut buttons),
                vec![(Button::A as usize, ButtonState::Released)]
            );
        }
    }

    #[test]
    fn disconnect_releases_controller_buttons_but_preserves_keyboard_input() {
        let mut buttons = connected();
        buttons.handle_event(keyboard_event(true));
        buttons.handle_event(gamepad_event(1, GamepadButton::South, true));
        buttons.handle_event(gamepad_event(1, GamepadButton::DPadUp, true));
        events(&mut buttons);

        assert!(!buttons.disconnect_gamepad(JoystickId::new(2)));
        assert!(events(&mut buttons).is_empty());
        assert!(buttons.disconnect_gamepad(JoystickId::new(1)));
        assert_eq!(
            events(&mut buttons),
            vec![(Button::Up as usize, ButtonState::Released)]
        );
        assert_eq!(buttons.gamepad_id, None);
        assert!(buttons.gamepad.iter().all(|pressed| !pressed));
        buttons.handle_event(keyboard_event(false));
        assert_eq!(
            events(&mut buttons),
            vec![(Button::A as usize, ButtonState::Released)]
        );
    }

    #[test]
    fn inactive_and_unmapped_gamepad_buttons_are_ignored() {
        let mut buttons = connected();
        buttons.handle_event(gamepad_event(2, GamepadButton::South, true));
        buttons.handle_event(gamepad_event(1, GamepadButton::Guide, true));
        assert!(events(&mut buttons).is_empty());

        buttons.disconnect_gamepad(JoystickId::new(1));
        buttons.gamepad_id = Some(JoystickId::new(2));
        buttons.handle_event(gamepad_event(2, GamepadButton::South, true));
        buttons.handle_event(gamepad_event(1, GamepadButton::South, false));
        assert_eq!(
            events(&mut buttons),
            vec![(Button::A as usize, ButtonState::Pressed)]
        );
        buttons.handle_event(gamepad_event(2, GamepadButton::South, false));
        assert_eq!(
            events(&mut buttons),
            vec![(Button::A as usize, ButtonState::Released)]
        );
    }
}
