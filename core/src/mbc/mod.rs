mod mbc0;

use std::fs::File;
use std::io::Read;
use crate::mbc::mbc0::Mbc0;

const ADDR_TITLE_START: u16 = 0x0134;
const TITLE_SIZE: u16 = 16;
const ADDR_CARTRIDGE_TYPE: usize = 0x0147;

pub trait Mbc : Send {
  fn read_rom(&self, address: u16) -> u8;
  fn read_ram(&self, address: u16) -> u8;
  fn write_rom(&mut self, address: u16, value: u8);
  fn write_ram(&mut self, address: u16, value: u8);
  fn name(&self) -> String {
    let mut name = String::with_capacity(TITLE_SIZE as usize);

    for i in 0..TITLE_SIZE {
      match self.read_rom(ADDR_TITLE_START + i ) {
        0 => break,
        ch => name.push(ch as char)
      }
    }

    name
  }
}

pub fn load_rom(file_name: &str) -> Result<Box<Mbc+'static>, &str> {
  let mut buffer = vec![];
  let mut f = File::open(file_name).unwrap();
  f.read_to_end(&mut buffer);

  match buffer[ADDR_CARTRIDGE_TYPE] {
    0x00 => Ok(Box::new(Mbc0::new(buffer))),
    //0x01...0x03 => "MBC1",
    //0x0F...0x13 => "MBC3",
    //0x19...0x1E => "MBC5",
    _ => Err("Unsupported cartridge type")
  }
}