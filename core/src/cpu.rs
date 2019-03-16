mod registers;
mod ops;

use crate::mmu::MMU;
use crate::cpu::registers::{Registers, CpuFlag};
use crate::SCREEN_WIDTH;
use crate::SCREEN_HEIGHT;
use crate::joypad::GBKeyEvent;

pub struct CPU {
  registers: Registers,
  mmu: MMU,
  halted: bool,
  ime: bool, // interrupt master enable - set by DI and EI
  ei_requested: usize, //EI has one cycle delay
}

impl CPU {
  pub fn new(buffer: [u8; 0xFFFF]) -> CPU {
    CPU {
      registers: Registers::new(),
      mmu: MMU::new(buffer),
      halted: false,
      ime: false,
      ei_requested: 0,
    }
  }

  pub fn tick(&mut self) -> usize { //, input_state: [bool; 8]) -> usize {
    //self.mmu.set_joypad_state(input_state);

    self.ei_requested = match self.ei_requested {
      2 => 1,
      1 => { self.ime = true; 0 },
      _ => 0
    };

    self.handle_irq();

    let ticks = if !self.halted {
      self.do_cylce()
    } else { 4 };

    self.mmu.do_ticks(ticks);

    ticks
  }

  fn handle_irq(&mut self) {
    self.mmu.process_irq_requests(); //loads the irq requests into 0xFF0F

    let irq_requested = self.mmu.read_byte(0xFF0F);
    let irq = self.mmu.read_byte(0xFFFF) & irq_requested & 0x1F;
    if irq > 0 { //there was an interrupt
      self.halted = false; //end halt when an interrupt occurs

      if self.ime { //if interrupts are enabled, handle them
        self.ime = false; //don´t allow new interrupts until we handled this one

        let irq_num = irq.trailing_zeros(); //0 vblank, 1 stat, 2 timer, 3 serial, 4 joypad

        self.push(self.registers.pc);
        self.registers.pc = (0x0040 + 8 * irq_num) as u16; // jump to the interrupt handler

        self.mmu.write_byte(0xFF0F, irq_requested & !(1 << irq_num));  //reset the irq request - like res

      }
    }
  }

  pub fn process_input_event(&mut self, event: GBKeyEvent) {
    self.mmu.joypad.receive_event(event);
  }


  pub fn get_screen_buffer(&self) -> Vec<u8> {
    self.mmu.get_screen_buffer()
  }

  pub fn get_screen_updated(&mut self) -> bool {
    self.mmu.get_screen_updated()
  }

  fn do_cylce(&mut self) -> usize {
    let current_address = self.registers.pc;
    let op_code = self.fetch_byte();

    //println!("do_cycle: {:#04X} @ {:#06X}", op_code, current_address);

    match op_code {
      0x00 => 4, //NOOP
      0x01 => { let next_word = self.fetch_word(); self.registers.set_bc(next_word); 12 }, //LD BC, d16
      0x02 => { self.mmu.write_byte(self.registers.get_bc(), self.registers.a); 8 }, //LD (BC),A
      0x03 => { self.registers.set_bc(self.registers.get_bc().wrapping_add(1)); 8 }, //INC BC
      0x04 => { self.registers.b = self.alu_inc(self.registers.b); 4 }, //INC B
      0x05 => { self.registers.b = self.alu_dec(self.registers.b); 4 }, //DEC B
      0x06 => { self.registers.b = self.fetch_byte(); 8 }, //LD B, n
      0x07 => { self.registers.a = self.alu_rlc(self.registers.a); 4 }, //RLCA - rotate left a
      0x08 => { let address = self.fetch_word(); self.mmu.write_word(address, self.registers.sp); 20 }, //LD (nn),SP
      0x09 => { self.alu_add16(self.registers.get_bc()); 8 }, //ADD HL,BC
      0x0A => { self.registers.a = self.mmu.read_byte(self.registers.get_bc()); 8 }, //LD A,(BC)
      0x0B => { self.registers.set_bc(self.registers.get_bc().wrapping_sub(1)); 8 }, //DEC BC - no flags set
      0x0C => { self.registers.c = self.alu_inc(self.registers.c); 4 }, //INC C
      0x0D => { self.registers.c = self.alu_dec(self.registers.c); 4}, //DEC C
      0x0E => { self.registers.c = self.fetch_byte(); 8 }, //LD C, n
      0x0F => { self.registers.a = self.alu_rrc(self.registers.a); 4 }, //RRCA
      0x10 => { self.halted = true; 4 }, //STOP @TODO implement
      0x11 => { let next_word = self.fetch_word(); self.registers.set_de(next_word); 12 }, //LD DE,nn
      0x12 => { self.mmu.write_byte(self.registers.get_de(), self.registers.a); 8 }, //LD (DE),A
      0x13 => { self.registers.set_de(self.registers.get_de().wrapping_add(1)); 8 }, //INC DE
      0x14 => { self.registers.d = self.alu_inc(self.registers.d); 4}, //INC D
      0x15 => { self.registers.d = self.alu_dec(self.registers.d); 4}, //DEC D
      0x16 => { self.registers.d = self.fetch_byte(); 8 }, //LD D,n
      0x17 => { self.registers.a = self.alu_rl(self.registers.a); 4 }, //RLA - rotate left through carry
      0x18 => { self.jump_r(); 12 }, //JR n
      0x19 => { self.alu_add16(self.registers.get_de()); 8 }, //ADD HL,DE
      0x1A => { self.registers.a = self.mmu.read_byte(self.registers.get_de()); 8 }, //LD A,(DE)
      0x1B => { self.registers.set_de(self.registers.get_de().wrapping_sub(1)); 8 }, //DEC DE - no flags set
      0x1C => { self.registers.e = self.alu_inc(self.registers.e); 4 }, //INC E
      0x1D => { self.registers.e = self.alu_dec(self.registers.e); 4}, //DEC E
      0x1E => { self.registers.e = self.fetch_byte(); 8 }, //LD E,n
      0x1F => { self.registers.a = self.alu_rr(self.registers.a); 4 }, //RRA
      0x20 => { if !self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 }, //JR NZ, n
      0x21 => { let next_word = self.fetch_word(); self.registers.set_hl(next_word); 12 }, //LD HL,nn
      0x22 => { self.mmu.write_byte(self.registers.get_hli(), self.registers.a); 12 }, //LD (HL+),A
      0x23 => { self.registers.set_hl(self.registers.get_hl().wrapping_add(1)); 8 }, //INC HL
      0x24 => { self.registers.h = self.alu_inc(self.registers.h); 4}, //INC H
      0x25 => { self.registers.h = self.alu_dec(self.registers.h); 4}, //DEC H
      0x26 => { self.registers.h = self.fetch_byte(); 8 }, //LD H,n
      0x27 => { self.alu_daa(); 4 }, //DAA
      0x28 => { if self.registers.get_flag(CpuFlag::Z) { self.jump_r(); } else { self.registers.pc += 1; }; 8 }, //JR Z, n
      0x29 => { self.alu_add16(self.registers.get_hl()); 8 }, //ADD HL,HL
      0x2A => { self.registers.a = self.mmu.read_byte(self.registers.get_hli()); 8 }, //LD A,(HL+)
      0x2B => { self.registers.set_hl(self.registers.get_hl().wrapping_sub(1)); 8 }, //DEC HL - no flags set
      0x2C => { self.registers.l = self.alu_inc(self.registers.l); 4 }, //INC L
      0x2D => { self.registers.l = self.alu_dec(self.registers.l); 4}, //DEC L
      0x2E => { self.registers.l = self.fetch_byte(); 8 }, //LD L,n
      0x2F => { self.alu_cpl(); 4 } //CPL - A=A XOR FF - method for flags
      0x30 => { if !self.registers.get_flag(CpuFlag::C) { self.jump_r(); 12 } else { self.registers.pc += 1; 8} }, //JR NC, n
      0x31 => { self.registers.sp = self.fetch_word(); 12 }, //LD SP,d16
      0x32 => { self.mmu.write_byte(self.registers.get_hld(), self.registers.a); 8 }, //LD (HL-), A
      0x33 => { self.registers.sp = self.registers.sp.wrapping_add(1); 8 }, //INC SP
      0x34 => { let inc_byte = self.alu_inc(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), inc_byte); 12 }, //INC (HL)
      0x35 => { let dec_byte = self.alu_dec(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), dec_byte); 12 } //DEC (HL)
      0x36 => { let next_byte = self.fetch_byte(); self.mmu.write_byte(self.registers.get_hl(), next_byte); 12 }, //LD (HL),n
      0x37 => { self.alu_scf(); 4 }, //SCF
      0x38 => { if self.registers.get_flag(CpuFlag::C) { self.jump_r(); 12 } else { self.registers.pc += 1; 8 } }, //JR C, n
      0x39 => { self.alu_add16(self.registers.sp); 8 } //ADD HL,SP
      0x3A => { self.registers.a = self.mmu.read_byte(self.registers.get_hld()); 8 }, //LD A,(HL-)
      0x3B => { self.registers.sp = self.registers.sp.wrapping_sub(1); 8 }, //DEC SP - no flags set
      0x3C => { self.registers.a = self.alu_inc(self.registers.a); 4 }, //INC A
      0x3D => { self.registers.a = self.alu_dec(self.registers.a); 4 }, //DEC A
      0x3E => { self.registers.a = self.fetch_byte(); 8 }, //LD A,#
      0x3F => { self.alu_ccf();  4 }, //CCF - flip carry
      0x40 => { 4 }, //LD B,B
      0x41 => { self.registers.b = self.registers.c; 4 }, //LD B,C
      0x42 => { self.registers.b = self.registers.d; 4 }, //LD B,D
      0x43 => { self.registers.b = self.registers.e; 4 }, //LD B,E
      0x44 => { self.registers.b = self.registers.h; 4 }, //LD B,H
      0x45 => { self.registers.b = self.registers.l; 4 }, //LD B,L
      0x46 => { self.registers.b = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD B,(HL)
      0x47 => { self.registers.b = self.registers.a; 4 }, //LD B,A
      0x48 => { self.registers.c = self.registers.b; 4 }, //LD C,B
      0x49 => { 4 }, //LD C,C
      0x4A => { self.registers.c = self.registers.d; 4 }, //LD C,D
      0x4B => { self.registers.c = self.registers.e; 4 }, //LD C,E
      0x4C => { self.registers.c = self.registers.h; 4 }, //LD C,H
      0x4D => { self.registers.c = self.registers.l; 4 }, //LD C,L
      0x4E => { self.registers.c = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD C,(HL)
      0x4F => { self.registers.c = self.registers.a; 4 }, //LD C,A
      0x50 => { self.registers.d = self.registers.b; 4 }, //LD D,B
      0x51 => { self.registers.d = self.registers.c; 4 }, //LD D,C
      0x52 => { 4 }, //LD D,D
      0x53 => { self.registers.d = self.registers.e; 4 }, //LD D,E
      0x54 => { self.registers.d = self.registers.h; 4 }, //LD D,H
      0x55 => { self.registers.d = self.registers.l; 4 }, //LD D,L
      0x56 => { self.registers.d = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD D,(HL)
      0x57 => { self.registers.d = self.registers.a; 4 }, //LD D,A
      0x58 => { self.registers.e = self.registers.b; 4 }, //LD E,B
      0x59 => { self.registers.e = self.registers.c; 4 }, //LD E,C
      0x5A => { self.registers.e = self.registers.d; 4 }, //LD E,D
      0x5B => { 4 }, //LD E,E
      0x5C => { self.registers.e = self.registers.h; 4 }, //LD E,H
      0x5D => { self.registers.e = self.registers.l; 4 }, //LD E,L
      0x5E => { self.registers.e = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD E,(HL)
      0x5F => { self.registers.e = self.registers.a; 4 }, //LD E,A
      0x60 => { self.registers.h = self.registers.b; 4 }, //LD H,B
      0x61 => { self.registers.h = self.registers.c; 4 }, //LD H,C
      0x62 => { self.registers.h = self.registers.d; 4 }, //LD H,D
      0x63 => { self.registers.h = self.registers.e; 4 }, //LD H,E
      0x64 => { 4 }, //LD H,H
      0x65 => { self.registers.h = self.registers.l; 4 }, //LD H,L
      0x66 => { self.registers.h = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD H,(HL)
      0x67 => { self.registers.h = self.registers.a; 4 }, //LD H,A
      0x68 => { self.registers.l = self.registers.b; 4 }, //LD L,B
      0x69 => { self.registers.l = self.registers.c; 4 }, //LD L,C
      0x6A => { self.registers.l = self.registers.d; 4 }, //LD L,D
      0x6B => { self.registers.l = self.registers.e; 4 }, //LD L,E
      0x6C => { self.registers.l = self.registers.h; 4 }, //LD L,H
      0x6D => { 4 }, //LD L,L
      0x6E => { self.registers.l = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD L,(HL)
      0x6F => { self.registers.l = self.registers.a; 4 }, //LD L,A
      0x70 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.b); 8 }, //LD (HL),B
      0x71 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.c); 8 }, //LD (HL),C
      0x72 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.d); 8 }, //LD (HL),D
      0x73 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.e); 8 }, //LD (HL),E
      0x74 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.h); 8 }, //LD (HL),H
      0x75 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.l); 8 }, //LD (HL),L
      0x76 => { self.halted = true; 4 }, //HALT
      0x77 => { self.mmu.write_byte(self.registers.get_hl(), self.registers.a); 8 }, //LD (HL),A
      0x78 => { self.registers.a = self.registers.b; 4 }, //LD A,B
      0x79 => { self.registers.a = self.registers.c; 4 }, //LD A,C
      0x7A => { self.registers.a = self.registers.d; 4 }, //LD A,D
      0x7B => { self.registers.a = self.registers.e; 4 }, //LD A,E
      0x7C => { self.registers.a = self.registers.h; 4 }, //LD A,H
      0x7D => { self.registers.a = self.registers.l; 4 }, //LD A,L
      0x7E => { self.registers.a = self.mmu.read_byte(self.registers.get_hl()); 8 }, //LD A,(HL)
      0x7F => { 4 }, //LD A,A
      0x80 => { self.alu_add(self.registers.b); 4 }, //ADD B
      0x81 => { self.alu_add(self.registers.c); 4 }, //ADD C
      0x82 => { self.alu_add(self.registers.d); 4 }, //ADD D
      0x83 => { self.alu_add(self.registers.e); 4 }, //ADD E
      0x84 => { self.alu_add(self.registers.h); 4 }, //ADD H
      0x85 => { self.alu_add(self.registers.l); 4 }, //ADD L
      0x86 => { self.alu_add(self.mmu.read_byte(self.registers.get_hl())); 4 }, //ADD (HL)
      0x87 => { self.alu_add(self.registers.a); 4 }, //ADD A
      0x88 => { self.alu_adc(self.registers.b); 4 },  //ADC B
      0x89 => { self.alu_adc(self.registers.c); 4 },  //ADC C
      0x8A => { self.alu_adc(self.registers.d); 4 },  //ADC D
      0x8B => { self.alu_adc(self.registers.e); 4 },  //ADC E
      0x8C => { self.alu_adc(self.registers.h); 4 },  //ADC H
      0x8D => { self.alu_adc(self.registers.l); 4 },  //ADC L
      0x8E => { self.alu_adc(self.mmu.read_byte(self.registers.get_hl())); 8 },  //ADC (HL)
      0x8F => { self.alu_adc(self.registers.a); 4 },  //ADC A
      0x90 => { self.alu_sub(self.registers.b); 4 }, //SUB B
      0x91 => { self.alu_sub(self.registers.c); 4 }, //SUB C
      0x92 => { self.alu_sub(self.registers.d); 4 }, //SUB D
      0x93 => { self.alu_sub(self.registers.e); 4 }, //SUB E
      0x94 => { self.alu_sub(self.registers.h); 4 }, //SUB H
      0x95 => { self.alu_sub(self.registers.l); 4 }, //SUB L
      0x96 => { self.alu_sub(self.mmu.read_byte(self.registers.get_hl())); 4 }, //SUB (HL)
      0x97 => { self.alu_sub(self.registers.a); 4 }, //SUB A
      0x98 => { self.alu_sbc(self.registers.b); 4 },  //SBC B
      0x99 => { self.alu_sbc(self.registers.c); 4 },  //SBC C
      0x9A => { self.alu_sbc(self.registers.d); 4 },  //SBC D
      0x9B => { self.alu_sbc(self.registers.e); 4 },  //SBC E
      0x9C => { self.alu_sbc(self.registers.h); 4 },  //SBC H
      0x9D => { self.alu_sbc(self.registers.l); 4 },  //SBC L
      0x9E => { self.alu_sbc(self.mmu.read_byte(self.registers.get_hl())); 8 },  //SBC (HL)
      0x9F => { self.alu_sbc(self.registers.a); 4 },  //SBC A
      0xA0 => { self.alu_and(self.registers.b); 4 }, //AND B
      0xA1 => { self.alu_and(self.registers.c); 4 }, //AND C
      0xA2 => { self.alu_and(self.registers.d); 4 }, //AND D
      0xA3 => { self.alu_and(self.registers.e); 4 }, //AND E
      0xA4 => { self.alu_and(self.registers.h); 4 }, //AND H
      0xA5 => { self.alu_and(self.registers.l); 4 }, //AND L
      0xA6 => { self.alu_and(self.mmu.read_byte(self.registers.get_hl())); 8 }, //AND (HL)
      0xA7 => { self.alu_and(self.registers.a); 4 }, //AND A
      0xA8 => { self.alu_xor(self.registers.b); 4 }, //XOR B
      0xA9 => { self.alu_xor(self.registers.c); 4 }, //XOR C
      0xAA => { self.alu_xor(self.registers.d); 4 }, //XOR D
      0xAB => { self.alu_xor(self.registers.e); 4 }, //XOR E
      0xAC => { self.alu_xor(self.registers.h); 4 }, //XOR H
      0xAD => { self.alu_xor(self.registers.l); 4 }, //XOR L
      0xAE => { self.alu_xor(self.mmu.read_byte(self.registers.get_hl())); 8 }, //XOR (HL)
      0xAF => { self.alu_xor(self.registers.a); 4 }, //XOR A
      0xB0 => { self.alu_or(self.registers.b); 4 }, //OR B
      0xB1 => { self.alu_or(self.registers.c); 4 }, //OR C
      0xB2 => { self.alu_or(self.registers.d); 4 }, //OR D
      0xB3 => { self.alu_or(self.registers.e); 4 }, //OR E
      0xB4 => { self.alu_or(self.registers.h); 4 }, //OR H
      0xB5 => { self.alu_or(self.registers.l); 4 }, //OR L
      0xB6 => { self.alu_or(self.mmu.read_byte(self.registers.get_hl())); 8 }, //OR (HL)
      0xB7 => { self.alu_or(self.registers.a); 4 }, //OR A
      0xB8 => { self.alu_cp(self.registers.b); 4 }, //CP B
      0xB9 => { self.alu_cp(self.registers.c); 4 }, //CP C
      0xBA => { self.alu_cp(self.registers.d); 4 }, //CP D
      0xBB => { self.alu_cp(self.registers.e); 4 }, //CP E
      0xBC => { self.alu_cp(self.registers.h); 4 }, //CP H
      0xBD => { self.alu_cp(self.registers.l); 4 }, //CP L
      0xBE => { self.alu_cp(self.mmu.read_byte(self.registers.get_hl())); 4 }, //CP (HL)
      0xBF => { self.alu_cp(self.registers.a); 4 }, //CP A
      0xC0 => { if !self.registers.get_flag(CpuFlag::Z) { self.retrn(); 20 } else { 8 } }, //RET NZ
      0xC1 => { let bc = self.pop(); self.registers.set_bc(bc); 12 }, //POP BC
      0xC2 => { if !self.registers.get_flag(CpuFlag::Z) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP NZ,nn
      0xC3 => { self.registers.pc = self.fetch_word(); 12 }, //JUMP nn
      0xC4 => { if !self.registers.get_flag(CpuFlag::Z) { let address = self.fetch_word(); self.call(address); 24 } else { self.registers.pc += 2; 12 } }, //CALL NZ,nn
      0xC5 => { self.push(self.registers.get_bc()); 16 } //PUSH BC
      0xC6 => { let next_byte = self.fetch_byte(); self.alu_add(next_byte); 8 }, //ADD A,n
      0xC7 => { self.call(0x0000); 16 }, //RST 00H
      0xC8 => { if self.registers.get_flag(CpuFlag::Z) { self.retrn(); 20 } else { 8 } }, //RET Z
      0xC9 => { self.retrn(); 16 }, //RET
      0xCA => { if self.registers.get_flag(CpuFlag::Z) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP Z,nn
      0xCB => { self.op_code_cb() + 4 }, //CB
      0xCC => { if self.registers.get_flag(CpuFlag::Z) { let address = self.fetch_word(); self.call(address); 24 } else { self.registers.pc += 2; 12 } }, //CALL Z,nn
      0xCD => { let address = self.fetch_word(); self.call(address); 24 }, //CALL a16
      0xCE => { let next_byte = self.fetch_byte(); self.alu_adc(next_byte); 8 }, //ADC A,n
      0xCF => { self.call(0x0008); 16 }, //RST 08H
      0xD0 => { if !self.registers.get_flag(CpuFlag::Z) { self.retrn(); 20 } else { 8 } }, //RET NZ
      0xD1 => { let de = self.pop(); self.registers.set_de(de); 12 } //POP DE
      0xD2 => { if !self.registers.get_flag(CpuFlag::C) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP NC,nn
    //0xD3
      0xD4 => { !if self.registers.get_flag(CpuFlag::C) { let address = self.fetch_word(); self.call(address); 24 } else { self.registers.pc += 2; 12 } }, //CALL NC,nn
      0xD5 => { self.push(self.registers.get_de()); 16 }, //PUSH DE
      0xD6 => { let next_byte = self.fetch_byte(); self.alu_sub(next_byte); 8 }, //SUB n
      0xD7 => { self.call(0x0010); 16 }, //RST 10H
      0xD8 => { if self.registers.get_flag(CpuFlag::Z) { self.retrn(); 20 } else { 8 } }, //RET C
      0xD9 => { self.ime = true; self.retrn(); 16 }, //RETI (return and enable interrupts)
      0xDA => { if self.registers.get_flag(CpuFlag::C) { self.registers.pc = self.fetch_word(); 16 } else { self.registers.pc += 2; 12 } }, //JP C,nn
    //0xDB
      0xDC => { if self.registers.get_flag(CpuFlag::C) { let address = self.fetch_word(); self.call(address); 24 } else { self.registers.pc += 2; 12 } }, //CALL Z,nn
    //0xDD
      0xDE => { let next_byte = self.fetch_byte(); self.alu_sbc(next_byte); 8 }, //SBC A,n
      0xDF => { self.call(0x0018); 16 }, //RST 18H
      0xE0 => { let address = 0xFF00 + self.fetch_byte() as u16; self.mmu.write_byte(address, self.registers.a); 12 }, //LDH (n),
      0xE1 => { let hl = self.pop(); self.registers.set_hl(hl); 12 } //POP HL
      0xE2 => { self.mmu.write_byte(0xFF00 + self.registers.c as u16, self.registers.a);  8 }, //LD (C),A
    //0xE3
    //0xE4
      0xE5 => { self.push(self.registers.get_hl()); 16 } //PUSH HL
      0xE6 => { let next_byte = self.fetch_byte(); self.alu_and(next_byte); 8 } //AND n
      0xE7 => { self.call(0x0020); 16 }, //RST 20H
      0xE8 => { self.registers.sp = self.alu_add_next_signed_byte_to_word(self.registers.sp); 16 }, //ADD SP,r8
      0xE9 => { self.registers.pc = self.registers.get_hl(); 4 }, //JP HL
      0xEA => { let address = self.fetch_word(); self.mmu.write_byte(address, self.registers.a); 16 }, //LD (nn),A
    //0xEB
    //0xEC
    //0xED
      0xEE => { let next_byte = self.fetch_byte(); self.alu_xor(next_byte); 8 }, //XOR n
      0xEF => { self.call(0x0028); 16 }, //RST 28H
      0xF0 => { let address = 0xFF00 + self.fetch_byte() as u16; self.registers.a = self.mmu.read_byte(address); 12 }, //LDH A,(n)
      0xF1 => { let af = self.pop(); self.registers.set_af(af); 12 } //POP AF
      0xF2 => { self.registers.a = self.mmu.read_byte(0xFF00 + self.registers.c as u16);  8 }, //LD A,(C)
      0xF3 => { self.ime = false; self.ei_requested = 0; 4 }, //DI disable interrupts
    //0xF4
      0xF5 => { self.push(self.registers.get_af()); 16 }, //PUSH AF
      0xF6 => { let next_byte = self.fetch_byte(); self.alu_or(next_byte); 8 } //OR n
      0xF7 => { self.call(0x0030); 16 }, //RST 30H
      0xF8 => { let value = self.alu_add_next_signed_byte_to_word(self.registers.sp); self.registers.set_hl(value); 12 }, //LD HL, SP+r8
      0xF9 => { self.registers.sp = self.registers.get_hl(); 8 }, //LD SP,HL
      0xFA => { let address = self.fetch_word(); self.registers.a = self.mmu.read_byte(address); 16 }, //LD A,(nn)
      0xFB => { self.ei_requested = 2; 4 }, //EI enable interrupts
    //0xFC
    //0xFD
      0xFE => { let next_byte = self.fetch_byte(); self.alu_cp(next_byte); 8 }, //CP A,n  compare a-n
      0xFF => { self.call(0x0038); 16 }, //RST 0x0038
      _ => { println!("Unknown command {:#04X} at {:#06X}", op_code, current_address); self.halted = true; 4 } //NOOP on unknown opcodes
    }
  }

  fn op_code_cb(&mut self) -> usize {
    let op_code = self.fetch_byte();

    match op_code {
      0x00 => { self.registers.b = self.alu_rlc(self.registers.b); 8 }, //RLC B
      0x01 => { self.registers.c = self.alu_rlc(self.registers.c); 8 }, //RLC C
      0x02 => { self.registers.d = self.alu_rlc(self.registers.d); 8 }, //RLC D
      0x03 => { self.registers.e = self.alu_rlc(self.registers.e); 8 }, //RLC E
      0x04 => { self.registers.h = self.alu_rlc(self.registers.h); 8 }, //RLC H
      0x05 => { self.registers.l = self.alu_rlc(self.registers.l); 8 }, //RLC L
      0x06 => { let new_value = self.alu_rlc(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RLC (HL)
      0x07 => { self.registers.a = self.alu_rlc(self.registers.a); 8 }, //RLC A
      0x08 => { self.registers.b = self.alu_rrc(self.registers.b); 8 }, //RRC B
      0x09 => { self.registers.c = self.alu_rrc(self.registers.c); 8 }, //RRC C
      0x0A => { self.registers.d = self.alu_rrc(self.registers.d); 8 }, //RRC D
      0x0B => { self.registers.e = self.alu_rrc(self.registers.e); 8 }, //RRC E
      0x0C => { self.registers.h = self.alu_rrc(self.registers.h); 8 }, //RRC H
      0x0D => { self.registers.l = self.alu_rrc(self.registers.l); 8 }, //RRC L
      0x0E => { let new_value = self.alu_rrc(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RRC (HL)
      0x0F => { self.registers.a = self.alu_rrc(self.registers.a); 8 }, //RRC A
      0x10 => { self.registers.b = self.alu_rl(self.registers.b); 8 }, //RL B
      0x11 => { self.registers.c = self.alu_rl(self.registers.c); 8 }, //RL C
      0x12 => { self.registers.d = self.alu_rl(self.registers.d); 8 }, //RL D
      0x13 => { self.registers.e = self.alu_rl(self.registers.e); 8 }, //RL E
      0x14 => { self.registers.h = self.alu_rl(self.registers.h); 8 }, //RL H
      0x15 => { self.registers.l = self.alu_rl(self.registers.l); 8 }, //RL L
      0x16 => { let new_value = self.alu_rl(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RL (HL)
      0x17 => { self.registers.a = self.alu_rl(self.registers.a); 8 }, //RL A
      0x18 => { self.registers.b = self.alu_rr(self.registers.b); 8 }, //RR B
      0x19 => { self.registers.c = self.alu_rr(self.registers.c); 8 }, //RR C
      0x1A => { self.registers.d = self.alu_rr(self.registers.d); 8 }, //RR D
      0x1B => { self.registers.e = self.alu_rr(self.registers.e); 8 }, //RR E
      0x1C => { self.registers.h = self.alu_rr(self.registers.h); 8 }, //RR H
      0x1D => { self.registers.l = self.alu_rr(self.registers.l); 8 }, //RR L
      0x1E => { let new_value = self.alu_rr(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RR (HL)
      0x1F => { self.registers.a = self.alu_rr(self.registers.a); 8 }, //RR A
      0x20 => { self.registers.b = self.alu_sla(self.registers.b); 8 }, //SLA B
      0x21 => { self.registers.c = self.alu_sla(self.registers.c); 8 }, //SLA C
      0x22 => { self.registers.d = self.alu_sla(self.registers.d); 8 }, //SLA D
      0x23 => { self.registers.e = self.alu_sla(self.registers.e); 8 }, //SLA E
      0x24 => { self.registers.h = self.alu_sla(self.registers.h); 8 }, //SLA H
      0x25 => { self.registers.l = self.alu_sla(self.registers.l); 8 }, //SLA L
      0x26 => { let new_value = self.alu_sla(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SLA (HL)
      0x27 => { self.registers.a = self.alu_sla(self.registers.a); 8 }, //SLA A
      0x28 => { self.registers.b = self.alu_sra(self.registers.b); 8 }, //SRA B
      0x29 => { self.registers.c = self.alu_sra(self.registers.c); 8 }, //SRA C
      0x2A => { self.registers.d = self.alu_sra(self.registers.d); 8 }, //SRA D
      0x2B => { self.registers.e = self.alu_sra(self.registers.e); 8 }, //SRA E
      0x2C => { self.registers.h = self.alu_sra(self.registers.h); 8 }, //SRA H
      0x2D => { self.registers.l = self.alu_sra(self.registers.l); 8 }, //SRA L
      0x2E => { let new_value = self.alu_sra(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SRA (HL)
      0x2F => { self.registers.a = self.alu_sra(self.registers.a); 8 }, //SRA A
      0x30 => { self.registers.b = self.alu_swap(self.registers.b); 8 }, //SWAP B
      0x31 => { self.registers.c = self.alu_swap(self.registers.c); 8 }, //SWAP C
      0x32 => { self.registers.d = self.alu_swap(self.registers.d); 8 }, //SWAP D
      0x33 => { self.registers.e = self.alu_swap(self.registers.e); 8 }, //SWAP E
      0x34 => { self.registers.h = self.alu_swap(self.registers.h); 8 }, //SWAP H
      0x35 => { self.registers.l = self.alu_swap(self.registers.l); 8 }, //SWAP L
      0x36 => { let new_value = self.alu_swap(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SWAP (HL)
      0x37 => { self.registers.a = self.alu_swap(self.registers.a); 8 }, //SWAP A
      0x38 => { self.registers.b = self.alu_srl(self.registers.b); 8 }, //SRL B
      0x39 => { self.registers.c = self.alu_srl(self.registers.c); 8 }, //SRL C
      0x3A => { self.registers.d = self.alu_srl(self.registers.d); 8 }, //SRL D
      0x3B => { self.registers.e = self.alu_srl(self.registers.e); 8 }, //SRL E
      0x3C => { self.registers.h = self.alu_srl(self.registers.h); 8 }, //SRL H
      0x3D => { self.registers.l = self.alu_srl(self.registers.l); 8 }, //SRL L
      0x3E => { let new_value = self.alu_srl(self.mmu.read_byte(self.registers.get_hl())); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SRL (HL)
      0x3F => { self.registers.a = self.alu_srl(self.registers.a); 8 }, //SRL A
      0x40 => { self.alu_bit(0, self.registers.b); 8 }, //BIT 0,B
      0x41 => { self.alu_bit(0, self.registers.c); 8 }, //BIT 0,C
      0x42 => { self.alu_bit(0, self.registers.d); 8 }, //BIT 0,D
      0x43 => { self.alu_bit(0, self.registers.e); 8 }, //BIT 0,E
      0x44 => { self.alu_bit(0, self.registers.h); 8 }, //BIT 0,H
      0x45 => { self.alu_bit(0, self.registers.l); 8 }, //BIT 0,L
      0x46 => { self.alu_bit(0, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 0,(HL)
      0x47 => { self.alu_bit(0, self.registers.a); 8 }, //BIT 0,A
      0x48 => { self.alu_bit(1, self.registers.b); 8 }, //BIT 1,B
      0x49 => { self.alu_bit(1, self.registers.c); 8 }, //BIT 1,C
      0x4A => { self.alu_bit(1, self.registers.d); 8 }, //BIT 1,D
      0x4B => { self.alu_bit(1, self.registers.e); 8 }, //BIT 1,E
      0x4C => { self.alu_bit(1, self.registers.h); 8 }, //BIT 1,H
      0x4D => { self.alu_bit(1, self.registers.l); 8 }, //BIT 1,L
      0x4E => { self.alu_bit(1, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 1,(HL)
      0x4F => { self.alu_bit(1, self.registers.a); 8 }, //BIT 1,A
      0x50 => { self.alu_bit(2, self.registers.b); 8 }, //BIT 2,B
      0x51 => { self.alu_bit(2, self.registers.c); 8 }, //BIT 2,C
      0x52 => { self.alu_bit(2, self.registers.d); 8 }, //BIT 2,D
      0x53 => { self.alu_bit(2, self.registers.e); 8 }, //BIT 2,E
      0x54 => { self.alu_bit(2, self.registers.h); 8 }, //BIT 2,H
      0x55 => { self.alu_bit(2, self.registers.l); 8 }, //BIT 2,L
      0x56 => { self.alu_bit(2, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 2,(HL)
      0x57 => { self.alu_bit(2, self.registers.a); 8 }, //BIT 2,A
      0x58 => { self.alu_bit(3, self.registers.b); 8 }, //BIT 3,B
      0x59 => { self.alu_bit(3, self.registers.c); 8 }, //BIT 3,C
      0x5A => { self.alu_bit(3, self.registers.d); 8 }, //BIT 3,D
      0x5B => { self.alu_bit(3, self.registers.e); 8 }, //BIT 3,E
      0x5C => { self.alu_bit(3, self.registers.h); 8 }, //BIT 3,H
      0x5D => { self.alu_bit(3, self.registers.l); 8 }, //BIT 3,L
      0x5E => { self.alu_bit(3, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 3,(HL)
      0x5F => { self.alu_bit(3, self.registers.a); 8 }, //BIT 3,A
      0x60 => { self.alu_bit(4, self.registers.b); 8 }, //BIT 4,B
      0x61 => { self.alu_bit(4, self.registers.c); 8 }, //BIT 4,C
      0x62 => { self.alu_bit(4, self.registers.d); 8 }, //BIT 4,D
      0x63 => { self.alu_bit(4, self.registers.e); 8 }, //BIT 4,E
      0x64 => { self.alu_bit(4, self.registers.h); 8 }, //BIT 4,H
      0x65 => { self.alu_bit(4, self.registers.l); 8 }, //BIT 4,L
      0x66 => { self.alu_bit(4, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 4,(HL)
      0x67 => { self.alu_bit(4, self.registers.a); 8 }, //BIT 4,A
      0x68 => { self.alu_bit(5, self.registers.b); 8 }, //BIT 5,B
      0x69 => { self.alu_bit(5, self.registers.c); 8 }, //BIT 5,C
      0x6A => { self.alu_bit(5, self.registers.d); 8 }, //BIT 5,D
      0x6B => { self.alu_bit(5, self.registers.e); 8 }, //BIT 5,E
      0x6C => { self.alu_bit(5, self.registers.h); 8 }, //BIT 5,H
      0x6D => { self.alu_bit(5, self.registers.l); 8 }, //BIT 5,L
      0x6E => { self.alu_bit(5, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 5,(HL)
      0x6F => { self.alu_bit(5, self.registers.a); 8 }, //BIT 5,A
      0x70 => { self.alu_bit(6, self.registers.b); 8 }, //BIT 6,B
      0x71 => { self.alu_bit(6, self.registers.c); 8 }, //BIT 6,C
      0x72 => { self.alu_bit(6, self.registers.d); 8 }, //BIT 6,D
      0x73 => { self.alu_bit(6, self.registers.e); 8 }, //BIT 6,E
      0x74 => { self.alu_bit(6, self.registers.h); 8 }, //BIT 6,H
      0x75 => { self.alu_bit(6, self.registers.l); 8 }, //BIT 6,L
      0x76 => { self.alu_bit(6, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 6,(HL)
      0x77 => { self.alu_bit(6, self.registers.a); 8 }, //BIT 6,A
      0x78 => { self.alu_bit(7, self.registers.b); 8 }, //BIT 7,B
      0x79 => { self.alu_bit(7, self.registers.c); 8 }, //BIT 7,C
      0x7A => { self.alu_bit(7, self.registers.d); 8 }, //BIT 7,D
      0x7B => { self.alu_bit(7, self.registers.e); 8 }, //BIT 7,E
      0x7C => { self.alu_bit(7, self.registers.h); 8 }, //BIT 7,H
      0x7D => { self.alu_bit(7, self.registers.l); 8 }, //BIT 7,L
      0x7E => { self.alu_bit(7, self.mmu.read_byte(self.registers.get_hl())); 8 }, //BIT 7,(HL)
      0x7F => { self.alu_bit(7, self.registers.a); 8 }, //BIT 7,A
      0x80 => { self.registers.b = self.registers.b & !(1 << 0); 8 }, //RES 0,B
      0x81 => { self.registers.c = self.registers.c & !(1 << 0); 8 }, //RES 0,C
      0x82 => { self.registers.d = self.registers.d & !(1 << 0); 8 }, //RES 0,D
      0x83 => { self.registers.e = self.registers.e & !(1 << 0); 8 }, //RES 0,E
      0x84 => { self.registers.h = self.registers.h & !(1 << 0); 8 }, //RES 0,H
      0x85 => { self.registers.l = self.registers.l & !(1 << 0); 8 }, //RES 0,L
      0x86 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 0); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 0,(HL)
      0x87 => { self.registers.a = self.registers.a & !(1 << 0); 8 }, //RES 0,A
      0x88 => { self.registers.b = self.registers.b & !(1 << 1); 8 }, //RES 1,B
      0x89 => { self.registers.c = self.registers.c & !(1 << 1); 8 }, //RES 1,C
      0x8A => { self.registers.d = self.registers.d & !(1 << 1); 8 }, //RES 1,D
      0x8B => { self.registers.e = self.registers.e & !(1 << 1); 8 }, //RES 1,E
      0x8C => { self.registers.h = self.registers.h & !(1 << 1); 8 }, //RES 1,H
      0x8D => { self.registers.l = self.registers.l & !(1 << 1); 8 }, //RES 1,L
      0x8E => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 1); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 1,(HL)
      0x8F => { self.registers.a = self.registers.a & !(1 << 1); 8 }, //RES 1,A
      0x90 => { self.registers.b = self.registers.b & !(1 << 2); 8 }, //RES 2,B
      0x91 => { self.registers.c = self.registers.c & !(1 << 2); 8 }, //RES 2,C
      0x92 => { self.registers.d = self.registers.d & !(1 << 2); 8 }, //RES 2,D
      0x93 => { self.registers.e = self.registers.e & !(1 << 2); 8 }, //RES 2,E
      0x94 => { self.registers.h = self.registers.h & !(1 << 2); 8 }, //RES 2,H
      0x95 => { self.registers.l = self.registers.l & !(1 << 2); 8 }, //RES 2,L
      0x96 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 2); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 2,(HL)
      0x97 => { self.registers.a = self.registers.a & !(1 << 2); 8 }, //RES 2,A
      0x98 => { self.registers.b = self.registers.b & !(1 << 3); 8 }, //RES 3,B
      0x99 => { self.registers.c = self.registers.c & !(1 << 3); 8 }, //RES 3,C
      0x9A => { self.registers.d = self.registers.d & !(1 << 3); 8 }, //RES 3,D
      0x9B => { self.registers.e = self.registers.e & !(1 << 3); 8 }, //RES 3,E
      0x9C => { self.registers.h = self.registers.h & !(1 << 3); 8 }, //RES 3,H
      0x9D => { self.registers.l = self.registers.l & !(1 << 3); 8 }, //RES 3,L
      0x9E => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 3); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 3,(HL)
      0x9F => { self.registers.a = self.registers.a & !(1 << 3); 8 }, //RES 3,A
      0xA0 => { self.registers.b = self.registers.b & !(1 << 4); 8 }, //RES 4,B
      0xA1 => { self.registers.c = self.registers.c & !(1 << 4); 8 }, //RES 4,C
      0xA2 => { self.registers.d = self.registers.d & !(1 << 4); 8 }, //RES 4,D
      0xA3 => { self.registers.e = self.registers.e & !(1 << 4); 8 }, //RES 4,E
      0xA4 => { self.registers.h = self.registers.h & !(1 << 4); 8 }, //RES 4,H
      0xA5 => { self.registers.l = self.registers.l & !(1 << 4); 8 }, //RES 4,L
      0xA6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 4); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 4,(HL)
      0xA7 => { self.registers.a = self.registers.a & !(1 << 4); 8 }, //RES 4,A
      0xA8 => { self.registers.b = self.registers.b & !(1 << 5); 8 }, //RES 5,B
      0xA9 => { self.registers.c = self.registers.c & !(1 << 5); 8 }, //RES 5,C
      0xAA => { self.registers.d = self.registers.d & !(1 << 5); 8 }, //RES 5,D
      0xAB => { self.registers.e = self.registers.e & !(1 << 5); 8 }, //RES 5,E
      0xAC => { self.registers.h = self.registers.h & !(1 << 5); 8 }, //RES 5,H
      0xAD => { self.registers.l = self.registers.l & !(1 << 5); 8 }, //RES 5,L
      0xAE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 5); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 5,(HL)
      0xAF => { self.registers.a = self.registers.a & !(1 << 5); 8 }, //RES 5,A
      0xB0 => { self.registers.b = self.registers.b & !(1 << 6); 8 }, //RES 6,B
      0xB1 => { self.registers.c = self.registers.c & !(1 << 6); 8 }, //RES 6,C
      0xB2 => { self.registers.d = self.registers.d & !(1 << 6); 8 }, //RES 6,D
      0xB3 => { self.registers.e = self.registers.e & !(1 << 6); 8 }, //RES 6,E
      0xB4 => { self.registers.h = self.registers.h & !(1 << 6); 8 }, //RES 6,H
      0xB5 => { self.registers.l = self.registers.l & !(1 << 6); 8 }, //RES 6,L
      0xB6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 6); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 6,(HL)
      0xB7 => { self.registers.a = self.registers.a & !(1 << 6); 8 }, //RES 6,A
      0xB8 => { self.registers.b = self.registers.b & !(1 << 7); 8 }, //RES 7,B
      0xB9 => { self.registers.c = self.registers.c & !(1 << 7); 8 }, //RES 7,C
      0xBA => { self.registers.d = self.registers.d & !(1 << 7); 8 }, //RES 7,D
      0xBB => { self.registers.e = self.registers.e & !(1 << 7); 8 }, //RES 7,E
      0xBC => { self.registers.h = self.registers.h & !(1 << 7); 8 }, //RES 7,H
      0xBD => { self.registers.l = self.registers.l & !(1 << 7); 8 }, //RES 7,L
      0xBE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  & !(1 << 7); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //RES 7,(HL)
      0xBF => { self.registers.a = self.registers.a & !(1 << 7); 8 }, //RES 7,A
      0xC0 => { self.registers.b = self.registers.b | (1 << 0); 8 }, //SET 0,B
      0xC1 => { self.registers.c = self.registers.c | (1 << 0); 8 }, //SET 0,C
      0xC2 => { self.registers.d = self.registers.d | (1 << 0); 8 }, //SET 0,D
      0xC3 => { self.registers.e = self.registers.e | (1 << 0); 8 }, //SET 0,E
      0xC4 => { self.registers.h = self.registers.h | (1 << 0); 8 }, //SET 0,H
      0xC5 => { self.registers.l = self.registers.l | (1 << 0); 8 }, //SET 0,L
      0xC6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 0); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 0,(HL)
      0xC7 => { self.registers.a = self.registers.a | (1 << 0); 8 }, //SET 0,A
      0xC8 => { self.registers.b = self.registers.b | (1 << 1); 8 }, //SET 1,B
      0xC9 => { self.registers.c = self.registers.c | (1 << 1); 8 }, //SET 1,C
      0xCA => { self.registers.d = self.registers.d | (1 << 1); 8 }, //SET 1,D
      0xCB => { self.registers.e = self.registers.e | (1 << 1); 8 }, //SET 1,E
      0xCC => { self.registers.h = self.registers.h | (1 << 1); 8 }, //SET 1,H
      0xCD => { self.registers.l = self.registers.l | (1 << 1); 8 }, //SET 1,L
      0xCE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 1); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 1,(HL)
      0xCF => { self.registers.a = self.registers.a | (1 << 1); 8 }, //SET 1,A
      0xD0 => { self.registers.b = self.registers.b | (1 << 2); 8 }, //SET 2,B
      0xD1 => { self.registers.c = self.registers.c | (1 << 2); 8 }, //SET 2,C
      0xD2 => { self.registers.d = self.registers.d | (1 << 2); 8 }, //SET 2,D
      0xD3 => { self.registers.e = self.registers.e | (1 << 2); 8 }, //SET 2,E
      0xD4 => { self.registers.h = self.registers.h | (1 << 2); 8 }, //SET 2,H
      0xD5 => { self.registers.l = self.registers.l | (1 << 2); 8 }, //SET 2,L
      0xD6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 2); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 2,(HL)
      0xD7 => { self.registers.a = self.registers.a | (1 << 2); 8 }, //SET 2,A
      0xD8 => { self.registers.b = self.registers.b | (1 << 3); 8 }, //SET 3,B
      0xD9 => { self.registers.c = self.registers.c | (1 << 3); 8 }, //SET 3,C
      0xDA => { self.registers.d = self.registers.d | (1 << 3); 8 }, //SET 3,D
      0xDB => { self.registers.e = self.registers.e | (1 << 3); 8 }, //SET 3,E
      0xDC => { self.registers.h = self.registers.h | (1 << 3); 8 }, //SET 3,H
      0xDD => { self.registers.l = self.registers.l | (1 << 3); 8 }, //SET 3,L
      0xDE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 3); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 3,(HL)
      0xDF => { self.registers.a = self.registers.a | (1 << 3); 8 }, //SET 3,A
      0xE0 => { self.registers.b = self.registers.b | (1 << 4); 8 }, //SET 4,B
      0xE1 => { self.registers.c = self.registers.c | (1 << 4); 8 }, //SET 4,C
      0xE2 => { self.registers.d = self.registers.d | (1 << 4); 8 }, //SET 4,D
      0xE3 => { self.registers.e = self.registers.e | (1 << 4); 8 }, //SET 4,E
      0xE4 => { self.registers.h = self.registers.h | (1 << 4); 8 }, //SET 4,H
      0xE5 => { self.registers.l = self.registers.l | (1 << 4); 8 }, //SET 4,L
      0xE6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 4); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 4,(HL)
      0xE7 => { self.registers.a = self.registers.a | (1 << 4); 8 }, //SET 4,A
      0xE8 => { self.registers.b = self.registers.b | (1 << 5); 8 }, //SET 5,B
      0xE9 => { self.registers.c = self.registers.c | (1 << 5); 8 }, //SET 5,C
      0xEA => { self.registers.d = self.registers.d | (1 << 5); 8 }, //SET 5,D
      0xEB => { self.registers.e = self.registers.e | (1 << 5); 8 }, //SET 5,E
      0xEC => { self.registers.h = self.registers.h | (1 << 5); 8 }, //SET 5,H
      0xED => { self.registers.l = self.registers.l | (1 << 5); 8 }, //SET 5,L
      0xEE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 5); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 5,(HL)
      0xEF => { self.registers.a = self.registers.a | (1 << 5); 8 }, //SET 5,A
      0xF0 => { self.registers.b = self.registers.b | (1 << 6); 8 }, //SET 6,B
      0xF1 => { self.registers.c = self.registers.c | (1 << 6); 8 }, //SET 6,C
      0xF2 => { self.registers.d = self.registers.d | (1 << 6); 8 }, //SET 6,D
      0xF3 => { self.registers.e = self.registers.e | (1 << 6); 8 }, //SET 6,E
      0xF4 => { self.registers.h = self.registers.h | (1 << 6); 8 }, //SET 6,H
      0xF5 => { self.registers.l = self.registers.l | (1 << 6); 8 }, //SET 6,L
      0xF6 => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 6); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 6,(HL)
      0xF7 => { self.registers.a = self.registers.a | (1 << 6); 8 }, //SET 6,A
      0xF8 => { self.registers.b = self.registers.b | (1 << 7); 8 }, //SET 7,B
      0xF9 => { self.registers.c = self.registers.c | (1 << 7); 8 }, //SET 7,C
      0xFA => { self.registers.d = self.registers.d | (1 << 7); 8 }, //SET 7,D
      0xFB => { self.registers.e = self.registers.e | (1 << 7); 8 }, //SET 7,E
      0xFC => { self.registers.h = self.registers.h | (1 << 7); 8 }, //SET 7,H
      0xFD => { self.registers.l = self.registers.l | (1 << 7); 8 }, //SET 7,L
      0xFE => { let new_value = self.mmu.read_byte(self.registers.get_hl())  | (1 << 7); self.mmu.write_byte(self.registers.get_hl(), new_value); 8 }, //SET 7,(HL)
      0xFF => { self.registers.a = self.registers.a | (1 << 7); 8 }, //SET 7,A
      _ => 4 //should not happen
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
    self.push(self.registers.pc); //it's not pc + 1 because after fetching the address the pc is already at the next instruction
    self.registers.pc = address;
  }

  fn retrn(&mut self) {
    self.registers.pc = self.pop();
  }

  fn alu_and(&mut self, value: u8) {
    self.registers.a &= value;

    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, self.registers.a == 0x00);
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
    self.registers.set_flag(CpuFlag::C, self.registers.a as usize + value as usize > 0xFF);
    self.registers.a = result;
  }

  fn alu_add16(&mut self, value: u16) {
    let result = self.registers.get_hl().wrapping_add(value);
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, (((self.registers.get_hl() & 0x00FF) + (value & 0x00FF)) & 0x0100) == 0x0100);
    self.registers.set_flag(CpuFlag::C, self.registers.get_hl() as usize + value as usize > 0xFFFF);
    self.registers.set_hl(result); //16bit add goes to hl
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

  fn alu_sbc(&mut self, value: u8) {
    let c: u8 = if self.registers.get_flag(CpuFlag::C) { 1 } else { 0 };
    let result = self.registers.a.wrapping_sub(value).wrapping_sub(c);
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::N, true);
    self.registers.set_flag(CpuFlag::H, (self.registers.a & 0x0F) < (value & 0x0F) + c);
    self.registers.set_flag(CpuFlag::C, self.registers.a < value + c);
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

  fn alu_shift_operation_flag_update(&mut self, result: u8, new_carry: bool) {
    self.registers.reset_flags();
    self.registers.set_flag(CpuFlag::Z, result == 0);
    self.registers.set_flag(CpuFlag::C, new_carry);
  }

  fn alu_rl(&mut self, value: u8) -> u8 { //rotate left through carry
    let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
    let result = (value << 1) | if self.registers.get_flag(CpuFlag::C) { 0x01 } else { 0x00 }; //push one to the right and add the carry to the right
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_rlc(&mut self, value: u8) -> u8 { //rotate left
    let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
    let result = (value << 1) | if new_carry { 0x01 } else { 0x00 }; //push one to the left and add the pushed out bit to the right
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_rr(&mut self, value: u8) -> u8 { //rotate right through carry
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1) | if self.registers.get_flag(CpuFlag::C) { 0x80 } else { 0x00 };
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_rrc(&mut self, value: u8) -> u8 { //rotate right
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1) | if new_carry { 0x80 } else { 0x00 };
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  //difference between shift and rotate is, that we don't add the pushed out bit on the other side
  fn alu_sla(&mut self, value: u8) -> u8 { //shift left arithmetic (b0=0)
    let new_carry = (value & 0x80) == 0x80;
    let result = value << 1;
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_sra(&mut self, value: u8) -> u8 { //shift left arithmetic (b0=0)
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1) | (value & 0x80);
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_srl(&mut self, value: u8) -> u8 { //shift left arithmetic (b0=0)
    let new_carry = (value & 0x01) == 0x01;
    let result = value >> 1;
    self.alu_shift_operation_flag_update(result, new_carry);
    result
  }

  fn alu_ccf(&mut self) { //compliment carry flag
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, false);
    self.registers.set_flag(CpuFlag::C, self.registers.get_flag(CpuFlag::C));
  }

  fn alu_scf(&mut self) { //set carry flag
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, false);
    self.registers.set_flag(CpuFlag::C, true);
  }

  fn alu_bit(&mut self, bit: u8, value: u8) { //check bit at
    self.registers.set_flag(CpuFlag::Z, (value & (1 << bit)) == 0 );
    self.registers.set_flag(CpuFlag::N, false);
    self.registers.set_flag(CpuFlag::H, true);
  }

  fn jump_r(&mut self) {
    let offset = self.fetch_byte();
    self.registers.pc = self.registers.pc.wrapping_add(offset as i8 as i16 as u16);
  }
}