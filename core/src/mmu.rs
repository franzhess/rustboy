use std::str;
use crate::joypad::Joypad;
use crate::timer::Timer;
use crate::gpu::VOAM_SIZE;
use crate::mbc::{load_rom, Mbc};
use crate::gpu::GPU;
use crate::serial::Serial;

const WRAM_SIZE: usize = 0x8000;
const HRAM_SIZE: usize = 0x7F;

pub struct Mmu {
  wram: [u8; WRAM_SIZE],
  hram: [u8; HRAM_SIZE],
  gpu: GPU,
  timer: Timer,
  pub joypad: Joypad,
  mbc: Box<Mbc+'static>,
  serial: Serial,
  interrupt_enable: u8,
  interrupt_request: u8,
  voam_oam: u8,
}

impl Mmu {
  pub fn new(file_name: &str) -> Mmu {
    let rom = load_rom(file_name);
    println!("Successfully loaded: {}", rom.name());

    Mmu {
      wram: [0; WRAM_SIZE],
      hram: [0; HRAM_SIZE],
      gpu: GPU::new(),
      timer: Timer::new(),
      joypad: Joypad::new(),
      mbc: rom,
      serial: Serial::new(),
      interrupt_enable: 0x00,
      interrupt_request: 0x00,
      voam_oam: 0x00,
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0x0000 ... 0x7FFF => self.mbc.read_rom(address), //ROM from cartridge
      0x8000 ... 0x9FFF => self.gpu.read_byte(address), //VRAM
      0xA000 ... 0xBFFF => self.mbc.read_ram(address - 0xA000),
      0xC000 ... 0xDFFF => self.wram[address as usize - 0xC000], //WRAM
      0xE000 ... 0xFDFF => self.wram[address as usize - 0xE000], //WRAM Echo
      0xFE00 ... 0xFE9F => self.gpu.read_byte(address), //OAM
      0xFEA0 ... 0xFEFF => 0, //not useable
      0xFF00 => self.joypad.read(), //Joypad
      0xFF01 ... 0xFF02 => self.serial.read(address), //serial
      0xFF04 ... 0xFF07 => self.timer.read_byte(address), //TIMER
      0xFF0F => self.interrupt_request,
      0xFF10 ... 0xFF3F => 0, //sound
      0xFF46 => self.voam_oam,
      0xFF40 ... 0xFF4B => self.gpu.read_byte(address),
      0xFF80 ... 0xFFFE => self.hram[address as usize - 0xFF80], //HRAM
      0xFFFF => self.interrupt_enable,
      _ => 0
    }
  }

  pub fn read_word(&self, address: u16) -> u16 { //LSB FIRST
    self.read_byte(address) as u16 | (self.read_byte(address + 1) as u16) << 8
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0x0000 ... 0x7FFF => self.mbc.write_rom(address, value), //ROM cartridge
      0x8000 ... 0x9FFF => self.gpu.write_byte(address, value), //VRAM
      0xA000 ... 0xBFFF => self.mbc.write_ram(address - 0xA000, value),
      0xC000 ... 0xDFFF => self.wram[address as usize - 0xC000] = value, //WRAM
      0xE000 ... 0xFDFF => self.wram[address as usize - 0xE000] = value, //WRAM Echo
      0xFE00 ... 0xFE9F => self.gpu.write_byte(address, value), //OAM
      0xFEA0 ... 0xFEFF => (), //not useable
      0xFF00 => self.joypad.write(value), //JOYPAD
      0xFF01 ... 0xFF02 => self.serial.write(address, value), //serial
      0xFF04 ... 0xFF07 => self.timer.write_byte(address, value), //timer
      0xFF0F => self.interrupt_request = value,
      0xFF10 ... 0xFF3F => (), //sound
      0xFF46 => { self.voam_oam = value; self.copy_to_voam(value) },
      0xFF40 ... 0xFF4B => self.gpu.write_byte(address, value),
      0xFF80 ... 0xFFFE => self.hram[address as usize - 0xFF80] = value, //HRAM
      0xFFFF => self.interrupt_enable = value,
      _ => {}
    }
  }

  pub fn write_word(&mut self, address: u16, value: u16) {
    self.write_byte(address, (value & 0x00FF) as u8); //LSB first
    self.write_byte(address + 1, ((value & 0xFF00) >> 8) as u8);
  }

  pub fn get_screen_buffer(&self) -> Vec<u8> {
    self.gpu.get_screen_buffer()
  }

  pub fn get_screen_updated(&mut self) -> bool {
    let b = self.gpu.screen_update;
    if self.gpu.screen_update {
      self.gpu.screen_update = false;
    }
    b
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.timer.do_ticks(ticks);
    self.gpu.do_ticks(ticks);
  }

  pub fn process_irq_requests(&mut self) {
    if self.gpu.irq_vblank {
      self.interrupt_request |= 0x01;
      self.gpu.irq_vblank = false;
    }

    if self.gpu.irq_stat {
      self.interrupt_request |= 0x02;
      self.gpu.irq_stat = false;
    }

    if self.timer.irq_timer {
      self.interrupt_request |= 0x04;
      self.timer.irq_timer = false;
    }
  }

  fn copy_to_voam(&mut self, value: u8) {
    let mem_start = (value as u16) << 8;
    for offset in 0..VOAM_SIZE {
      self.write_byte(0xFE00 + offset as u16, self.read_byte(mem_start + offset as u16));
    }
  }
}

/*
    buffer[0xFF05] = 0x00; // TIMA
    buffer[0xFF06] = 0x00; // TMA
    buffer[0xFF07] = 0x00; // TAC
    buffer[0xFF10] = 0x80; // NR10
    buffer[0xFF11] = 0xBF; // NR11
    buffer[0xFF12] = 0xF3; // NR12
    buffer[0xFF14] = 0xBF; // NR14
    buffer[0xFF16] = 0x3F; // NR21
    buffer[0xFF17] = 0x00; // NR22
    buffer[0xFF19] = 0xBF; // NR24
    buffer[0xFF1A] = 0x7F; // NR30
    buffer[0xFF1B] = 0xFF; // NR31
    buffer[0xFF1C] = 0x9F; // NR32
    buffer[0xFF1E] = 0xBF; // NR33
    buffer[0xFF20] = 0xFF; // NR41
    buffer[0xFF21] = 0x00; // NR42
    buffer[0xFF22] = 0x00; // NR43
    buffer[0xFF23] = 0xBF; // NR30
    buffer[0xFF24] = 0x77; // NR50
    buffer[0xFF25] = 0xF3; // NR51
    buffer[0xFF26] = 0xF1; //-GB, 0xF0-SGB // NR52
    buffer[0xFF40] = 0x91; // LCDC
    buffer[0xFF42] = 0x00; // SCY
    buffer[0xFF43] = 0x00; // SCx
    buffer[0xFF45] = 0x00; // LYC
    buffer[0xFF47] = 0xFC; // BGP
    buffer[0xFF48] = 0xFF; // OBP0
    buffer[0xFF49] = 0xFF; // OBP1
    buffer[0xFF4A] = 0x00; // WY
    buffer[0xFF4B] = 0x00; // Wx
    buffer[0xFFFF] = 0x00; // IE
*/