use crate::cpu::alu;
use crate::cpu::OpCodeResultCb;
use crate::cpu::OpCodeResultCb::{Executed, UnknownOpCode};
use crate::cpu::Cpu;
use crate::cpu::registers::RegisterName8;

pub fn execute(op_code: u8, cpu: &mut Cpu) -> OpCodeResultCb {
  match op_code {
    0x00 => { alu::rlc(&mut &mut cpu.registers, RegisterName8::B ); Executed(8) }, //RLC B
    0x01 => { cpu.registers.c = alu::rlc(&mut cpu.registers, cpu.registers.c); Executed(8) }, //RLC C
    0x02 => { cpu.registers.d = alu::rlc(&mut cpu.registers, cpu.registers.d); Executed(8) }, //RLC D
    0x03 => { cpu.registers.e = alu::rlc(&mut cpu.registers, cpu.registers.e); Executed(8) }, //RLC E
    0x04 => { cpu.registers.h = alu::rlc(&mut cpu.registers, cpu.registers.h); Executed(8) }, //RLC H
    0x05 => { cpu.registers.l = alu::rlc(&mut cpu.registers, cpu.registers.l); Executed(8) }, //RLC L
    0x06 => { let new_value = alu::rlc(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RLC (HL)
    0x07 => { cpu.registers.a = alu::rlc(&mut cpu.registers, cpu.registers.a); Executed(8) }, //RLC A
    0x08 => { cpu.registers.b = alu::rrc(&mut cpu.registers, cpu.registers.b); Executed(8) }, //RRC B
    0x09 => { cpu.registers.c = alu::rrc(&mut cpu.registers, cpu.registers.c); Executed(8) }, //RRC C
    0x0A => { cpu.registers.d = alu::rrc(&mut cpu.registers, cpu.registers.d); Executed(8) }, //RRC D
    0x0B => { cpu.registers.e = alu::rrc(&mut cpu.registers, cpu.registers.e); Executed(8) }, //RRC E
    0x0C => { cpu.registers.h = alu::rrc(&mut cpu.registers, cpu.registers.h); Executed(8) }, //RRC H
    0x0D => { cpu.registers.l = alu::rrc(&mut cpu.registers, cpu.registers.l); Executed(8) }, //RRC L
    0x0E => { let new_value = alu::rrc(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RRC (HL)
    0x0F => { cpu.registers.a = alu::rrc(&mut cpu.registers, cpu.registers.a); Executed(8) }, //RRC A
    0x10 => { cpu.registers.b = alu::rl(&mut cpu.registers, cpu.registers.b); Executed(8) }, //RL B
    0x11 => { cpu.registers.c = alu::rl(&mut cpu.registers, cpu.registers.c); Executed(8) }, //RL C
    0x12 => { cpu.registers.d = alu::rl(&mut cpu.registers, cpu.registers.d); Executed(8) }, //RL D
    0x13 => { cpu.registers.e = alu::rl(&mut cpu.registers, cpu.registers.e); Executed(8) }, //RL E
    0x14 => { cpu.registers.h = alu::rl(&mut cpu.registers, cpu.registers.h); Executed(8) }, //RL H
    0x15 => { cpu.registers.l = alu::rl(&mut cpu.registers, cpu.registers.l); Executed(8) }, //RL L
    0x16 => { let new_value = alu::rl(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RL (HL)
    0x17 => { cpu.registers.a = alu::rl(&mut cpu.registers, cpu.registers.a); Executed(8) }, //RL A
    0x18 => { cpu.registers.b = alu::rr(&mut cpu.registers, cpu.registers.b); Executed(8) }, //RR B
    0x19 => { cpu.registers.c = alu::rr(&mut cpu.registers, cpu.registers.c); Executed(8) }, //RR C
    0x1A => { cpu.registers.d = alu::rr(&mut cpu.registers, cpu.registers.d); Executed(8) }, //RR D
    0x1B => { cpu.registers.e = alu::rr(&mut cpu.registers, cpu.registers.e); Executed(8) }, //RR E
    0x1C => { cpu.registers.h = alu::rr(&mut cpu.registers, cpu.registers.h); Executed(8) }, //RR H
    0x1D => { cpu.registers.l = alu::rr(&mut cpu.registers, cpu.registers.l); Executed(8) }, //RR L
    0x1E => { let new_value = alu::rr(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RR (HL)
    0x1F => { cpu.registers.a = alu::rr(&mut cpu.registers, cpu.registers.a); Executed(8) }, //RR A
    0x20 => { cpu.registers.b = alu::sla(&mut cpu.registers, cpu.registers.b); Executed(8) }, //SLA B
    0x21 => { cpu.registers.c = alu::sla(&mut cpu.registers, cpu.registers.c); Executed(8) }, //SLA C
    0x22 => { cpu.registers.d = alu::sla(&mut cpu.registers, cpu.registers.d); Executed(8) }, //SLA D
    0x23 => { cpu.registers.e = alu::sla(&mut cpu.registers, cpu.registers.e); Executed(8) }, //SLA E
    0x24 => { cpu.registers.h = alu::sla(&mut cpu.registers, cpu.registers.h); Executed(8) }, //SLA H
    0x25 => { cpu.registers.l = alu::sla(&mut cpu.registers, cpu.registers.l); Executed(8) }, //SLA L
    0x26 => { let new_value = alu::sla(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SLA (HL)
    0x27 => { cpu.registers.a = alu::sla(&mut cpu.registers, cpu.registers.a); Executed(8) }, //SLA A
    0x28 => { cpu.registers.b = alu::sra(&mut cpu.registers, cpu.registers.b); Executed(8) }, //SRA B
    0x29 => { cpu.registers.c = alu::sra(&mut cpu.registers, cpu.registers.c); Executed(8) }, //SRA C
    0x2A => { cpu.registers.d = alu::sra(&mut cpu.registers, cpu.registers.d); Executed(8) }, //SRA D
    0x2B => { cpu.registers.e = alu::sra(&mut cpu.registers, cpu.registers.e); Executed(8) }, //SRA E
    0x2C => { cpu.registers.h = alu::sra(&mut cpu.registers, cpu.registers.h); Executed(8) }, //SRA H
    0x2D => { cpu.registers.l = alu::sra(&mut cpu.registers, cpu.registers.l); Executed(8) }, //SRA L
    0x2E => { let new_value = alu::sra(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SRA (HL)
    0x2F => { cpu.registers.a = alu::sra(&mut cpu.registers, cpu.registers.a); Executed(8) }, //SRA A
    0x30 => { cpu.registers.b = alu::swap(&mut cpu.registers, cpu.registers.b); Executed(8) }, //SWAP B
    0x31 => { cpu.registers.c = alu::swap(&mut cpu.registers, cpu.registers.c); Executed(8) }, //SWAP C
    0x32 => { cpu.registers.d = alu::swap(&mut cpu.registers, cpu.registers.d); Executed(8) }, //SWAP D
    0x33 => { cpu.registers.e = alu::swap(&mut cpu.registers, cpu.registers.e); Executed(8) }, //SWAP E
    0x34 => { cpu.registers.h = alu::swap(&mut cpu.registers, cpu.registers.h); Executed(8) }, //SWAP H
    0x35 => { cpu.registers.l = alu::swap(&mut cpu.registers, cpu.registers.l); Executed(8) }, //SWAP L
    0x36 => { let new_value = alu::swap(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SWAP (HL)
    0x37 => { cpu.registers.a = alu::swap(&mut cpu.registers, cpu.registers.a); Executed(8) }, //SWAP A
    0x38 => { cpu.registers.b = alu::srl(&mut cpu.registers, cpu.registers.b); Executed(8) }, //SRL B
    0x39 => { cpu.registers.c = alu::srl(&mut cpu.registers, cpu.registers.c); Executed(8) }, //SRL C
    0x3A => { cpu.registers.d = alu::srl(&mut cpu.registers, cpu.registers.d); Executed(8) }, //SRL D
    0x3B => { cpu.registers.e = alu::srl(&mut cpu.registers, cpu.registers.e); Executed(8) }, //SRL E
    0x3C => { cpu.registers.h = alu::srl(&mut cpu.registers, cpu.registers.h); Executed(8) }, //SRL H
    0x3D => { cpu.registers.l = alu::srl(&mut cpu.registers, cpu.registers.l); Executed(8) }, //SRL L
    0x3E => { let new_value = alu::srl(&mut cpu.registers, cpu.mmu.read_byte(cpu.registers.get_hl())); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SRL (HL)
    0x3F => { cpu.registers.a = alu::srl(&mut cpu.registers, cpu.registers.a); Executed(8) }, //SRL A
    0x40 => { alu::bit(&mut cpu.registers, 0, cpu.registers.b); Executed(8) }, //BIT 0,B
    0x41 => { alu::bit(&mut cpu.registers, 0, cpu.registers.c); Executed(8) }, //BIT 0,C
    0x42 => { alu::bit(&mut cpu.registers, 0, cpu.registers.d); Executed(8) }, //BIT 0,D
    0x43 => { alu::bit(&mut cpu.registers, 0, cpu.registers.e); Executed(8) }, //BIT 0,E
    0x44 => { alu::bit(&mut cpu.registers, 0, cpu.registers.h); Executed(8) }, //BIT 0,H
    0x45 => { alu::bit(&mut cpu.registers, 0, cpu.registers.l); Executed(8) }, //BIT 0,L
    0x46 => { alu::bit(&mut cpu.registers, 0, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 0,(HL)
    0x47 => { alu::bit(&mut cpu.registers, 0, cpu.registers.a); Executed(8) }, //BIT 0,A
    0x48 => { alu::bit(&mut cpu.registers, 1, cpu.registers.b); Executed(8) }, //BIT 1,B
    0x49 => { alu::bit(&mut cpu.registers, 1, cpu.registers.c); Executed(8) }, //BIT 1,C
    0x4A => { alu::bit(&mut cpu.registers, 1, cpu.registers.d); Executed(8) }, //BIT 1,D
    0x4B => { alu::bit(&mut cpu.registers, 1, cpu.registers.e); Executed(8) }, //BIT 1,E
    0x4C => { alu::bit(&mut cpu.registers, 1, cpu.registers.h); Executed(8) }, //BIT 1,H
    0x4D => { alu::bit(&mut cpu.registers, 1, cpu.registers.l); Executed(8) }, //BIT 1,L
    0x4E => { alu::bit(&mut cpu.registers, 1, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 1,(HL)
    0x4F => { alu::bit(&mut cpu.registers, 1, cpu.registers.a); Executed(8) }, //BIT 1,A
    0x50 => { alu::bit(&mut cpu.registers, 2, cpu.registers.b); Executed(8) }, //BIT 2,B
    0x51 => { alu::bit(&mut cpu.registers, 2, cpu.registers.c); Executed(8) }, //BIT 2,C
    0x52 => { alu::bit(&mut cpu.registers, 2, cpu.registers.d); Executed(8) }, //BIT 2,D
    0x53 => { alu::bit(&mut cpu.registers, 2, cpu.registers.e); Executed(8) }, //BIT 2,E
    0x54 => { alu::bit(&mut cpu.registers, 2, cpu.registers.h); Executed(8) }, //BIT 2,H
    0x55 => { alu::bit(&mut cpu.registers, 2, cpu.registers.l); Executed(8) }, //BIT 2,L
    0x56 => { alu::bit(&mut cpu.registers, 2, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 2,(HL)
    0x57 => { alu::bit(&mut cpu.registers, 2, cpu.registers.a); Executed(8) }, //BIT 2,A
    0x58 => { alu::bit(&mut cpu.registers, 3, cpu.registers.b); Executed(8) }, //BIT 3,B
    0x59 => { alu::bit(&mut cpu.registers, 3, cpu.registers.c); Executed(8) }, //BIT 3,C
    0x5A => { alu::bit(&mut cpu.registers, 3, cpu.registers.d); Executed(8) }, //BIT 3,D
    0x5B => { alu::bit(&mut cpu.registers, 3, cpu.registers.e); Executed(8) }, //BIT 3,E
    0x5C => { alu::bit(&mut cpu.registers, 3, cpu.registers.h); Executed(8) }, //BIT 3,H
    0x5D => { alu::bit(&mut cpu.registers, 3, cpu.registers.l); Executed(8) }, //BIT 3,L
    0x5E => { alu::bit(&mut cpu.registers, 3, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 3,(HL)
    0x5F => { alu::bit(&mut cpu.registers, 3, cpu.registers.a); Executed(8) }, //BIT 3,A
    0x60 => { alu::bit(&mut cpu.registers, 4, cpu.registers.b); Executed(8) }, //BIT 4,B
    0x61 => { alu::bit(&mut cpu.registers, 4, cpu.registers.c); Executed(8) }, //BIT 4,C
    0x62 => { alu::bit(&mut cpu.registers, 4, cpu.registers.d); Executed(8) }, //BIT 4,D
    0x63 => { alu::bit(&mut cpu.registers, 4, cpu.registers.e); Executed(8) }, //BIT 4,E
    0x64 => { alu::bit(&mut cpu.registers, 4, cpu.registers.h); Executed(8) }, //BIT 4,H
    0x65 => { alu::bit(&mut cpu.registers, 4, cpu.registers.l); Executed(8) }, //BIT 4,L
    0x66 => { alu::bit(&mut cpu.registers, 4, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 4,(HL)
    0x67 => { alu::bit(&mut cpu.registers, 4, cpu.registers.a); Executed(8) }, //BIT 4,A
    0x68 => { alu::bit(&mut cpu.registers, 5, cpu.registers.b); Executed(8) }, //BIT 5,B
    0x69 => { alu::bit(&mut cpu.registers, 5, cpu.registers.c); Executed(8) }, //BIT 5,C
    0x6A => { alu::bit(&mut cpu.registers, 5, cpu.registers.d); Executed(8) }, //BIT 5,D
    0x6B => { alu::bit(&mut cpu.registers, 5, cpu.registers.e); Executed(8) }, //BIT 5,E
    0x6C => { alu::bit(&mut cpu.registers, 5, cpu.registers.h); Executed(8) }, //BIT 5,H
    0x6D => { alu::bit(&mut cpu.registers, 5, cpu.registers.l); Executed(8) }, //BIT 5,L
    0x6E => { alu::bit(&mut cpu.registers, 5, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 5,(HL)
    0x6F => { alu::bit(&mut cpu.registers, 5, cpu.registers.a); Executed(8) }, //BIT 5,A
    0x70 => { alu::bit(&mut cpu.registers, 6, cpu.registers.b); Executed(8) }, //BIT 6,B
    0x71 => { alu::bit(&mut cpu.registers, 6, cpu.registers.c); Executed(8) }, //BIT 6,C
    0x72 => { alu::bit(&mut cpu.registers, 6, cpu.registers.d); Executed(8) }, //BIT 6,D
    0x73 => { alu::bit(&mut cpu.registers, 6, cpu.registers.e); Executed(8) }, //BIT 6,E
    0x74 => { alu::bit(&mut cpu.registers, 6, cpu.registers.h); Executed(8) }, //BIT 6,H
    0x75 => { alu::bit(&mut cpu.registers, 6, cpu.registers.l); Executed(8) }, //BIT 6,L
    0x76 => { alu::bit(&mut cpu.registers, 6, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 6,(HL)
    0x77 => { alu::bit(&mut cpu.registers, 6, cpu.registers.a); Executed(8) }, //BIT 6,A
    0x78 => { alu::bit(&mut cpu.registers, 7, cpu.registers.b); Executed(8) }, //BIT 7,B
    0x79 => { alu::bit(&mut cpu.registers, 7, cpu.registers.c); Executed(8) }, //BIT 7,C
    0x7A => { alu::bit(&mut cpu.registers, 7, cpu.registers.d); Executed(8) }, //BIT 7,D
    0x7B => { alu::bit(&mut cpu.registers, 7, cpu.registers.e); Executed(8) }, //BIT 7,E
    0x7C => { alu::bit(&mut cpu.registers, 7, cpu.registers.h); Executed(8) }, //BIT 7,H
    0x7D => { alu::bit(&mut cpu.registers, 7, cpu.registers.l); Executed(8) }, //BIT 7,L
    0x7E => { alu::bit(&mut cpu.registers, 7, cpu.mmu.read_byte(cpu.registers.get_hl())); Executed(8) }, //BIT 7,(HL)
    0x7F => { alu::bit(&mut cpu.registers, 7, cpu.registers.a); Executed(8) }, //BIT 7,A
    0x80 => { cpu.registers.b = cpu.registers.b & !(1 << 0); Executed(8) }, //RES 0,B
    0x81 => { cpu.registers.c = cpu.registers.c & !(1 << 0); Executed(8) }, //RES 0,C
    0x82 => { cpu.registers.d = cpu.registers.d & !(1 << 0); Executed(8) }, //RES 0,D
    0x83 => { cpu.registers.e = cpu.registers.e & !(1 << 0); Executed(8) }, //RES 0,E
    0x84 => { cpu.registers.h = cpu.registers.h & !(1 << 0); Executed(8) }, //RES 0,H
    0x85 => { cpu.registers.l = cpu.registers.l & !(1 << 0); Executed(8) }, //RES 0,L
    0x86 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 0); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 0,(HL)
    0x87 => { cpu.registers.a = cpu.registers.a & !(1 << 0); Executed(8) }, //RES 0,A
    0x88 => { cpu.registers.b = cpu.registers.b & !(1 << 1); Executed(8) }, //RES 1,B
    0x89 => { cpu.registers.c = cpu.registers.c & !(1 << 1); Executed(8) }, //RES 1,C
    0x8A => { cpu.registers.d = cpu.registers.d & !(1 << 1); Executed(8) }, //RES 1,D
    0x8B => { cpu.registers.e = cpu.registers.e & !(1 << 1); Executed(8) }, //RES 1,E
    0x8C => { cpu.registers.h = cpu.registers.h & !(1 << 1); Executed(8) }, //RES 1,H
    0x8D => { cpu.registers.l = cpu.registers.l & !(1 << 1); Executed(8) }, //RES 1,L
    0x8E => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 1); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 1,(HL)
    0x8F => { cpu.registers.a = cpu.registers.a & !(1 << 1); Executed(8) }, //RES 1,A
    0x90 => { cpu.registers.b = cpu.registers.b & !(1 << 2); Executed(8) }, //RES 2,B
    0x91 => { cpu.registers.c = cpu.registers.c & !(1 << 2); Executed(8) }, //RES 2,C
    0x92 => { cpu.registers.d = cpu.registers.d & !(1 << 2); Executed(8) }, //RES 2,D
    0x93 => { cpu.registers.e = cpu.registers.e & !(1 << 2); Executed(8) }, //RES 2,E
    0x94 => { cpu.registers.h = cpu.registers.h & !(1 << 2); Executed(8) }, //RES 2,H
    0x95 => { cpu.registers.l = cpu.registers.l & !(1 << 2); Executed(8) }, //RES 2,L
    0x96 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 2); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 2,(HL)
    0x97 => { cpu.registers.a = cpu.registers.a & !(1 << 2); Executed(8) }, //RES 2,A
    0x98 => { cpu.registers.b = cpu.registers.b & !(1 << 3); Executed(8) }, //RES 3,B
    0x99 => { cpu.registers.c = cpu.registers.c & !(1 << 3); Executed(8) }, //RES 3,C
    0x9A => { cpu.registers.d = cpu.registers.d & !(1 << 3); Executed(8) }, //RES 3,D
    0x9B => { cpu.registers.e = cpu.registers.e & !(1 << 3); Executed(8) }, //RES 3,E
    0x9C => { cpu.registers.h = cpu.registers.h & !(1 << 3); Executed(8) }, //RES 3,H
    0x9D => { cpu.registers.l = cpu.registers.l & !(1 << 3); Executed(8) }, //RES 3,L
    0x9E => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 3); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 3,(HL)
    0x9F => { cpu.registers.a = cpu.registers.a & !(1 << 3); Executed(8) }, //RES 3,A
    0xA0 => { cpu.registers.b = cpu.registers.b & !(1 << 4); Executed(8) }, //RES 4,B
    0xA1 => { cpu.registers.c = cpu.registers.c & !(1 << 4); Executed(8) }, //RES 4,C
    0xA2 => { cpu.registers.d = cpu.registers.d & !(1 << 4); Executed(8) }, //RES 4,D
    0xA3 => { cpu.registers.e = cpu.registers.e & !(1 << 4); Executed(8) }, //RES 4,E
    0xA4 => { cpu.registers.h = cpu.registers.h & !(1 << 4); Executed(8) }, //RES 4,H
    0xA5 => { cpu.registers.l = cpu.registers.l & !(1 << 4); Executed(8) }, //RES 4,L
    0xA6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 4); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 4,(HL)
    0xA7 => { cpu.registers.a = cpu.registers.a & !(1 << 4); Executed(8) }, //RES 4,A
    0xA8 => { cpu.registers.b = cpu.registers.b & !(1 << 5); Executed(8) }, //RES 5,B
    0xA9 => { cpu.registers.c = cpu.registers.c & !(1 << 5); Executed(8) }, //RES 5,C
    0xAA => { cpu.registers.d = cpu.registers.d & !(1 << 5); Executed(8) }, //RES 5,D
    0xAB => { cpu.registers.e = cpu.registers.e & !(1 << 5); Executed(8) }, //RES 5,E
    0xAC => { cpu.registers.h = cpu.registers.h & !(1 << 5); Executed(8) }, //RES 5,H
    0xAD => { cpu.registers.l = cpu.registers.l & !(1 << 5); Executed(8) }, //RES 5,L
    0xAE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 5); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 5,(HL)
    0xAF => { cpu.registers.a = cpu.registers.a & !(1 << 5); Executed(8) }, //RES 5,A
    0xB0 => { cpu.registers.b = cpu.registers.b & !(1 << 6); Executed(8) }, //RES 6,B
    0xB1 => { cpu.registers.c = cpu.registers.c & !(1 << 6); Executed(8) }, //RES 6,C
    0xB2 => { cpu.registers.d = cpu.registers.d & !(1 << 6); Executed(8) }, //RES 6,D
    0xB3 => { cpu.registers.e = cpu.registers.e & !(1 << 6); Executed(8) }, //RES 6,E
    0xB4 => { cpu.registers.h = cpu.registers.h & !(1 << 6); Executed(8) }, //RES 6,H
    0xB5 => { cpu.registers.l = cpu.registers.l & !(1 << 6); Executed(8) }, //RES 6,L
    0xB6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 6); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 6,(HL)
    0xB7 => { cpu.registers.a = cpu.registers.a & !(1 << 6); Executed(8) }, //RES 6,A
    0xB8 => { cpu.registers.b = cpu.registers.b & !(1 << 7); Executed(8) }, //RES 7,B
    0xB9 => { cpu.registers.c = cpu.registers.c & !(1 << 7); Executed(8) }, //RES 7,C
    0xBA => { cpu.registers.d = cpu.registers.d & !(1 << 7); Executed(8) }, //RES 7,D
    0xBB => { cpu.registers.e = cpu.registers.e & !(1 << 7); Executed(8) }, //RES 7,E
    0xBC => { cpu.registers.h = cpu.registers.h & !(1 << 7); Executed(8) }, //RES 7,H
    0xBD => { cpu.registers.l = cpu.registers.l & !(1 << 7); Executed(8) }, //RES 7,L
    0xBE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !(1 << 7); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 7,(HL)
    0xBF => { cpu.registers.a = cpu.registers.a & !(1 << 7); Executed(8) }, //RES 7,A
    0xC0 => { cpu.registers.b = cpu.registers.b | (1 << 0); Executed(8) }, //SET 0,B
    0xC1 => { cpu.registers.c = cpu.registers.c | (1 << 0); Executed(8) }, //SET 0,C
    0xC2 => { cpu.registers.d = cpu.registers.d | (1 << 0); Executed(8) }, //SET 0,D
    0xC3 => { cpu.registers.e = cpu.registers.e | (1 << 0); Executed(8) }, //SET 0,E
    0xC4 => { cpu.registers.h = cpu.registers.h | (1 << 0); Executed(8) }, //SET 0,H
    0xC5 => { cpu.registers.l = cpu.registers.l | (1 << 0); Executed(8) }, //SET 0,L
    0xC6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 0); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 0,(HL)
    0xC7 => { cpu.registers.a = cpu.registers.a | (1 << 0); Executed(8) }, //SET 0,A
    0xC8 => { cpu.registers.b = cpu.registers.b | (1 << 1); Executed(8) }, //SET 1,B
    0xC9 => { cpu.registers.c = cpu.registers.c | (1 << 1); Executed(8) }, //SET 1,C
    0xCA => { cpu.registers.d = cpu.registers.d | (1 << 1); Executed(8) }, //SET 1,D
    0xCB => { cpu.registers.e = cpu.registers.e | (1 << 1); Executed(8) }, //SET 1,E
    0xCC => { cpu.registers.h = cpu.registers.h | (1 << 1); Executed(8) }, //SET 1,H
    0xCD => { cpu.registers.l = cpu.registers.l | (1 << 1); Executed(8) }, //SET 1,L
    0xCE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 1); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 1,(HL)
    0xCF => { cpu.registers.a = cpu.registers.a | (1 << 1); Executed(8) }, //SET 1,A
    0xD0 => { cpu.registers.b = cpu.registers.b | (1 << 2); Executed(8) }, //SET 2,B
    0xD1 => { cpu.registers.c = cpu.registers.c | (1 << 2); Executed(8) }, //SET 2,C
    0xD2 => { cpu.registers.d = cpu.registers.d | (1 << 2); Executed(8) }, //SET 2,D
    0xD3 => { cpu.registers.e = cpu.registers.e | (1 << 2); Executed(8) }, //SET 2,E
    0xD4 => { cpu.registers.h = cpu.registers.h | (1 << 2); Executed(8) }, //SET 2,H
    0xD5 => { cpu.registers.l = cpu.registers.l | (1 << 2); Executed(8) }, //SET 2,L
    0xD6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 2); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 2,(HL)
    0xD7 => { cpu.registers.a = cpu.registers.a | (1 << 2); Executed(8) }, //SET 2,A
    0xD8 => { cpu.registers.b = cpu.registers.b | (1 << 3); Executed(8) }, //SET 3,B
    0xD9 => { cpu.registers.c = cpu.registers.c | (1 << 3); Executed(8) }, //SET 3,C
    0xDA => { cpu.registers.d = cpu.registers.d | (1 << 3); Executed(8) }, //SET 3,D
    0xDB => { cpu.registers.e = cpu.registers.e | (1 << 3); Executed(8) }, //SET 3,E
    0xDC => { cpu.registers.h = cpu.registers.h | (1 << 3); Executed(8) }, //SET 3,H
    0xDD => { cpu.registers.l = cpu.registers.l | (1 << 3); Executed(8) }, //SET 3,L
    0xDE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 3); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 3,(HL)
    0xDF => { cpu.registers.a = cpu.registers.a | (1 << 3); Executed(8) }, //SET 3,A
    0xE0 => { cpu.registers.b = cpu.registers.b | (1 << 4); Executed(8) }, //SET 4,B
    0xE1 => { cpu.registers.c = cpu.registers.c | (1 << 4); Executed(8) }, //SET 4,C
    0xE2 => { cpu.registers.d = cpu.registers.d | (1 << 4); Executed(8) }, //SET 4,D
    0xE3 => { cpu.registers.e = cpu.registers.e | (1 << 4); Executed(8) }, //SET 4,E
    0xE4 => { cpu.registers.h = cpu.registers.h | (1 << 4); Executed(8) }, //SET 4,H
    0xE5 => { cpu.registers.l = cpu.registers.l | (1 << 4); Executed(8) }, //SET 4,L
    0xE6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 4); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 4,(HL)
    0xE7 => { cpu.registers.a = cpu.registers.a | (1 << 4); Executed(8) }, //SET 4,A
    0xE8 => { cpu.registers.b = cpu.registers.b | (1 << 5); Executed(8) }, //SET 5,B
    0xE9 => { cpu.registers.c = cpu.registers.c | (1 << 5); Executed(8) }, //SET 5,C
    0xEA => { cpu.registers.d = cpu.registers.d | (1 << 5); Executed(8) }, //SET 5,D
    0xEB => { cpu.registers.e = cpu.registers.e | (1 << 5); Executed(8) }, //SET 5,E
    0xEC => { cpu.registers.h = cpu.registers.h | (1 << 5); Executed(8) }, //SET 5,H
    0xED => { cpu.registers.l = cpu.registers.l | (1 << 5); Executed(8) }, //SET 5,L
    0xEE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 5); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 5,(HL)
    0xEF => { cpu.registers.a = cpu.registers.a | (1 << 5); Executed(8) }, //SET 5,A
    0xF0 => { cpu.registers.b = cpu.registers.b | (1 << 6); Executed(8) }, //SET 6,B
    0xF1 => { cpu.registers.c = cpu.registers.c | (1 << 6); Executed(8) }, //SET 6,C
    0xF2 => { cpu.registers.d = cpu.registers.d | (1 << 6); Executed(8) }, //SET 6,D
    0xF3 => { cpu.registers.e = cpu.registers.e | (1 << 6); Executed(8) }, //SET 6,E
    0xF4 => { cpu.registers.h = cpu.registers.h | (1 << 6); Executed(8) }, //SET 6,H
    0xF5 => { cpu.registers.l = cpu.registers.l | (1 << 6); Executed(8) }, //SET 6,L
    0xF6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 6); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 6,(HL)
    0xF7 => { cpu.registers.a = cpu.registers.a | (1 << 6); Executed(8) }, //SET 6,A
    0xF8 => { cpu.registers.b = cpu.registers.b | (1 << 7); Executed(8) }, //SET 7,B
    0xF9 => { cpu.registers.c = cpu.registers.c | (1 << 7); Executed(8) }, //SET 7,C
    0xFA => { cpu.registers.d = cpu.registers.d | (1 << 7); Executed(8) }, //SET 7,D
    0xFB => { cpu.registers.e = cpu.registers.e | (1 << 7); Executed(8) }, //SET 7,E
    0xFC => { cpu.registers.h = cpu.registers.h | (1 << 7); Executed(8) }, //SET 7,H
    0xFD => { cpu.registers.l = cpu.registers.l | (1 << 7); Executed(8) }, //SET 7,L
    0xFE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | (1 << 7); cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 7,(HL)
    0xFF => { cpu.registers.a = cpu.registers.a | (1 << 7); Executed(8) }, //SET 7,A
    _ => UnknownOpCode
  }
}