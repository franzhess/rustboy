use crate::mbc::Mbc;

pub struct Mbc0 {
  rom: Vec<u8>
}

impl Mbc0 {
  pub fn new(buffer: Vec<u8>) -> Mbc0 {
    Mbc0 {
      rom: buffer
    }
  }
}

impl Mbc for Mbc0 {
  fn read_rom(&self, address: u16) -> u8 {
    self.rom[address as usize]
  }

  fn read_ram(&self, address: u16) -> u8 {
    0
  }

  fn write_rom(&mut self, address: u16, value: u8) {
    ()
  }

  fn write_ram(&mut self, address: u16, value: u8) {
    ()
  }
}