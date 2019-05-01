pub struct Serial {
  last_byte_written: u8,
  other_byte: u8
}

impl Serial {
  pub fn new() -> Serial {
    Serial {
      last_byte_written: 0,
      other_byte: 0
    }
  }

  pub fn read(&self, address:u16) -> u8 {
    match address {
      0xFF01 => self.last_byte_written,
      0xFF02 => self.other_byte,
      _ => 0
    }
  }

  pub fn write(&mut self, address:u16, value: u8) {
    match address {
      0xFF01 => self.last_byte_written = value,
      0xFF02 => if value == 0x81 { print!("{}", self.last_byte_written as char); self.other_byte = value; },
      _ => ()
    }
  }
}