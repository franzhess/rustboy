use crate::GBKeyCode;
use crate::GBKeyEvent;
use crate::GBKeyState;

pub struct Joypad {
  pub irq_joypad: bool, //interrupt is true when input has changed
  state: [bool;8], //the state of the 8 buttons
  selector: bool, //true = buttons, false = directions
}

impl Joypad {
  pub fn new() -> Joypad {
    Joypad {
      irq_joypad: false,
      state:[false; 8],
      selector: false
    }
  }

  /*
  Bit 7 - Not used
  Bit 6 - Not used
  Bit 5 - P15 Select Button Keys      (0=Select)
  Bit 4 - P14 Select Direction Keys   (0=Select)
  Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
  Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
  Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
  Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
  */

  pub fn read(&self) -> u8 {
    let mut joypad = 0x00;
    if self.selector {
      joypad |= 0x20; //P15
      if self.state[GBKeyCode::A as usize] { joypad |= 0x01 };
      if self.state[GBKeyCode::B as usize] { joypad |= 0x02 };
      if self.state[GBKeyCode::Select as usize] { joypad |= 0x04 };
      if self.state[GBKeyCode::Start as usize] { joypad |= 0x08 };
    } else {
      joypad |= 0x10; //P14
      if self.state[GBKeyCode::Right as usize] { joypad |= 0x01 };
      if self.state[GBKeyCode::Left as usize] { joypad |= 0x02 };
      if self.state[GBKeyCode::Up as usize] { joypad |= 0x04 };
      if self.state[GBKeyCode::Down as usize] { joypad |= 0x08 };
    }

    !joypad //the gameboy has the input array inverted
  }

  pub fn write(&mut self, value: u8) {
    if (!value & 0x20) == 0x20 { self.selector = true; }
    else if (!value & 0x10) == 0x10 { self.selector = false; }
  }

  pub fn receive_event(&mut self, event: GBKeyEvent) {
    let key_index = event.key_code as usize;
    self.irq_joypad |= !self.state[key_index] && event.state == GBKeyState::KeyDown;
    self.state[key_index] = match event.state {
      GBKeyState::KeyDown => true,
      GBKeyState::KeyUp => false
    };
  }
}