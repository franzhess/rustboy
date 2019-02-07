pub enum GBKeyCode {
  Up = 0,
  Down,
  Left,
  Right,
  A,
  B,
  Start,
  Select
}

pub struct Joypad {
  pub irq_joypad: bool,

  state: [bool;8],
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
      joypad |= 0x20;
      if self.state[GBKeyCode::A as usize] { joypad |= 0x01 };
      if self.state[GBKeyCode::B as usize] { joypad |= 0x02 };
      if self.state[GBKeyCode::Select as usize] { joypad |= 0x04 };
      if self.state[GBKeyCode::Start as usize] { joypad |= 0x08 };
    } else {
      joypad |= 0x10;
      if self.state[GBKeyCode::Right as usize] { joypad |= 0x01 };
      if self.state[GBKeyCode::Left as usize] { joypad |= 0x02 };
      if self.state[GBKeyCode::Up as usize] { joypad |= 0x04 };
      if self.state[GBKeyCode::Down as usize] { joypad |= 0x08 };
    }

    !joypad
  }

  pub fn write(&mut self, value: u8) {
    if (!value & 0x20) == 0x20 { self.selector = true; }
    else if (!value & 0x10) == 0x10 { self.selector = false; }
  }

  pub fn set_state(&mut self, new_state: [bool; 8]) {
    for i in 0..8 {
      self.irq_joypad |= !self.state[i] && new_state[i]; //we fire an irq when a button was pressed
    }

    self.state = new_state;
  }
}