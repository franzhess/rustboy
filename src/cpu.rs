mod registers;
pub mod mmu;

use crate::cpu::mmu::MMU;
use crate::cpu::registers::{Registers, CpuFlag};

pub struct CPU {
  registers: Registers,
  mmu: MMU,
  halted: bool,
  ime: bool, // interrupt master enable - set by DI and EI
  di_counter: usize, // DI sets this - when it reaches zero, ime is set to false
  ei_counter: usize, // EI sets this - when it reaches zero, ime is set to false
}

impl CPU {
  pub fn new(buffer: [u8; 0xFFFF]) -> CPU {
    CPU {
      registers: Registers::new(),
      mmu: MMU::new(buffer),
      halted: false,
      ime: false,
      di_counter: 0,
      ei_counter: 0
    }
  }

  pub fn tick(&mut self, input_state: [bool; 8]) {
    self.do_cylce();
  }

  fn do_cylce(&mut self) -> usize {
    //let current_address = self.registers.pc;

    let op_code = self.fetch_byte();

    //println!("do_cycle: {:#04X} @ {:#06X}", op_code, current_address);

    match op_code {
      0x00 => 4, //NOOP
      0x05 => { self.registers.b = self.alu_dec(self.registers.b); 4 } //DEC B
      0x06 => { self.registers.b = self.fetch_byte(); 8 } //LD B, n
      0x0D => { self.registers.c = self.alu_dec(self.registers.c); 4} //DEC C
      0x0E => { self.registers.c = self.fetch_byte(); 8 } //LD C, n
      0x20 => { if !self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 } //JR NZ, n
      0x28 => { if self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 } //JR Z, n
      0x30 => { if !self.registers.get_flag(CpuFlag::C) { self.jump_r(); } else { self.registers.pc += 1; }; 8 } //JR NC, n
      0x38 => { if self.registers.get_flag(CpuFlag::C) { self.jump_r(); } else { self.registers.pc += 1; }; 8 } //JR C, n
      0x21 => { let next_word = self.fetch_word(); self.registers.set_hl(next_word); 12 }, //LD HL, nn
      0x32 => { self.mmu.write_byte(self.registers.get_hld(), self.registers.a); 8 }, //LD (HL-), A
      0x3E => { self.registers.a = self.fetch_byte(); 8 } // LD A,#
      0x76 => { self.halted = true; 4 } //HALT
      0xAF => { self.alu_xor(self.registers.a); 4 } //XOR A
      0xC3 => { self.registers.pc = self.fetch_word(); 12 } //JUMP nn
      0xE0 => { let offset = self.fetch_byte() as u16; self.mmu.write_byte(0xFF00 +  offset, self.registers.a); 12 } //LDH (n),A
      0xF0 => { let offset = self.fetch_byte() as u16; self.registers.a = self.mmu.read_byte(0xFF00 + offset); 12 } // LDH A,(n)
      0xF3 => { self.di_counter = 2; 4 } // DI disable interrupts after the next op
      0xFE => { let next_byte = self.fetch_byte(); self.alu_cp(next_byte); 8 }

      _ => panic!("Unknown command {:#04X} at {:#06X}", op_code, self.registers.pc - 1)
    }
  }

  fn fetch_byte(&mut self) -> u8 {
    let res = self.mmu.read_byte(self.registers.pc);
    self.registers.pc += 1;
    res
  }

  fn fetch_word(&mut self) -> u16 {
    let res = self.mmu.read_word(self.registers.pc);
    self.registers.pc += 2;
    res
  }

  fn alu_xor(&mut self, value: u8) {
    self.registers.a ^= value;
    self.registers.reset_flags();

    if self.registers.a == 0 {
      self.registers.set_flag(CpuFlag::Z, true);
    }

  }

  fn alu_dec(&mut self, value: u8) -> u8 {
    let result = value.wrapping_sub(1);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, true);
    self.registers.set_flag(CpuFlag::H, (value & 0x0F) == 0); //a half carry will occur when the low nibble is all zeros
    result
  }

  fn alu_add(&mut self, value: u8) {
    let result = self.registers.a.wrapping_add(value);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, (((self.registers.a & 0xf) + (value & 0xf)) & 0x10) == 0x10);
    self.registers.set_flag(CpuFlag::C, self.registers.a as u16 + value as u16 > 255);
    self.registers.a = result;
  }

  fn alu_sub(&mut self, value: u8) {
    let result = self.registers.a.wrapping_sub(value);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, true);
    self.registers.set_flag(CpuFlag::H, (self.registers.a & 0x0F) < (value & 0x0F));
    self.registers.set_flag(CpuFlag::C, self.registers.a < value);
    self.registers.a = result;
  }

  fn alu_cp(&mut self, value: u8) {
    let temp = self.registers.a;
    self.alu_sub(value);
    self.registers.a = temp;
  }

  fn jump_r(&mut self) {
    let offset = self.fetch_byte() as i8;
    self.registers.pc = (self.registers.pc as i32 + offset as i32) as u16;
  }
}