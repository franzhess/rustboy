use crate::cpu::alu;
use crate::cpu::registers::{RegisterName8, RegisterName16, FlagRegister};
use crate::cpu::OpCodeResult::{Executed, UnknownOpCode };
use crate::cpu::registers::CpuFlag;
use crate::cpu::OpCodeResult;
use crate::cpu::Cpu;
use crate::cpu::op_codes_cb;

pub fn execute(op_code: u8, cpu: &mut Cpu) -> OpCodeResult {
  match op_code {
    0x00 => { Executed(4) }, //NOOP
    0x01 => { let next_word = cpu.fetch_word(); cpu.registers.set_bc(next_word); Executed(12) }, //LD BC,nn
    0x02 => { cpu.mmu.write_byte(cpu.registers.get_bc(), cpu.registers.a); Executed(8) }, //LD (BC),A
    0x03 => { cpu.registers.set_bc(cpu.registers.get_bc().wrapping_add(1)); Executed(8) }, //INC BC
    0x04 => { cpu.execute(alu::inc, RegisterName8::B); Executed(4) }, //INC B
    0x05 => { cpu.execute(alu::dec, RegisterName8::B); Executed(4) }, //DEC B
    0x06 => { cpu.registers.b = cpu.fetch_byte(); Executed(8) }, //LD B,n
    0x07 => { cpu.execute(alu::rlc, RegisterName8::A); Executed(4) }, //RLC A - rotate left a
    0x08 => { let address = cpu.fetch_word(); cpu.mmu.write_word(address, cpu.registers.sp); Executed(20) }, //LD (nn),SP
    0x09 => { cpu.execute16(alu::add16, RegisterName16::BC); Executed(8) }, //ADD HL,BC
    0x0A => { cpu.registers.a = cpu.mmu.read_byte(cpu.registers.get_bc()); Executed(8) }, //LD A,(BC)
    0x0B => { cpu.registers.set_bc(cpu.registers.get_bc().wrapping_sub(1)); Executed(8) }, //DEC BC - no flags set
    0x0C => { cpu.execute(alu::inc, RegisterName8::C); Executed(4) }, //INC C
    0x0D => { cpu.execute(alu::dec, RegisterName8::C); Executed(4) }, //DEC C
    0x0E => { cpu.registers.c = cpu.fetch_byte(); Executed(8) }, //LD C, n
    0x0F => { cpu.execute(alu::rrc, RegisterName8::A); Executed(4) }, //RRC A
    0x10 => { cpu.fetch_byte(); cpu.halted = true; Executed(4) }, //STOP @TODO implement resume on button press
    0x11 => { let next_word = cpu.fetch_word(); cpu.registers.set_de(next_word); Executed(12) }, //LD DE,nn
    0x12 => { cpu.mmu.write_byte(cpu.registers.get_de(), cpu.registers.a); Executed(8) }, //LD (DE),A
    0x13 => { cpu.registers.set_de(cpu.registers.get_de().wrapping_add(1)); Executed(8) }, //INC DE
    0x14 => { cpu.execute(alu::inc, RegisterName8::D); Executed(4) }, //INC D
    0x15 => { cpu.execute(alu::dec, RegisterName8::D); Executed(4) }, //DEC D
    0x16 => { cpu.registers.d = cpu.fetch_byte(); Executed(8) }, //LD D,n
    0x17 => { cpu.execute(alu::rl, RegisterName8::A); Executed(4) }, //RL A - rotate left through carry
    0x18 => { cpu.jump_r(); Executed(12) }, //JR n
    0x19 => { cpu.execute16(alu::add16, RegisterName16::DE); Executed(8) }, //ADD HL,DE
    0x1A => { cpu.registers.a = cpu.mmu.read_byte(cpu.registers.get_de()); Executed(8) }, //LD A,(DE)
    0x1B => { cpu.registers.set_de(cpu.registers.get_de().wrapping_sub(1)); Executed(8) }, //DEC DE - no flags set
    0x1C => { cpu.execute(alu::inc, RegisterName8::E); Executed(4) }, //INC E
    0x1D => { cpu.execute(alu::dec, RegisterName8::E); Executed(4) }, //DEC E
    0x1E => { cpu.registers.e = cpu.fetch_byte(); Executed(8) }, //LD E,n
    0x1F => { cpu.execute(alu::rr, RegisterName8::A); Executed(4) }, //RRA
    0x20 => { if !cpu.registers.get_flag(CpuFlag::Z) { cpu.jump_r(); Executed(12) } else { cpu.registers.pc += 1; Executed(8) } }, //JR NZ,n
    0x21 => { let next_word = cpu.fetch_word(); cpu.registers.set_hl(next_word); Executed(12) }, //LD HL,nn
    0x22 => { cpu.mmu.write_byte(cpu.registers.get_hli(), cpu.registers.a); Executed(12) }, //LD (HL+),A
    0x23 => { cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1)); Executed(8) }, //INC HL
    0x24 => { cpu.execute(alu::inc, RegisterName8::H); Executed(4) }, //INC H
    0x25 => { cpu.execute(alu::dec, RegisterName8::H); Executed(4) }, //DEC H
    0x26 => { cpu.registers.h = cpu.fetch_byte(); Executed(8) }, //LD H,n
    0x27 => { cpu.execute(alu::daa, RegisterName8::A); Executed(4) }, //DAA
    0x28 => { if cpu.registers.get_flag(CpuFlag::Z) { cpu.jump_r(); Executed(12) } else { cpu.registers.pc += 1; Executed(8) } }, //JR Z,n
    0x29 => { cpu.execute16(alu::add16, RegisterName16::HL); Executed(8) }, //ADD HL,HL
    0x2A => { cpu.registers.a = cpu.mmu.read_byte(cpu.registers.get_hli()); Executed(8) }, //LD A,(HL+)
    0x2B => { cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1)); Executed(8) }, //DEC HL - no flags set
    0x2C => { cpu.execute(alu::inc, RegisterName8::L); Executed(4) }, //INC L
    0x2D => { cpu.execute(alu::dec, RegisterName8::L); Executed(4)}, //DEC L
    0x2E => { cpu.registers.l = cpu.fetch_byte(); Executed(8) }, //LD L,n
    0x2F => { cpu.execute(alu::cpl, RegisterName8::A); Executed(4) } //CPL - A=A XOR FF - method for flags
    0x30 => { if !cpu.registers.get_flag(CpuFlag::C) { cpu.jump_r(); Executed(12) } else { cpu.registers.pc += 1; Executed(8) } }, //JR NC,n
    0x31 => { cpu.registers.sp = cpu.fetch_word(); Executed(12) }, //LD SP,nn
    0x32 => { cpu.mmu.write_byte(cpu.registers.get_hld(), cpu.registers.a); Executed(8) }, //LD (HL-),A
    0x33 => { cpu.registers.sp = cpu.registers.sp.wrapping_add(1); Executed(8) }, //INC SP
    0x34 => { let address = cpu.registers.get_hl(); let inc_byte = alu::inc(&mut cpu.registers, cpu.mmu.read_byte(address)); cpu.mmu.write_byte(address, inc_byte); Executed(12) }, //INC (HL)
    0x35 => { let address = cpu.registers.get_hl(); let dec_byte = alu::dec(&mut cpu.registers, cpu.mmu.read_byte(address)); cpu.mmu.write_byte(address, dec_byte); Executed(12) }, //DEC (HL)
    0x36 => { let next_byte = cpu.fetch_byte(); cpu.mmu.write_byte(cpu.registers.get_hl(), next_byte); Executed(12) }, //LD (HL),n
    0x37 => { alu::scf(&mut cpu.registers); Executed(4) }, //SCF
    0x38 => { if cpu.registers.get_flag(CpuFlag::C) { cpu.jump_r(); Executed(12) } else { cpu.registers.pc += 1; Executed(8) } }, //JR C,n
    0x39 => { cpu.execute16(alu::add16, RegisterName16::SP); Executed(8) } //ADD HL,SP
    0x3A => { cpu.registers.a = cpu.mmu.read_byte(cpu.registers.get_hld()); Executed(8) }, //LD A,(HL-)
    0x3B => { cpu.registers.sp = cpu.registers.sp.wrapping_sub(1); Executed(8) }, //DEC SP - no flags set
    0x3C => { cpu.execute(alu::inc, RegisterName8::A); Executed(4) }, //INC A
    0x3D => { cpu.execute(alu::dec, RegisterName8::A); Executed(4) }, //DEC A
    0x3E => { cpu.registers.a = cpu.fetch_byte(); Executed(8) }, //LD A,n
    0x3F => { alu::ccf(&mut cpu.registers);  Executed(4) }, //CCF - flip carry
    0x40 => { Executed(4) }, //LD B,B
    0x41 => { cpu.registers.b = cpu.registers.c; Executed(4) }, //LD B,C
    0x42 => { cpu.registers.b = cpu.registers.d; Executed(4) }, //LD B,D
    0x43 => { cpu.registers.b = cpu.registers.e; Executed(4) }, //LD B,E
    0x44 => { cpu.registers.b = cpu.registers.h; Executed(4) }, //LD B,H
    0x45 => { cpu.registers.b = cpu.registers.l; Executed(4) }, //LD B,L
    0x46 => { cpu.registers.b = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD B,(HL)
    0x47 => { cpu.registers.b = cpu.registers.a; Executed(4) }, //LD B,A
    0x48 => { cpu.registers.c = cpu.registers.b; Executed(4) }, //LD C,B
    0x49 => { Executed(4) }, //LD C,C
    0x4A => { cpu.registers.c = cpu.registers.d; Executed(4) }, //LD C,D
    0x4B => { cpu.registers.c = cpu.registers.e; Executed(4) }, //LD C,E
    0x4C => { cpu.registers.c = cpu.registers.h; Executed(4) }, //LD C,H
    0x4D => { cpu.registers.c = cpu.registers.l; Executed(4) }, //LD C,L
    0x4E => { cpu.registers.c = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD C,(HL)
    0x4F => { cpu.registers.c = cpu.registers.a; Executed(4) }, //LD C,A
    0x50 => { cpu.registers.d = cpu.registers.b; Executed(4) }, //LD D,B
    0x51 => { cpu.registers.d = cpu.registers.c; Executed(4) }, //LD D,C
    0x52 => { Executed(4) }, //LD D,D
    0x53 => { cpu.registers.d = cpu.registers.e; Executed(4) }, //LD D,E
    0x54 => { cpu.registers.d = cpu.registers.h; Executed(4) }, //LD D,H
    0x55 => { cpu.registers.d = cpu.registers.l; Executed(4) }, //LD D,L
    0x56 => { cpu.registers.d = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD D,(HL)
    0x57 => { cpu.registers.d = cpu.registers.a; Executed(4) }, //LD D,A
    0x58 => { cpu.registers.e = cpu.registers.b; Executed(4) }, //LD E,B
    0x59 => { cpu.registers.e = cpu.registers.c; Executed(4) }, //LD E,C
    0x5A => { cpu.registers.e = cpu.registers.d; Executed(4) }, //LD E,D
    0x5B => { Executed(4) }, //LD E,E
    0x5C => { cpu.registers.e = cpu.registers.h; Executed(4) }, //LD E,H
    0x5D => { cpu.registers.e = cpu.registers.l; Executed(4) }, //LD E,L
    0x5E => { cpu.registers.e = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD E,(HL)
    0x5F => { cpu.registers.e = cpu.registers.a; Executed(4) }, //LD E,A
    0x60 => { cpu.registers.h = cpu.registers.b; Executed(4) }, //LD H,B
    0x61 => { cpu.registers.h = cpu.registers.c; Executed(4) }, //LD H,C
    0x62 => { cpu.registers.h = cpu.registers.d; Executed(4) }, //LD H,D
    0x63 => { cpu.registers.h = cpu.registers.e; Executed(4) }, //LD H,E
    0x64 => { Executed(4) }, //LD H,H
    0x65 => { cpu.registers.h = cpu.registers.l; Executed(4) }, //LD H,L
    0x66 => { cpu.registers.h = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD H,(HL)
    0x67 => { cpu.registers.h = cpu.registers.a; Executed(4) }, //LD H,A
    0x68 => { cpu.registers.l = cpu.registers.b; Executed(4) }, //LD L,B
    0x69 => { cpu.registers.l = cpu.registers.c; Executed(4) }, //LD L,C
    0x6A => { cpu.registers.l = cpu.registers.d; Executed(4) }, //LD L,D
    0x6B => { cpu.registers.l = cpu.registers.e; Executed(4) }, //LD L,E
    0x6C => { cpu.registers.l = cpu.registers.h; Executed(4) }, //LD L,H
    0x6D => { Executed(4) }, //LD L,L
    0x6E => { cpu.registers.l = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD L,(HL)
    0x6F => { cpu.registers.l = cpu.registers.a; Executed(4) }, //LD L,A
    0x70 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.b); Executed(8) }, //LD (HL),B
    0x71 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.c); Executed(8) }, //LD (HL),C
    0x72 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.d); Executed(8) }, //LD (HL),D
    0x73 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.e); Executed(8) }, //LD (HL),E
    0x74 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.h); Executed(8) }, //LD (HL),H
    0x75 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.l); Executed(8) }, //LD (HL),L
    0x76 => { cpu.halted = true; Executed(4) }, //HALT
    0x77 => { cpu.mmu.write_byte(cpu.registers.get_hl(), cpu.registers.a); Executed(8) }, //LD (HL),A
    0x78 => { cpu.registers.a = cpu.registers.b; Executed(4) }, //LD A,B
    0x79 => { cpu.registers.a = cpu.registers.c; Executed(4) }, //LD A,C
    0x7A => { cpu.registers.a = cpu.registers.d; Executed(4) }, //LD A,D
    0x7B => { cpu.registers.a = cpu.registers.e; Executed(4) }, //LD A,E
    0x7C => { cpu.registers.a = cpu.registers.h; Executed(4) }, //LD A,H
    0x7D => { cpu.registers.a = cpu.registers.l; Executed(4) }, //LD A,L
    0x7E => { cpu.registers.a = cpu.mmu.read_byte(cpu.registers.get_hl()); Executed(8) }, //LD A,(HL)
    0x7F => { Executed(4) }, //LD A,A
    0x80 => { cpu.execute_binary(alu::add, RegisterName8::B); Executed(4) }, //ADD A,B
    0x81 => { cpu.execute_binary(alu::add, RegisterName8::C); Executed(4) }, //ADD A,C
    0x82 => { cpu.execute_binary(alu::add, RegisterName8::D); Executed(4) }, //ADD A,D
    0x83 => { cpu.execute_binary(alu::add, RegisterName8::E); Executed(4) }, //ADD A,E
    0x84 => { cpu.execute_binary(alu::add, RegisterName8::H); Executed(4) }, //ADD A,H
    0x85 => { cpu.execute_binary(alu::add, RegisterName8::L); Executed(4) }, //ADD A,L
    0x86 => { cpu.execute_binary_with_value(alu::add, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(4) }, //ADD A,(HL)
    0x87 => { cpu.execute_binary(alu::add, RegisterName8::A); Executed(4) }, //ADD A,A
    0x88 => { cpu.execute_binary(alu::adc, RegisterName8::B); Executed(4) },  //ADC A,B
    0x89 => { cpu.execute_binary(alu::adc, RegisterName8::C); Executed(4) },  //ADC A,C
    0x8A => { cpu.execute_binary(alu::adc, RegisterName8::D); Executed(4) },  //ADC A,D
    0x8B => { cpu.execute_binary(alu::adc, RegisterName8::E); Executed(4) },  //ADC A,E
    0x8C => { cpu.execute_binary(alu::adc, RegisterName8::H); Executed(4) },  //ADC A,H
    0x8D => { cpu.execute_binary(alu::adc, RegisterName8::L); Executed(4) },  //ADC A,L
    0x8E => { cpu.execute_binary_with_value(alu::adc, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) },  //ADC A,(HL)
    0x8F => { cpu.execute_binary(alu::adc, RegisterName8::A); Executed(4) },  //ADC A,A
    0x90 => { cpu.execute_binary(alu::sub, RegisterName8::B); Executed(4) }, //SUB A,B
    0x91 => { cpu.execute_binary(alu::sub, RegisterName8::C); Executed(4) }, //SUB A,C
    0x92 => { cpu.execute_binary(alu::sub, RegisterName8::D); Executed(4) }, //SUB A,D
    0x93 => { cpu.execute_binary(alu::sub, RegisterName8::E); Executed(4) }, //SUB A,E
    0x94 => { cpu.execute_binary(alu::sub, RegisterName8::H); Executed(4) }, //SUB A,H
    0x95 => { cpu.execute_binary(alu::sub, RegisterName8::L); Executed(4) }, //SUB A,L
    0x96 => { cpu.execute_binary_with_value(alu::sub, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(4) }, //SUB A,(HL)
    0x97 => { cpu.execute_binary(alu::sub, RegisterName8::A); Executed(4) }, //SUB A,A
    0x98 => { cpu.execute_binary(alu::sbc, RegisterName8::B); Executed(4) },  //SBC A,B
    0x99 => { cpu.execute_binary(alu::sbc, RegisterName8::C); Executed(4) },  //SBC A,C
    0x9A => { cpu.execute_binary(alu::sbc, RegisterName8::D); Executed(4) },  //SBC A,D
    0x9B => { cpu.execute_binary(alu::sbc, RegisterName8::E); Executed(4) },  //SBC A,E
    0x9C => { cpu.execute_binary(alu::sbc, RegisterName8::H); Executed(4) },  //SBC A,H
    0x9D => { cpu.execute_binary(alu::sbc, RegisterName8::L); Executed(4) },  //SBC A,L
    0x9E => { cpu.execute_binary_with_value(alu::sbc, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) },  //SBC A,(HL)
    0x9F => { cpu.execute_binary(alu::sbc, RegisterName8::A); Executed(4) },  //SBC A,A
    0xA0 => { cpu.execute_binary(alu::and, RegisterName8::B); Executed(4) }, //AND B
    0xA1 => { cpu.execute_binary(alu::and, RegisterName8::C); Executed(4) }, //AND C
    0xA2 => { cpu.execute_binary(alu::and, RegisterName8::D); Executed(4) }, //AND D
    0xA3 => { cpu.execute_binary(alu::and, RegisterName8::E); Executed(4) }, //AND E
    0xA4 => { cpu.execute_binary(alu::and, RegisterName8::H); Executed(4) }, //AND H
    0xA5 => { cpu.execute_binary(alu::and, RegisterName8::L); Executed(4) }, //AND L
    0xA6 => { cpu.execute_binary_with_value(alu::and, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //AND (HL)
    0xA7 => { cpu.execute_binary(alu::and, RegisterName8::A); Executed(4) }, //AND A
    0xA8 => { cpu.execute_binary(alu::xor, RegisterName8::B); Executed(4) }, //XOR B
    0xA9 => { cpu.execute_binary(alu::xor, RegisterName8::C); Executed(4) }, //XOR C
    0xAA => { cpu.execute_binary(alu::xor, RegisterName8::D); Executed(4) }, //XOR D
    0xAB => { cpu.execute_binary(alu::xor, RegisterName8::E); Executed(4) }, //XOR E
    0xAC => { cpu.execute_binary(alu::xor, RegisterName8::H); Executed(4) }, //XOR H
    0xAD => { cpu.execute_binary(alu::xor, RegisterName8::L); Executed(4) }, //XOR L
    0xAE => { cpu.execute_binary_with_value(alu::xor, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //XOR (HL)
    0xAF => { cpu.execute_binary(alu::xor, RegisterName8::A); Executed(4) }, //XOR A
    0xB0 => { cpu.execute_binary(alu::or, RegisterName8::B); Executed(4) }, //OR B
    0xB1 => { cpu.execute_binary(alu::or, RegisterName8::C); Executed(4) }, //OR C
    0xB2 => { cpu.execute_binary(alu::or, RegisterName8::D); Executed(4) }, //OR D
    0xB3 => { cpu.execute_binary(alu::or, RegisterName8::E); Executed(4) }, //OR E
    0xB4 => { cpu.execute_binary(alu::or, RegisterName8::H); Executed(4) }, //OR H
    0xB5 => { cpu.execute_binary(alu::or, RegisterName8::L); Executed(4) }, //OR L
    0xB6 => { cpu.execute_binary_with_value(alu::or, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //OR (HL)
    0xB7 => { cpu.execute_binary(alu::or, RegisterName8::A); Executed(4) }, //OR A
    0xB8 => { cpu.execute_binary(alu::cp, RegisterName8::B); Executed(4) }, //CP B
    0xB9 => { cpu.execute_binary(alu::cp, RegisterName8::C); Executed(4) }, //CP C
    0xBA => { cpu.execute_binary(alu::cp, RegisterName8::D); Executed(4) }, //CP D
    0xBB => { cpu.execute_binary(alu::cp, RegisterName8::E); Executed(4) }, //CP E
    0xBC => { cpu.execute_binary(alu::cp, RegisterName8::H); Executed(4) }, //CP H
    0xBD => { cpu.execute_binary(alu::cp, RegisterName8::L); Executed(4) }, //CP L
    0xBE => { cpu.execute_binary_with_value(alu::cp, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(4) }, //CP (HL)
    0xBF => { cpu.execute_binary(alu::cp, RegisterName8::A); Executed(4) }, //CP A
    0xC0 => { if !cpu.registers.get_flag(CpuFlag::Z) { cpu.retrn(); Executed(20) } else { Executed(8) } }, //RET NZ
    0xC1 => { let bc = cpu.pop(); cpu.registers.set_bc(bc); Executed(12) }, //POP BC
    0xC2 => { if !cpu.registers.get_flag(CpuFlag::Z) { cpu.registers.pc = cpu.fetch_word(); Executed(16) } else { cpu.registers.pc += 2; Executed(12) } }, //JP NZ,nn
    0xC3 => { cpu.registers.pc = cpu.fetch_word(); Executed(12) }, //JUMP nn
    0xC4 => { if !cpu.registers.get_flag(CpuFlag::Z) { let address = cpu.fetch_word(); cpu.call(address); Executed(24) } else { cpu.registers.pc += 2; Executed(12) } }, //CALL NZ,nn
    0xC5 => { cpu.push(cpu.registers.get_bc()); Executed(16) } //PUSH BC
    0xC6 => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::add,next_byte); Executed(8) }, //ADD A,n
    0xC7 => { cpu.call(0x0000); Executed(16) }, //RST 00H
    0xC8 => { if cpu.registers.get_flag(CpuFlag::Z) { cpu.retrn(); Executed(20) } else { Executed(8) } }, //RET Z
    0xC9 => { cpu.retrn(); Executed(16) }, //RET
    0xCA => { if cpu.registers.get_flag(CpuFlag::Z) { cpu.registers.pc = cpu.fetch_word(); Executed(16) } else { cpu.registers.pc += 2; Executed(12) } }, //JP Z,nn
    0xCB => { let op = cpu.fetch_byte(); op_codes_cb::execute(op, cpu) }, //CB
    0xCC => { if cpu.registers.get_flag(CpuFlag::Z) { let address = cpu.fetch_word(); cpu.call(address); Executed(24) } else { cpu.registers.pc += 2; Executed(12) } }, //CALL Z,nn
    0xCD => { let address = cpu.fetch_word(); cpu.call(address); Executed(24) }, //CALL a16
    0xCE => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::adc, next_byte); Executed(8) }, //ADC A,n
    0xCF => { cpu.call(0x0008); Executed(16) }, //RST 08H
    0xD0 => { if !cpu.registers.get_flag(CpuFlag::C) { cpu.retrn(); Executed(20) } else { Executed(8) } }, //RET NC
    0xD1 => { let de = cpu.pop(); cpu.registers.set_de(de); Executed(12) } //POP DE
    0xD2 => { if !cpu.registers.get_flag(CpuFlag::C) { cpu.registers.pc = cpu.fetch_word(); Executed(16) } else { cpu.registers.pc += 2; Executed(12) } }, //JP NC,nn
    //0xD3
    0xD4 => { if !cpu.registers.get_flag(CpuFlag::C) { let address = cpu.fetch_word(); cpu.call(address); Executed(24) } else { cpu.registers.pc += 2; Executed(12) } }, //CALL NC,nn
    0xD5 => { cpu.push(cpu.registers.get_de()); Executed(16) }, //PUSH DE
    0xD6 => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::add, next_byte); Executed(8) }, //SUB A,n
    0xD7 => { cpu.call(0x0010); Executed(16) }, //RST 10H
    0xD8 => { if cpu.registers.get_flag(CpuFlag::C) { cpu.retrn(); Executed(20) } else { Executed(8) } }, //RET C
    0xD9 => { cpu.ime = true; cpu.retrn(); Executed(16) }, //RETI (return and enable interrupts)
    0xDA => { if cpu.registers.get_flag(CpuFlag::C) { cpu.registers.pc = cpu.fetch_word(); Executed(16) } else { cpu.registers.pc += 2; Executed(12) } }, //JP C,nn
    //0xDB
    0xDC => { if cpu.registers.get_flag(CpuFlag::C) { let address = cpu.fetch_word(); cpu.call(address); Executed(24) } else { cpu.registers.pc += 2; Executed(12) } }, //CALL Z,nn
    //0xDD
    0xDE => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::sbc, next_byte); Executed(8) }, //SBC A,n
    0xDF => { cpu.call(0x0018); Executed(16) }, //RST 18H
    0xE0 => { let address = 0xFF00 + cpu.fetch_byte() as u16; cpu.mmu.write_byte(address, cpu.registers.a); Executed(12) }, //LDH (n),
    0xE1 => { let hl = cpu.pop(); cpu.registers.set_hl(hl); Executed(12) } //POP HL
    0xE2 => { cpu.mmu.write_byte(0xFF00 + cpu.registers.c as u16, cpu.registers.a);  Executed(8) }, //LD (C),A
    //0xE3
    //0xE4
    0xE5 => { cpu.push(cpu.registers.get_hl()); Executed(16) } //PUSH HL
    0xE6 => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::and, next_byte); Executed(8) } //AND A,n
    0xE7 => { cpu.call(0x0020); Executed(16) }, //RST 20H
    0xE8 => {
      let value1 = cpu.registers.sp;
      let value2 = cpu.fetch_byte() as i8 as i16 as u16; //some rust magic that magic to add the u16 with wrapping add; i16 -> if i16 < 0 { u16 = u16.max - abs(i16) } and the sign is then done via the wrap around
      cpu.registers.sp = alu::add_next_signed_byte_to_word(&mut cpu.registers, value1, value2);
      Executed(16)
    }, //ADD SP,r8
    0xE9 => { cpu.registers.pc = cpu.registers.get_hl(); Executed(4) }, //JP (HL)
    0xEA => { let address = cpu.fetch_word(); cpu.mmu.write_byte(address, cpu.registers.a); Executed(16) }, //LD (nn),A
    //0xEB
    //0xEC
    //0xED
    0xEE => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::xor, next_byte); Executed(8) }, //XOR A,n
    0xEF => { cpu.call(0x0028); Executed(16) }, //RST 28H
    0xF0 => { let address = 0xFF00 + cpu.fetch_byte() as u16; cpu.registers.a = cpu.mmu.read_byte(address); Executed(12) }, //LDH A,(n)
    0xF1 => { let af = cpu.pop(); cpu.registers.set_af(af); Executed(12) } //POP AF
    0xF2 => { cpu.registers.a = cpu.mmu.read_byte(0xFF00 + cpu.registers.c as u16);  Executed(8) }, //LD A,(C)
    0xF3 => { cpu.ime = false; cpu.ei_requested = 0; Executed(4) }, //DI disable interrupts
    //0xF4
    0xF5 => { cpu.push(cpu.registers.get_af()); Executed(16) }, //PUSH AF
    0xF6 => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::or, next_byte); Executed(8) } //OR A,n
    0xF7 => { cpu.call(0x0030); Executed(16) }, //RST 30H
    0xF8 => {
      let value1 = cpu.registers.sp;
      let value2 = cpu.fetch_byte() as i8 as i16 as u16; //some rust magic that magic to add the u16 with wrapping add; i16 -> if i16 < 0 { u16 = u16.max - abs(i16) } and the sign is then done via the wrap around
      let result = alu::add_next_signed_byte_to_word(&mut cpu.registers, value1, value2);
      cpu.registers.set_hl(result);
      Executed(12)
    }, //LD HL, SP+r8
    0xF9 => { cpu.registers.sp = cpu.registers.get_hl(); Executed(8) }, //LD SP,HL
    0xFA => { let address = cpu.fetch_word(); cpu.registers.a = cpu.mmu.read_byte(address); Executed(16) }, //LD A,(nn)
    0xFB => { cpu.ei_requested = 2; Executed(4) }, //EI enable interrupts
    //0xFC
    //0xFD
    0xFE => { let next_byte = cpu.fetch_byte(); cpu.execute_binary_with_value(alu::cp, next_byte); Executed(8) }, //CP A,n  compare a-n
    0xFF => { cpu.call(0x0038); Executed(16) }, //RST 0x0038
    _ => { UnknownOpCode }
  }
}