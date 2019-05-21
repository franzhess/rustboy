pub struct Noise {
  enabled: bool
}

impl Noise {
  pub fn new() -> Noise {
    Noise {
      enabled: false
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    0
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      _ => ()
    }
  }
}