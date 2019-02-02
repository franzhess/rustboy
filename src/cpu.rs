use crate::mmu::MMU;
use crate::registers::{Registers, CpuFlag};

pub const SCREEN_BUFFER_WIDTH: usize = 166;
pub const SCREEN_BUFFER_HEIGHT: usize = 144;

pub struct CPU {
  registers: Registers,
  mmu: MMU,
  halted: bool,
}

impl CPU {
  pub fn new(buffer: [u8; 0xFFFF]) -> CPU {
    CPU {
      registers: Registers::new(),
      mmu: MMU::new(buffer),
      halted: false,
    }
  }

  pub fn tick(&mut self, input_state: [bool; 8]) {
    self.do_cylce();
  }

  fn do_cylce(&mut self) -> usize {
    let op_code = self.read_byte();

    match op_code {
      0x00 => 4, //NOOP
      0x06 => { self.registers.b = self.read_byte(); 8 } //LD B, n
      0x0E => { self.registers.c = self.read_byte(); 8 } //LD C, n
      0x21 => { let value = self.read_word(); self.registers.set_hl(value); 12 }, //LD HL, nn
      0x76 => { self.halted = true; 4 } //HALT
      0xAF => { self.xor(self.registers.a); 4 } //XOR A
      0xC3 => { self.registers.pc = self.read_word(); 12 } //JUMP nn

      _ => panic!("Unknown command {:#06X} at {:#06X}", op_code, self.registers.pc - 1)
    }
  }

  fn read_byte(&mut self) -> u8 {
    let res = self.mmu.read_byte(self.registers.pc);
    self.registers.pc += 1;
    res
  }

  fn read_word(&mut self) -> u16 { //LSB FIRST
    let res = self.mmu.read_word(self.registers.pc);
    self.registers.pc += 2;
    res
  }

  fn xor(&mut self, value: u8) {
    self.registers.a ^= value;
    self.registers.reset_flags();

    if self.registers.a == 0 {
      self.registers.set_flag(CpuFlag::Z, true);
    }

  }
}