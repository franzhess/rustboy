use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;

const RAM_BANK_SIZE: usize = 0x2000;
const NUM_RAM_BANK: usize = 16;

pub struct Mbc5 {
  rom: Vec<u8>,
  selected_rom_bank: usize,
  ram: [u8; RAM_BANK_SIZE * NUM_RAM_BANK],
  ram_enabled: bool,
  selected_ram_bank: usize,
}

impl Mbc5 {
  pub fn new(buffer: Vec<u8>) -> Mbc5 {
    Mbc5 {
      rom:buffer,
      selected_rom_bank: 1, //0 is mapped to 0000-3FFF
      ram: [0; RAM_BANK_SIZE * NUM_RAM_BANK],
      selected_ram_bank: 0,
      ram_enabled: false,
    }
  }
}

impl Mbc for Mbc5 {
  fn read_rom(&self, address: u16) -> u8 {
    match address {
      0x0000 ... 0x3FFF => self.rom[address as usize],
      0x4000 ... 0x7FFF => self.rom[ROM_BANK_SIZE * self.selected_rom_bank + (address - 0x4000) as usize],
      _ => 0
    }
  }

  fn read_ram(&self, address: u16) -> u8 {
    if self.ram_enabled {
      self.ram[RAM_BANK_SIZE * self.selected_ram_bank + address as usize]
    } else {
      0
    }
  }

  fn write_rom(&mut self, address: u16, value: u8) {
    match address {
      0x0000 ... 0x1FFF => self.ram_enabled = value == 0x0A,
      0x2000 ... 0x2FFF => self.selected_rom_bank = (self.selected_rom_bank & 0x100) | value as usize , //lower 8 bits
      0x3000 ... 0x3FFF => self.selected_rom_bank = (self.selected_rom_bank) & 0xFF | ((value as usize) << 8),
      0x4000 ... 0x5FFF => self.selected_ram_bank = value as usize,
      _  => ()
    }
  }

  fn write_ram(&mut self, address: u16, value: u8) {
    if self.ram_enabled {
      self.ram[RAM_BANK_SIZE * self.selected_ram_bank + address as usize] = value;
    }
  }
}