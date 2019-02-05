mod registers;
pub mod mmu;

use crate::cpu::mmu::MMU;
use crate::cpu::registers::{Registers, CpuFlag};

pub struct CPU {
  registers: Registers,
  mmu: MMU,
  halted: bool,
  ime: bool, // interrupt master enable - set by DI and EI
}

impl CPU {
  pub fn new(buffer: [u8; 0xFFFF]) -> CPU {
    CPU {
      registers: Registers::new(),
      mmu: MMU::new(buffer),
      halted: false,
      ime: false,
    }
  }

  pub fn tick(&mut self, input_state: [bool; 8]) -> usize {
    //@TODO interrupt magic
    if !self.halted {
      if self.ime {
        let irq = self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F);
        if irq > 0 { // there is an interrupt we need to handle it @TODO only vblank for now
          self.ime = false; //don´t allow new interrupts until we handled this one

          self.push(self.registers.pc);
          self.registers.pc = 0x0040; //vblank handler address
          self.mmu.write_byte(0xFF0F, 0x00); //reset interupt request

          /*Bit 0: V-Blank  Interrupt Request (INT 40h)  (1=Request)
            Bit 1: LCD STAT Interrupt Request (INT 48h)  (1=Request)
            Bit 2: Timer    Interrupt Request (INT 50h)  (1=Request)
            Bit 3: Serial   Interrupt Request (INT 58h)  (1=Request)
            Bit 4: Joypad */



        }
      }

      let ticks = self.do_cylce();

      self.mmu.do_ticks(ticks);

      ticks
    } else {
      //check for interrupts
      0
    }
  }

  pub fn get_screen_buffer(&self) -> &[[u8; crate::cpu::mmu::gpu::SCREEN_WIDTH]; crate::cpu::mmu::gpu::SCREEN_HEIGHT] {
    self.mmu.get_screen_buffer()
  }

  pub fn screen_changed(&self) -> bool {
    self.mmu.read_byte(0xFF0F) & 0x01 == 0x01
  }

  fn do_cylce(&mut self) -> usize {
    let current_address = self.registers.pc;
    let op_code = self.fetch_byte();

    //println!("do_cycle: {:#04X} @ {:#06X}", op_code, current_address);

    match op_code {
      0x00 => 4, //NOOP
      0x01 => { let next_word = self.fetch_word(); self.registers.set_bc(next_word); 12 }, //LD BC, d16
      0x02 => { self.mmu.write_byte(self.registers.get_bc(), self.registers.a); 8 }, //LD (BC),A
      0x05 => { self.registers.b = self.alu_dec(self.registers.b); 4 }, //DEC B
      0x06 => { self.registers.b = self.fetch_byte(); 8 }, //LD B, n
      0x0B => { let dec = self.registers.get_bc().wrapping_sub(1); self.registers.set_bc(dec); 8 }, //DEC BC - no flags set
      0x0C => { self.registers.c = self.alu_inc(self.registers.c); 4 }, //INC C
      0x0D => { self.registers.c = self.alu_dec(self.registers.c); 4}, //DEC C
      0x0E => { self.registers.c = self.fetch_byte(); 8 }, //LD C, n
      0x11 => { let next_word = self.fetch_word(); self.registers.set_de(next_word); 12 }, //LD DE,nn
      0x12 => { self.mmu.write_byte(self.registers.get_de(), self.registers.a); 8 }, //LD (DE),A
      0x1C => { self.registers.e = self.alu_inc(self.registers.e); 4 }, //INC C
      0x20 => { if !self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 }, //JR NZ, n
      0x21 => { let next_word = self.fetch_word(); self.registers.set_hl(next_word); 12 }, //LD HL, nn
      0x23 => { self.registers.set_hl(self.registers.get_hl().wrapping_add(1)); 8 }, //INC HL
      0x27 => { self.alu_daa(); 4 }, //DAA
      0x28 => { if self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 }, //JR Z, n
      0x2A => { self.registers.a = self.mmu.read_byte(self.registers.get_hli()); 8 }, //LD A,(HL+)
      0x2C => { self.registers.l = self.alu_inc(self.registers.l); 4 }, //INC L
      0x2F => { self.alu_cpl(); 4 } //CPL - A=A XOR FF - method for flags
      0x30 => { if !self.registers.get_flag(CpuFlag::C) { self.jump_r(); 12 } else { self.registers.pc += 1; 8} }, //JR NC, n
      0x31 => { self.registers.sp = self.fetch_word(); 12 }, //LD SP,d16
      0x32 => { self.mmu.write_byte(self.registers.get_hld(), self.registers.a); 8 }, //LD (HL-), A
      0x35 => { let dec_byte = self.alu_dec(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), dec_byte); 12 } //DEC (HL)W
      0x36 => { let next_byte = self.fetch_byte(); self.mmu.write_byte(self.registers.get_hl(), next_byte); 12 }, //LD (HL),n
      0x38 => { if self.registers.get_flag(CpuFlag::C) { self.jump_r(); 12 } else { self.registers.pc += 1; 8 } }, //JR C, n
      0x3A => { self.registers.a = self.mmu.read_byte(self.registers.get_hld()); 8 }, //LD A,(HL-)
      0x3E => { self.registers.a = self.fetch_byte(); 8 }, //LD A,#
      0x47 => { self.registers.b = self.registers.a; 4 }, //LD B,A
      0x4F => { self.registers.c = self.registers.a; 4 }, //LD C,A
      0x50 => { self.registers.d = self.registers.b; 4 }, //LD D,B
      0x6E => { self.registers.l = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD L,(HL)
      0x76 => { self.halted = true; 4 }, //HALT
      0x77 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.a); 8 }, //LD (HL),A
      0x78 => { self.registers.a = self.registers.b; 4 }, //LD A,B
      0x79 => { self.registers.a = self.registers.c; 4 }, //LD A,C
      0x7A => { self.registers.a = self.registers.d; 4 }, //LD A,D
      0x7B => { self.registers.a = self.registers.e; 4 }, //LD A,E
      0x7C => { self.registers.a = self.registers.h; 4 }, //LD A,H
      0x7D => { self.registers.a = self.registers.l; 4 }, //LD A,L
      0x7E => { self.registers.a = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD A,(HL)
      0x7F => { 4 }, //LD A,A but why?
      0x80 => { self.alu_add(self.registers.b); 4 }, //ADD B
      0x81 => { self.alu_add(self.registers.c); 4 }, //ADD C
      0x82 => { self.alu_add(self.registers.d); 4 }, //ADD D
      0x83 => { self.alu_add(self.registers.e); 4 }, //ADD E
      0x84 => { self.alu_add(self.registers.h); 4 }, //ADD H
      0x85 => { self.alu_add(self.registers.l); 4 }, //ADD L
      0x86 => { self.alu_add(self.mmu.read_byte(self.registers.get_hl())); 4 }, //ADD (HL)
      0x87 => { self.alu_add(self.registers.a); 4 }, //ADD A
      0x8C => { self.alu_adc(self.registers.h); 4 },  //ADC H
      0xA1 => { self.alu_and(self.registers.c); 4 }, //AND C
      0xA7 => { self.registers.b = self.registers.a; 4 }, //LD B,A
      0xA9 => { self.alu_xor(self.registers.c); 4 }, //XOR C
      0xAF => { self.alu_xor(self.registers.a); 4 }, //XOR A
      0xB0 => { self.alu_or(self.registers.b); 4 }, //OR B
      0xB1 => { self.alu_or(self.registers.c); 4 }, //OR C
      0xBB => { self.alu_cp(self.registers.e); 4 }, //CP E
      0xB7 => { self.alu_or(self.registers.a); 4 }, //OR A
      0xC0 => { if !self.registers.get_flag(CpuFlag::Z) { self.retrn(); 20 } else { 8 } }, //RET NZ
      0xC1 => { let bc = self.pop(); self.registers.set_bc(bc); 12 }, //POP BC
      0xC3 => { self.registers.pc = self.fetch_word(); 12 }, //JUMP nn
      0xC4 => { self.registers.c = self.registers.h; 4 }, //LD C,H
      0xC5 => { self.push(self.registers.get_bc()); 16 } //PUSH BC
      0xC9 => { self.retrn(); 16 }, //RET
      0xCA => { if self.registers.get_flag(CpuFlag::Z) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP Z,nn
      0xCC => { if self.registers.get_flag(CpuFlag::Z) { let address = self.fetch_word(); self.call(address); 24 } else { self.registers.pc += 2; 12 } }, //CALL Z,nn
      0xCD => { let address = self.fetch_word(); self.call(address); 24 }, //CALL a16
      0xCB => { self.op_code_cb() + 4 }, //CB
      0xD1 => { let de = self.pop(); self.registers.set_de(de); 12 } //POP DE
      0xD5 => { self.push(self.registers.get_de()); 16 }, //PUSH DE
      0xD8 => { self.alu_adc(self.registers.l); 4 }, //ADC A,L
      0xDA => { if self.registers.get_flag(CpuFlag::C) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP C,nn
      0xE0 => { let address = 0xFF00 + self.fetch_byte() as u16; self.mmu.write_byte(address, self.registers.a); 12 }, //LDH (n),
      0xE1 => { let hl = self.pop(); self.registers.set_hl(hl); 12 } //POP HL
      0xE2 => { self.mmu.write_byte(0xFF00 + self.registers.c as u16, self.registers.a);  8 }, //LD (C),A
      0xE5 => { self.push(self.registers.get_hl()); 16 } //PUSH HL
      0xE6 => { let next_byte = self.fetch_byte(); self.alu_and(next_byte); 8 } //AND n
      0xEA => { let address = self.fetch_word(); self.mmu.write_byte(address, self.registers.a); 16 }, //LD (nn),A
      0xF0 => { let address = 0xFF00 + self.fetch_byte() as u16; self.registers.a = self.mmu.read_byte(address); 12 }, //LDH A,(n)
      0xF1 => { let af = self.pop(); self.registers.set_af(af); 12 } //POP AF
      0xF3 => { self.ime = false; 4 }, //DI disable interrupts
      0xF5 => { self.push(self.registers.get_af()); 16 }, //PUSH AF
      0xF8 => { let value = self.alu_add_next_signed_byte_to_word(self.registers.sp); self.registers.set_hl(value); 12 }, //LD HL, SP+r8
      0xFA => { let address = self.fetch_word(); self.registers.a = self.mmu.read_byte(address); 16 }, //LD A,(nn)
      0xFB => { self.ime = true; 4 }, //EI enable interrupts
      0xFE => { let next_byte = self.fetch_byte(); self.alu_cp(next_byte); 8 }, //CP A,n  compare a-n
      0xFF => { self.call(0x0038); 16 }, //RST 0x0038
      _ => { panic!("Unknown command {:#04X} at {:#06X}", op_code, self.registers.pc - 1); }
    }
  }

  fn op_code_cb(&mut self) -> usize {
    let op_code = self.fetch_byte();

    match op_code {
      0x37 => { self.registers.a = self.alu_swap(self.registers.a); 8 }, //SWAP A

      _ => panic!("Unknown 0xCB command {:#04X} at {:#06X}", op_code, self.registers.pc - 1)
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

  fn push(&mut self, value: u16) {
    self.registers.sp -= 2; //stack grows down from 0xFFFE and stores words
    self.mmu.write_word(self.registers.sp, value);
  }

  fn pop(&mut self) -> u16 {
    let result = self.mmu.read_word(self.registers.sp);
    self.registers.sp += 2;
    result
  }

  fn call(&mut self, address: u16) {
    //println!("CALL {:#06X}@{:#04X}", address, self.registers.pc - 1);
    self.push(self.registers.pc + 1);
    self.registers.pc = address;
  }

  fn retrn(&mut self) {
    self.registers.pc = self.pop();
  }

  fn alu_and(&mut self, value: u8) {
    self.registers.a &= value;

    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, self.registers.a == 0);
    self.registers.set_flag(CpuFlag::H, true);
  }

  fn alu_or(&mut self, value: u8) {
    self.registers.a |= value;

    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, self.registers.a == 0);
  }

  fn alu_xor(&mut self, value: u8) {
    self.registers.a ^= value;

    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, self.registers.a == 0);
  }

  fn alu_cpl(&mut self) {
    self.registers.a ^= 0xFF;

    self.registers.set_flag(CpuFlag::N, true);
    self.registers.set_flag(CpuFlag::H, true);
  }

  fn alu_inc(&mut self, value: u8) -> u8 {
    let result = value.wrapping_add(1);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, (value & 0x0F) + 1 > 0x0F); //a half carry occurs when the low nibble + 1 is greater than 0x0F
    result
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
    self.registers.set_flag(CpuFlag::H, (((self.registers.a & 0x0F) + (value & 0x0F)) & 0x10) == 0x10);
    self.registers.set_flag(CpuFlag::C, self.registers.a as u16 + value as u16 > 0xFF);
    self.registers.a = result;
  }

  fn alu_adc(&mut self, value: u8) { //like add + carry flag
    let c: u8 = if self.registers.get_flag(CpuFlag::C) { 1 } else { 0 };
    let result = self.registers.a.wrapping_add(value).wrapping_add(c);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, (((self.registers.a & 0x0F) + (value & 0x0F) + c) & 0x10) == 0x10);
    self.registers.set_flag(CpuFlag::C, self.registers.a as u16 + value as u16 + c as u16 > 0xFF);

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

  fn alu_swap(&mut self, value: u8) -> u8 {
    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, value == 0);
    (value << 4) | (value >> 4)
  }

  fn alu_add_next_signed_byte_to_word(&mut self, value: u16) -> u16 {
    let byte = self.fetch_byte() as i8 as i16 as u16; //some rust magic that magic to add the u16 with wrapping add; i16 -> if i16 < 0 { u16 = u16.max - abs(i16) } and the sign is then done via the wrap around
    self.registers.reset_flags();

    self.registers.set_flag(CpuFlag::H, (value & 0x000F) + (byte & 0x000F) > 0x000F);
    self.registers.set_flag(CpuFlag::C, (value & 0x00FF) + (byte & 0x00FF) > 0x00FF);

    value.wrapping_add(byte)
  }

  fn alu_daa(&mut self) {  //i got no idea what i'm doing
    let mut adjust = if self.registers.get_flag(CpuFlag::C) { 0x60 } else { 0x00 };
    if self.registers.get_flag(CpuFlag::H) { adjust |= 0x06; }
    if !self.registers.get_flag(CpuFlag::N) {
      if self.registers.a & 0x0F > 0x09 { adjust |= 0x06; };
      if self.registers.a > 0x99 { adjust |= 0x60; };
      self.registers.a = self.registers.a.wrapping_add(adjust);
    } else {
      self.registers.a = self.registers.a.wrapping_sub(adjust);
    }

    self.registers.set_flag(CpuFlag::Z, self.registers.a == 0);
    self.registers.set_flag(CpuFlag::H, false);
    self.registers.set_flag(CpuFlag::C, adjust >= 0x60);
  }

  fn jump_r(&mut self) {
    let offset = self.fetch_byte() as i8;
    self.registers.pc = (self.registers.pc as isize + offset as isize) as u16;
  }
}