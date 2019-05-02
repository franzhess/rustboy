extern crate zip;

mod mbc0;
mod mbc1;
mod mbc3;
mod mbc5;

use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
use crate::mbc::mbc0::Mbc0;
use crate::mbc::mbc1::Mbc1;
use std::io::Cursor;

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

pub fn load_rom(file_name: &str) -> Box<Mbc+'static> {
  let mut buffer = vec![];
  let mut f = File::open(file_name).unwrap();
  f.read_to_end(&mut buffer).expect("Error reading ROM!");

  if file_name.ends_with(".zip") {
    buffer = extract_rom_from_zip(buffer).unwrap();
  }

  match buffer[ADDR_CARTRIDGE_TYPE] {
    0x00 => Box::new(Mbc0::new(buffer)),
    0x01...0x03 => Box::new(Mbc1::new(buffer)),
    //0x05...0x06 => "MBC2",
    //0x0F...0x13 => "MBC3",
    //0x19...0x1E => "MBC5",
    v => panic!("Unsupported cartridge type {:#02X}", v)
  }
}

fn extract_rom_from_zip(buffer: Vec<u8>) -> Result<Vec<u8>, &'static str> {
  let mut archive = ZipArchive::new(Cursor::new(buffer)).expect("Failed to read ZIP file!");

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    if file.name().ends_with(".gb") {
      let mut result = vec![];
      file.read_to_end(&mut result).expect("Failed to extract rom from zip file!");
      return Ok(result)
    }
  }
  Err("No rom found in zip file!")
}

