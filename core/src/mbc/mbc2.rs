use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x200;

pub struct Mbc2 {
  rom: Vec<u8>,
  selected_rom_bank: usize,
  ram: [u8; RAM_BANK_SIZE],
  ram_enabled: bool,
}

impl Mbc2 {
  pub fn new(buffer: Vec<u8>) -> Mbc2 {
    Mbc2 {
      rom:buffer,
      selected_rom_bank: 1, //0 is mapped to 0000-3FFF
      ram: [0; RAM_BANK_SIZE ],
      ram_enabled: false,
    }
  }
}

impl Mbc for Mbc2 {
  fn read_rom(&self, address: u16) -> u8 {
    match address {
      0x0000 ..= 0x3FFF => self.rom[address as usize],
      0x4000 ..= 0x7FFF => self.rom[ROM_BANK_SIZE * self.selected_rom_bank + (address - 0x4000) as usize],
      _ => 0
    }
  }

  fn read_ram(&self, address: u16) -> u8 {
    if self.ram_enabled {
      self.ram[address as usize]
    } else {
      0
    }
  }

  fn write_rom(&mut self, address: u16, value: u8) {
    if address & 0x100 == 0x100 {
      match address {
        0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
        0x2000..=0x3FFF => self.selected_rom_bank = (value as usize) & 0xF,
        _ => ()
      }
    }
  }

  fn write_ram(&mut self, address: u16, value: u8) {
    if self.ram_enabled {
      self.ram[address as usize] = value;
    }
  }
}