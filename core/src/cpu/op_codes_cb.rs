use crate::cpu::alu;
use crate::cpu::OpCodeResult;
use crate::cpu::OpCodeResult::Executed;
use crate::cpu::Cpu;
use crate::cpu::registers::RegisterName8;

#[allow(unreachable_patterns)]
pub fn execute(op_code: u8, cpu: &mut Cpu) -> OpCodeResult {
  match op_code {
    0x00 => { cpu.execute(alu::rlc, RegisterName8::B); Executed(8) }, //RLC B
    0x01 => { cpu.execute(alu::rlc, RegisterName8::C); Executed(8) }, //RLC C
    0x02 => { cpu.execute(alu::rlc, RegisterName8::D); Executed(8) }, //RLC D
    0x03 => { cpu.execute(alu::rlc, RegisterName8::E); Executed(8) }, //RLC E
    0x04 => { cpu.execute(alu::rlc, RegisterName8::H); Executed(8) }, //RLC H
    0x05 => { cpu.execute(alu::rlc, RegisterName8::L); Executed(8) }, //RLC L
    0x06 => { cpu.execute_hl(alu::rlc); Executed(8) }, //RLC (HL)
    0x07 => { cpu.execute(alu::rlc, RegisterName8::A); Executed(8) }, //RLC A
    0x08 => { cpu.execute(alu::rrc, RegisterName8::B); Executed(8) }, //RRC B
    0x09 => { cpu.execute(alu::rrc, RegisterName8::C); Executed(8) }, //RRC C
    0x0A => { cpu.execute(alu::rrc, RegisterName8::D); Executed(8) }, //RRC D
    0x0B => { cpu.execute(alu::rrc, RegisterName8::E); Executed(8) }, //RRC E
    0x0C => { cpu.execute(alu::rrc, RegisterName8::H); Executed(8) }, //RRC H
    0x0D => { cpu.execute(alu::rrc, RegisterName8::L); Executed(8) }, //RRC L
    0x0E => { cpu.execute_hl(alu::rrc); Executed(8) }, //RRC (HL)
    0x0F => { cpu.execute(alu::rrc, RegisterName8::A); Executed(8) }, //RRC A
    0x10 => { cpu.execute(alu::rl, RegisterName8::B); Executed(8) }, //RL B
    0x11 => { cpu.execute(alu::rl, RegisterName8::C); Executed(8) }, //RL C
    0x12 => { cpu.execute(alu::rl, RegisterName8::D); Executed(8) }, //RL D
    0x13 => { cpu.execute(alu::rl, RegisterName8::E); Executed(8) }, //RL E
    0x14 => { cpu.execute(alu::rl, RegisterName8::H); Executed(8) }, //RL H
    0x15 => { cpu.execute(alu::rl, RegisterName8::L); Executed(8) }, //RL L
    0x16 => { cpu.execute_hl(alu::rl); Executed(8) }, //RL (HL)
    0x17 => { cpu.execute(alu::rl, RegisterName8::A); Executed(8) }, //RL A
    0x18 => { cpu.execute(alu::rr, RegisterName8::B); Executed(8) }, //RR B
    0x19 => { cpu.execute(alu::rr, RegisterName8::C); Executed(8) }, //RR C
    0x1A => { cpu.execute(alu::rr, RegisterName8::D); Executed(8) }, //RR D
    0x1B => { cpu.execute(alu::rr, RegisterName8::E); Executed(8) }, //RR E
    0x1C => { cpu.execute(alu::rr, RegisterName8::H); Executed(8) }, //RR H
    0x1D => { cpu.execute(alu::rr, RegisterName8::L); Executed(8) }, //RR L
    0x1E => { cpu.execute_hl(alu::rr); Executed(8) }, //RR (HL)
    0x1F => { cpu.execute(alu::rr, RegisterName8::A); Executed(8) }, //RR A
    0x20 => { cpu.execute(alu::sla, RegisterName8::B); Executed(8) }, //SLA B
    0x21 => { cpu.execute(alu::sla, RegisterName8::C); Executed(8) }, //SLA C
    0x22 => { cpu.execute(alu::sla, RegisterName8::D); Executed(8) }, //SLA D
    0x23 => { cpu.execute(alu::sla, RegisterName8::E); Executed(8) }, //SLA E
    0x24 => { cpu.execute(alu::sla, RegisterName8::H); Executed(8) }, //SLA H
    0x25 => { cpu.execute(alu::sla, RegisterName8::L); Executed(8) }, //SLA L
    0x26 => { cpu.execute_hl(alu::sla); Executed(8) }, //SLA (HL)
    0x27 => { cpu.execute(alu::sla, RegisterName8::A); Executed(8) }, //SLA A
    0x28 => { cpu.execute(alu::sra, RegisterName8::B); Executed(8) }, //SRA B
    0x29 => { cpu.execute(alu::sra, RegisterName8::C); Executed(8) }, //SRA C
    0x2A => { cpu.execute(alu::sra, RegisterName8::D); Executed(8) }, //SRA D
    0x2B => { cpu.execute(alu::sra, RegisterName8::E); Executed(8) }, //SRA E
    0x2C => { cpu.execute(alu::sra, RegisterName8::H); Executed(8) }, //SRA H
    0x2D => { cpu.execute(alu::sra, RegisterName8::L); Executed(8) }, //SRA L
    0x2E => { cpu.execute_hl(alu::sra); Executed(8) }, //SRA (HL)
    0x2F => { cpu.execute(alu::sra, RegisterName8::A); Executed(8) }, //SRA A
    0x30 => { cpu.execute(alu::swap, RegisterName8::B); Executed(8) }, //SWAP B
    0x31 => { cpu.execute(alu::swap, RegisterName8::C); Executed(8) }, //SWAP C
    0x32 => { cpu.execute(alu::swap, RegisterName8::D); Executed(8) }, //SWAP D
    0x33 => { cpu.execute(alu::swap, RegisterName8::E); Executed(8) }, //SWAP E
    0x34 => { cpu.execute(alu::swap, RegisterName8::H); Executed(8) }, //SWAP H
    0x35 => { cpu.execute(alu::swap, RegisterName8::L); Executed(8) }, //SWAP L
    0x36 => { cpu.execute_hl(alu::swap); Executed(8) }, //SWAP (HL)
    0x37 => { cpu.execute(alu::swap, RegisterName8::A); Executed(8) }, //SWAP A
    0x38 => { cpu.execute(alu::srl, RegisterName8::B); Executed(8) }, //SRL B
    0x39 => { cpu.execute(alu::srl, RegisterName8::C); Executed(8) }, //SRL C
    0x3A => { cpu.execute(alu::srl, RegisterName8::D); Executed(8) }, //SRL D
    0x3B => { cpu.execute(alu::srl, RegisterName8::E); Executed(8) }, //SRL E
    0x3C => { cpu.execute(alu::srl, RegisterName8::H); Executed(8) }, //SRL H
    0x3D => { cpu.execute(alu::srl, RegisterName8::L); Executed(8) }, //SRL L
    0x3E => { cpu.execute_hl(alu::srl); Executed(8) }, //SRL (HL)
    0x3F => { cpu.execute(alu::srl, RegisterName8::A); Executed(8) }, //SRL A
    0x40 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,B
    0x41 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,C
    0x42 => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,D
    0x43 => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,E
    0x44 => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,H
    0x45 => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,L
    0x46 => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,(HL)
    0x47 => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 0, value); Executed(8) }, //BIT 0,A
    0x48 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,B
    0x49 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,C
    0x4A => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,D
    0x4B => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,E
    0x4C => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,H
    0x4D => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,L
    0x4E => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,(HL)
    0x4F => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 1, value); Executed(8) }, //BIT 1,A
    0x50 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,B
    0x51 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,C
    0x52 => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,D
    0x53 => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,E
    0x54 => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,H
    0x55 => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,L
    0x56 => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,(HL)
    0x57 => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 2, value); Executed(8) }, //BIT 2,A
    0x58 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,B
    0x59 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,C
    0x5A => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,D
    0x5B => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,E
    0x5C => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,H
    0x5D => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,L
    0x5E => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,(HL)
    0x5F => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 3, value); Executed(8) }, //BIT 3,A
    0x60 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,B
    0x61 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,C
    0x62 => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,D
    0x63 => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,E
    0x64 => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,H
    0x65 => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,L
    0x66 => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,(HL)
    0x67 => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 4, value); Executed(8) }, //BIT 4,A
    0x68 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,B
    0x69 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,C
    0x6A => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,D
    0x6B => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,E
    0x6C => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,H
    0x6D => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,L
    0x6E => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,(HL)
    0x6F => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 5, value); Executed(8) }, //BIT 5,A
    0x70 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,B
    0x71 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,C
    0x72 => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,D
    0x73 => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,E
    0x74 => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,H
    0x75 => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,L
    0x76 => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,(HL)
    0x77 => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 6, value); Executed(8) }, //BIT 6,A
    0x78 => { let value = cpu.registers.b; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,B
    0x79 => { let value = cpu.registers.c; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,C
    0x7A => { let value = cpu.registers.d; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,D
    0x7B => { let value = cpu.registers.e; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,E
    0x7C => { let value = cpu.registers.h; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,H
    0x7D => { let value = cpu.registers.l; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,L
    0x7E => { let value = cpu.mmu.read_byte(cpu.registers.get_hl()); alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,(HL)
    0x7F => { let value = cpu.registers.a; alu::bit(&mut cpu.registers, 7, value); Executed(8) }, //BIT 7,A
    0x80 => { cpu.registers.b &= !0b0000_0001; Executed(8) }, //RES 0,B
    0x81 => { cpu.registers.c &= !0b0000_0001; Executed(8) }, //RES 0,C
    0x82 => { cpu.registers.d &= !0b0000_0001; Executed(8) }, //RES 0,D
    0x83 => { cpu.registers.e &= !0b0000_0001; Executed(8) }, //RES 0,E
    0x84 => { cpu.registers.h &= !0b0000_0001; Executed(8) }, //RES 0,H
    0x85 => { cpu.registers.l &= !0b0000_0001; Executed(8) }, //RES 0,L
    0x86 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0000_0001; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 0,(HL)
    0x87 => { cpu.registers.a &= !0b0000_0001; Executed(8) }, //RES 0,A
    0x88 => { cpu.registers.b &= !0b0000_0010; Executed(8) }, //RES 1,B
    0x89 => { cpu.registers.c &= !0b0000_0010; Executed(8) }, //RES 1,C
    0x8A => { cpu.registers.d &= !0b0000_0010; Executed(8) }, //RES 1,D
    0x8B => { cpu.registers.e &= !0b0000_0010; Executed(8) }, //RES 1,E
    0x8C => { cpu.registers.h &= !0b0000_0010; Executed(8) }, //RES 1,H
    0x8D => { cpu.registers.l &= !0b0000_0010; Executed(8) }, //RES 1,L
    0x8E => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0000_0010; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 1,(HL)
    0x8F => { cpu.registers.a &= !0b0000_0010; Executed(8) }, //RES 1,A
    0x90 => { cpu.registers.b &= !0b0000_0100; Executed(8) }, //RES 2,B
    0x91 => { cpu.registers.c &= !0b0000_0100; Executed(8) }, //RES 2,C
    0x92 => { cpu.registers.d &= !0b0000_0100; Executed(8) }, //RES 2,D
    0x93 => { cpu.registers.e &= !0b0000_0100; Executed(8) }, //RES 2,E
    0x94 => { cpu.registers.h &= !0b0000_0100; Executed(8) }, //RES 2,H
    0x95 => { cpu.registers.l &= !0b0000_0100; Executed(8) }, //RES 2,L
    0x96 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0000_0100; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 2,(HL)
    0x97 => { cpu.registers.a &= !0b0000_0100; Executed(8) }, //RES 2,A
    0x98 => { cpu.registers.b &= !0b0000_1000; Executed(8) }, //RES 3,B
    0x99 => { cpu.registers.c &= !0b0000_1000; Executed(8) }, //RES 3,C
    0x9A => { cpu.registers.d &= !0b0000_1000; Executed(8) }, //RES 3,D
    0x9B => { cpu.registers.e &= !0b0000_1000; Executed(8) }, //RES 3,E
    0x9C => { cpu.registers.h &= !0b0000_1000; Executed(8) }, //RES 3,H
    0x9D => { cpu.registers.l &= !0b0000_1000; Executed(8) }, //RES 3,L
    0x9E => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0000_1000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 3,(HL)
    0x9F => { cpu.registers.a &= !0b0000_1000; Executed(8) }, //RES 3,A
    0xA0 => { cpu.registers.b &= !0b0001_0000; Executed(8) }, //RES 4,B
    0xA1 => { cpu.registers.c &= !0b0001_0000; Executed(8) }, //RES 4,C
    0xA2 => { cpu.registers.d &= !0b0001_0000; Executed(8) }, //RES 4,D
    0xA3 => { cpu.registers.e &= !0b0001_0000; Executed(8) }, //RES 4,E
    0xA4 => { cpu.registers.h &= !0b0001_0000; Executed(8) }, //RES 4,H
    0xA5 => { cpu.registers.l &= !0b0001_0000; Executed(8) }, //RES 4,L
    0xA6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0001_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 4,(HL)
    0xA7 => { cpu.registers.a &= !0b0001_0000; Executed(8) }, //RES 4,A
    0xA8 => { cpu.registers.b &= !0b0010_0000; Executed(8) }, //RES 5,B
    0xA9 => { cpu.registers.c &= !0b0010_0000; Executed(8) }, //RES 5,C
    0xAA => { cpu.registers.d &= !0b0010_0000; Executed(8) }, //RES 5,D
    0xAB => { cpu.registers.e &= !0b0010_0000; Executed(8) }, //RES 5,E
    0xAC => { cpu.registers.h &= !0b0010_0000; Executed(8) }, //RES 5,H
    0xAD => { cpu.registers.l &= !0b0010_0000; Executed(8) }, //RES 5,L
    0xAE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0010_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 5,(HL)
    0xAF => { cpu.registers.a &= !0b0010_0000; Executed(8) }, //RES 5,A
    0xB0 => { cpu.registers.b &= !0b0100_0000; Executed(8) }, //RES 6,B
    0xB1 => { cpu.registers.c &= !0b0100_0000; Executed(8) }, //RES 6,C
    0xB2 => { cpu.registers.d &= !0b0100_0000; Executed(8) }, //RES 6,D
    0xB3 => { cpu.registers.e &= !0b0100_0000; Executed(8) }, //RES 6,E
    0xB4 => { cpu.registers.h &= !0b0100_0000; Executed(8) }, //RES 6,H
    0xB5 => { cpu.registers.l &= !0b0100_0000; Executed(8) }, //RES 6,L
    0xB6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b0100_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 6,(HL)
    0xB7 => { cpu.registers.a &= !0b0100_0000; Executed(8) }, //RES 6,A
    0xB8 => { cpu.registers.b &= !0b1000_0000; Executed(8) }, //RES 7,B
    0xB9 => { cpu.registers.c &= !0b1000_0000; Executed(8) }, //RES 7,C
    0xBA => { cpu.registers.d &= !0b1000_0000; Executed(8) }, //RES 7,D
    0xBB => { cpu.registers.e &= !0b1000_0000; Executed(8) }, //RES 7,E
    0xBC => { cpu.registers.h &= !0b1000_0000; Executed(8) }, //RES 7,H
    0xBD => { cpu.registers.l &= !0b1000_0000; Executed(8) }, //RES 7,L
    0xBE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  & !0b1000_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //RES 7,(HL)
    0xBF => { cpu.registers.a &= !0b1000_0000; Executed(8) }, //RES 7,A
    0xC0 => { cpu.registers.b |= 0b0000_0001; Executed(8) }, //SET 0,B
    0xC1 => { cpu.registers.c |= 0b0000_0001; Executed(8) }, //SET 0,C
    0xC2 => { cpu.registers.d |= 0b0000_0001; Executed(8) }, //SET 0,D
    0xC3 => { cpu.registers.e |= 0b0000_0001; Executed(8) }, //SET 0,E
    0xC4 => { cpu.registers.h |= 0b0000_0001; Executed(8) }, //SET 0,H
    0xC5 => { cpu.registers.l |= 0b0000_0001; Executed(8) }, //SET 0,L
    0xC6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0000_0001; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 0,(HL)
    0xC7 => { cpu.registers.a |= 0b0000_0001; Executed(8) }, //SET 0,A
    0xC8 => { cpu.registers.b |= 0b0000_0010; Executed(8) }, //SET 1,B
    0xC9 => { cpu.registers.c |= 0b0000_0010; Executed(8) }, //SET 1,C
    0xCA => { cpu.registers.d |= 0b0000_0010; Executed(8) }, //SET 1,D
    0xCB => { cpu.registers.e |= 0b0000_0010; Executed(8) }, //SET 1,E
    0xCC => { cpu.registers.h |= 0b0000_0010; Executed(8) }, //SET 1,H
    0xCD => { cpu.registers.l |= 0b0000_0010; Executed(8) }, //SET 1,L
    0xCE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0000_0010; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 1,(HL)
    0xCF => { cpu.registers.a |= 0b0000_0010; Executed(8) }, //SET 1,A
    0xD0 => { cpu.registers.b |= 0b0000_0100; Executed(8) }, //SET 2,B
    0xD1 => { cpu.registers.c |= 0b0000_0100; Executed(8) }, //SET 2,C
    0xD2 => { cpu.registers.d |= 0b0000_0100; Executed(8) }, //SET 2,D
    0xD3 => { cpu.registers.e |= 0b0000_0100; Executed(8) }, //SET 2,E
    0xD4 => { cpu.registers.h |= 0b0000_0100; Executed(8) }, //SET 2,H
    0xD5 => { cpu.registers.l |= 0b0000_0100; Executed(8) }, //SET 2,L
    0xD6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0000_0100; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 2,(HL)
    0xD7 => { cpu.registers.a |= 0b0000_0100; Executed(8) }, //SET 2,A
    0xD8 => { cpu.registers.b |= 0b0000_1000; Executed(8) }, //SET 3,B
    0xD9 => { cpu.registers.c |= 0b0000_1000; Executed(8) }, //SET 3,C
    0xDA => { cpu.registers.d |= 0b0000_1000; Executed(8) }, //SET 3,D
    0xDB => { cpu.registers.e |= 0b0000_1000; Executed(8) }, //SET 3,E
    0xDC => { cpu.registers.h |= 0b0000_1000; Executed(8) }, //SET 3,H
    0xDD => { cpu.registers.l |= 0b0000_1000; Executed(8) }, //SET 3,L
    0xDE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0000_1000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 3,(HL)
    0xDF => { cpu.registers.a |= 0b0000_1000; Executed(8) }, //SET 3,A
    0xE0 => { cpu.registers.b |= 0b0001_0000; Executed(8) }, //SET 4,B
    0xE1 => { cpu.registers.c |= 0b0001_0000; Executed(8) }, //SET 4,C
    0xE2 => { cpu.registers.d |= 0b0001_0000; Executed(8) }, //SET 4,D
    0xE3 => { cpu.registers.e |= 0b0001_0000; Executed(8) }, //SET 4,E
    0xE4 => { cpu.registers.h |= 0b0001_0000; Executed(8) }, //SET 4,H
    0xE5 => { cpu.registers.l |= 0b0001_0000; Executed(8) }, //SET 4,L
    0xE6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0001_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 4,(HL)
    0xE7 => { cpu.registers.a |= 0b0001_0000; Executed(8) }, //SET 4,A
    0xE8 => { cpu.registers.b |= 0b0010_0000; Executed(8) }, //SET 5,B
    0xE9 => { cpu.registers.c |= 0b0010_0000; Executed(8) }, //SET 5,C
    0xEA => { cpu.registers.d |= 0b0010_0000; Executed(8) }, //SET 5,D
    0xEB => { cpu.registers.e |= 0b0010_0000; Executed(8) }, //SET 5,E
    0xEC => { cpu.registers.h |= 0b0010_0000; Executed(8) }, //SET 5,H
    0xED => { cpu.registers.l |= 0b0010_0000; Executed(8) }, //SET 5,L
    0xEE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0010_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 5,(HL)
    0xEF => { cpu.registers.a |= 0b0010_0000; Executed(8) }, //SET 5,A
    0xF0 => { cpu.registers.b |= 0b0100_0000; Executed(8) }, //SET 6,B
    0xF1 => { cpu.registers.c |= 0b0100_0000; Executed(8) }, //SET 6,C
    0xF2 => { cpu.registers.d |= 0b0100_0000; Executed(8) }, //SET 6,D
    0xF3 => { cpu.registers.e |= 0b0100_0000; Executed(8) }, //SET 6,E
    0xF4 => { cpu.registers.h |= 0b0100_0000; Executed(8) }, //SET 6,H
    0xF5 => { cpu.registers.l |= 0b0100_0000; Executed(8) }, //SET 6,L
    0xF6 => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b0100_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 6,(HL)
    0xF7 => { cpu.registers.a |= 0b0100_0000; Executed(8) }, //SET 6,A
    0xF8 => { cpu.registers.b |= 0b1000_0000; Executed(8) }, //SET 7,B
    0xF9 => { cpu.registers.c |= 0b1000_0000; Executed(8) }, //SET 7,C
    0xFA => { cpu.registers.d |= 0b1000_0000; Executed(8) }, //SET 7,D
    0xFB => { cpu.registers.e |= 0b1000_0000; Executed(8) }, //SET 7,E
    0xFC => { cpu.registers.h |= 0b1000_0000; Executed(8) }, //SET 7,H
    0xFD => { cpu.registers.l |= 0b1000_0000; Executed(8) }, //SET 7,L
    0xFE => { let new_value = cpu.mmu.read_byte(cpu.registers.get_hl())  | 0b1000_0000; cpu.mmu.write_byte(cpu.registers.get_hl(), new_value); Executed(8) }, //SET 7,(HL)
    0xFF => { cpu.registers.a |= 0b1000_0000; Executed(8) }, //SET 7,A
    _ => { cpu.halted = true; Executed(4) } //unreachable, but linux compiler will complain
  }
}