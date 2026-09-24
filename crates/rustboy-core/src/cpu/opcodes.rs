//! SM83 base and CB-prefixed decoding, using fields `xx yyy zzz`.
//!
//! `x = opcode >> 6`, `y = (opcode >> 3) & 7`, `z = opcode & 7`;
//! register-pair families further split y into `p = y >> 1`, `q = y & 1`.
//!
//! Base groups:
//! - x=0: z selects special/JR, word load/ADD HL, indirect A load, word INC/DEC,
//!   byte INC, byte DEC, immediate byte load, or accumulator/flag operations.
//! - x=1: LD r[y],r[z], except y=z=6 is HALT.
//! - x=2: ALU[y] A,r[z] (ADD, ADC, SUB, SBC, AND, XOR, OR, CP).
//! - x=3: z selects return/special, POP/special, jump/special, control,
//!   conditional CALL, PUSH/CALL, immediate ALU, or RST. Illegal slots stay explicit.
//!
//! CB groups: x=0 rotates/shifts/SWAP, x=1 BIT, x=2 RES, x=3 SET. Here y
//! selects the operation or bit index, and z selects the byte operand.
//!
//! Byte order is B,C,D,E,H,L,(HL),A. Word order is BC,DE,HL,SP, while stack
//! operations use BC,DE,HL,AF. Conditions are NZ,Z,NC,C. See the CPU README
//! for subgroup tables, worked examples and instruction-total timing limits.

use super::alu;
use super::flags::{CpuFlag, Flags};
use super::operand::Operand8;
use super::registers::RegisterName16;
use super::OpcodeResult::{Executed, UnknownOpcode};
use super::{BinaryOperation8, Cpu, CpuState, OpcodeResult, UnaryOperation8};

pub fn execute(opcode: u8, cpu: &mut Cpu) -> OpcodeResult {
    let y = (opcode >> 3) & 7;
    let z = opcode & 7;
    match opcode >> 6 {
        0b00 => Executed(execute_misc(y, z, cpu)),
        0b01 => Executed(execute_load(y, z, cpu)),
        0b10 => Executed(execute_alu(y, z, cpu)),
        _ => execute_control(y, z, cpu),
    }
}

/// x=0: z selects a family; y (or its p/q split) selects that family's operation.
fn execute_misc(y: u8, z: u8, cpu: &mut Cpu) -> usize {
    let p = y >> 1;
    let q = y & 1;
    match z {
        0 => execute_relative_or_misc(y, cpu),
        1 => {
            let pair = register_pair(p);
            if q == 0 {
                // LD rp,d16
                let value = cpu.fetch_word();
                cpu.registers.set16(pair, value);
                12
            } else {
                // ADD HL,rp
                let hl = cpu.registers.get_hl();
                let rhs = cpu.registers.get16(pair);
                let result = alu::add16(&mut cpu.registers.flags, hl, rhs);
                cpu.registers.set_hl(result);
                8
            }
        }
        2 => {
            // LD (BC/DE/HL+/HL-),A or the reverse.
            let address = match p {
                0 => cpu.registers.get_bc(),
                1 => cpu.registers.get_de(),
                2 => cpu.registers.post_increment_hl(),
                _ => cpu.registers.post_decrement_hl(),
            };
            if q == 0 {
                cpu.mmu.write_byte(address, cpu.registers.a);
            } else {
                cpu.registers.a = cpu.mmu.read_byte(address);
            }
            8
        }
        3 => {
            // INC/DEC rp (no flag changes).
            let pair = register_pair(p);
            let value = cpu.registers.get16(pair);
            let result = if q == 0 {
                value.wrapping_add(1)
            } else {
                value.wrapping_sub(1)
            };
            cpu.registers.set16(pair, result);
            8
        }
        4 | 5 => {
            // INC/DEC r8, including read/modify/write for (HL).
            let operand = Operand8::decode(y);
            let value = operand.read(cpu);
            let operation = if z == 4 { alu::inc } else { alu::dec };
            let result = operation(&mut cpu.registers.flags, value);
            operand.write(cpu, result);
            if matches!(operand, Operand8::IndirectHl) {
                12
            } else {
                4
            }
        }
        6 => {
            // LD r8,d8: do not read the destination.
            let operand = Operand8::decode(y);
            let value = cpu.fetch_byte();
            operand.write(cpu, value);
            if matches!(operand, Operand8::IndirectHl) {
                12
            } else {
                8
            }
        }
        _ => execute_accumulator_misc(y, cpu),
    }
}

/// x=0,z=0: relative branches, NOP, LD (a16),SP and STOP.
fn execute_relative_or_misc(y: u8, cpu: &mut Cpu) -> usize {
    match y {
        0 => 4, // NOP
        1 => {
            // LD (a16),SP
            let address = cpu.fetch_word();
            cpu.mmu.write_word(address, cpu.registers.sp);
            20
        }
        2 => {
            // STOP: consume padding and enter its distinct state; wake behavior
            // remains the current HALT-like approximation in Cpu::handle_irq.
            cpu.fetch_byte();
            cpu.state = CpuState::Stopped;
            4
        }
        3 => {
            // JR e8
            cpu.jump_r();
            12
        }
        _ => {
            // JR cc,e8
            if condition_holds(y - 4, &cpu.registers.flags) {
                cpu.jump_r();
                12
            } else {
                cpu.registers.pc = cpu.registers.pc.wrapping_add(1);
                8
            }
        }
    }
}

/// x=0,z=7: accumulator rotations (which clear Z), DAA and flag control.
fn execute_accumulator_misc(y: u8, cpu: &mut Cpu) -> usize {
    let operation: UnaryOperation8 = match y {
        0 => alu::rlca,
        1 => alu::rrca,
        2 => alu::rla,
        3 => alu::rra,
        4 => alu::daa,
        5 => alu::cpl,
        6 => {
            alu::scf(&mut cpu.registers.flags);
            return 4;
        }
        _ => {
            alu::ccf(&mut cpu.registers.flags);
            return 4;
        }
    };
    cpu.registers.a = operation(&mut cpu.registers.flags, cpu.registers.a);
    4
}

/// x=1: LD r[y],r[z], with HALT handled before touching either operand.
fn execute_load(destination_index: u8, source_index: u8, cpu: &mut Cpu) -> usize {
    if destination_index == 6 && source_index == 6 {
        // 76 is HALT, not LD (HL),(HL).
        if !cpu.ime && cpu.pending_interrupts() != 0 {
            cpu.halt_bug = true;
        } else {
            cpu.state = CpuState::Halted;
        }
        return 4;
    }
    let source = Operand8::decode(source_index);
    let destination = Operand8::decode(destination_index);
    let value = source.read(cpu);
    destination.write(cpu, value);
    if matches!(source, Operand8::IndirectHl) || matches!(destination, Operand8::IndirectHl) {
        8
    } else {
        4
    }
}

/// x=2: ALU[y] A,r[z]. Reading (HL) never writes the operand back.
fn execute_alu(operation_index: u8, source_index: u8, cpu: &mut Cpu) -> usize {
    let source = Operand8::decode(source_index);
    let value = source.read(cpu);
    apply_alu(operation_index, value, cpu);
    if matches!(source, Operand8::IndirectHl) {
        8
    } else {
        4
    }
}

/// x=3: match (family z, selector y) to keep irregular encodings visible.
fn execute_control(y: u8, z: u8, cpu: &mut Cpu) -> OpcodeResult {
    let p = y >> 1;
    let cycles = match (z, y) {
        (0, 0..=3) => {
            // RET cc
            if condition_holds(y, &cpu.registers.flags) {
                cpu.return_from_call();
                20
            } else {
                8
            }
        }
        (0, 4) => {
            // LDH (a8),A
            let address = 0xFF00 + u16::from(cpu.fetch_byte());
            cpu.mmu.write_byte(address, cpu.registers.a);
            12
        }
        (0, 5) => {
            // ADD SP,e8
            cpu.registers.sp = add_sp_offset(cpu);
            16
        }
        (0, 6) => {
            // LDH A,(a8)
            let address = 0xFF00 + u16::from(cpu.fetch_byte());
            cpu.registers.a = cpu.mmu.read_byte(address);
            12
        }
        (0, 7) => {
            // LD HL,SP+e8
            let result = add_sp_offset(cpu);
            cpu.registers.set_hl(result);
            12
        }
        (1, 0 | 2 | 4 | 6) => {
            // POP rp2 (q=0)
            let value = cpu.pop();
            cpu.registers.set16(stack_pair(p), value);
            12
        }
        (1, 1) => {
            // RET
            cpu.return_from_call();
            16
        }
        (1, 3) => {
            // RETI
            cpu.ime = true;
            cpu.return_from_call();
            16
        }
        (1, 5) => {
            // JP HL
            cpu.registers.pc = cpu.registers.get_hl();
            4
        }
        (1, 7) => {
            // LD SP,HL
            cpu.registers.sp = cpu.registers.get_hl();
            8
        }
        (2, 0..=3) => {
            // JP cc,a16
            if condition_holds(y, &cpu.registers.flags) {
                cpu.registers.pc = cpu.fetch_word();
                16
            } else {
                cpu.registers.pc = cpu.registers.pc.wrapping_add(2);
                12
            }
        }
        (2, 4) => {
            // LD (FF00+C),A
            cpu.mmu
                .write_byte(0xFF00 + u16::from(cpu.registers.c), cpu.registers.a);
            8
        }
        (2, 5) => {
            // LD (a16),A
            let address = cpu.fetch_word();
            cpu.mmu.write_byte(address, cpu.registers.a);
            16
        }
        (2, 6) => {
            // LD A,(FF00+C)
            cpu.registers.a = cpu.mmu.read_byte(0xFF00 + u16::from(cpu.registers.c));
            8
        }
        (2, 7) => {
            // LD A,(a16)
            let address = cpu.fetch_word();
            cpu.registers.a = cpu.mmu.read_byte(address);
            16
        }
        (3, 0) => {
            // JP a16
            cpu.registers.pc = cpu.fetch_word();
            16
        }
        (3, 1) => {
            // CB prefix
            let opcode = cpu.fetch_byte();
            return execute_cb(opcode, cpu);
        }
        (3, 6) => {
            // DI
            cpu.ime = false;
            cpu.ei_requested = 0;
            4
        }
        (3, 7) => {
            // EI: repeating it must not postpone an already scheduled enable.
            if cpu.ei_requested == 0 {
                cpu.ei_requested = 2;
            }
            4
        }
        (4, 0..=3) => {
            // CALL cc,a16
            if condition_holds(y, &cpu.registers.flags) {
                let address = cpu.fetch_word();
                cpu.call(address);
                24
            } else {
                cpu.registers.pc = cpu.registers.pc.wrapping_add(2);
                12
            }
        }
        (5, 0 | 2 | 4 | 6) => {
            // PUSH rp2 (q=0)
            cpu.push(cpu.registers.get16(stack_pair(p)));
            16
        }
        (5, 1) => {
            // CALL a16
            let address = cpu.fetch_word();
            cpu.call(address);
            24
        }
        (6, _) => {
            // ALU A,d8, in the same operation order as x=2.
            let value = cpu.fetch_byte();
            apply_alu(y, value, cpu);
            8
        }
        (7, _) => {
            // RST y*8
            cpu.call(u16::from(y) * 8);
            16
        }
        // The only remaining encodings: D3 DB DD E3 E4 EB EC ED F4 FC FD.
        _ => return UnknownOpcode,
    };
    Executed(cycles)
}

fn execute_cb(opcode: u8, cpu: &mut Cpu) -> OpcodeResult {
    let group = opcode >> 6;
    let operation_or_bit = (opcode >> 3) & 7;
    let operand = Operand8::decode(opcode);
    let value = operand.read(cpu);
    match group {
        0 => {
            let operation: UnaryOperation8 = match operation_or_bit {
                0 => alu::rlc,
                1 => alu::rrc,
                2 => alu::rl,
                3 => alu::rr,
                4 => alu::sla,
                5 => alu::sra,
                6 => alu::swap,
                _ => alu::srl,
            };
            let result = operation(&mut cpu.registers.flags, value);
            operand.write(cpu, result);
        }
        1 => alu::bit(&mut cpu.registers.flags, operation_or_bit, value), // No writeback.
        2 => operand.write(cpu, value & !(1 << operation_or_bit)),        // RES
        _ => operand.write(cpu, value | (1 << operation_or_bit)),         // SET
    }
    // Includes the prefix; BIT avoids the (HL) write cycle.
    Executed(match operand {
        Operand8::Register(_) => 8,
        Operand8::IndirectHl if group == 1 => 12,
        Operand8::IndirectHl => 16,
    })
}

fn apply_alu(operation: u8, rhs: u8, cpu: &mut Cpu) {
    let operation: BinaryOperation8 = match operation {
        0 => alu::add,
        1 => alu::adc,
        2 => alu::sub,
        3 => alu::sbc,
        4 => alu::and,
        5 => alu::xor,
        6 => alu::or,
        _ => alu::cp,
    };
    cpu.registers.a = operation(&mut cpu.registers.flags, cpu.registers.a, rhs);
}

fn condition_holds(index: u8, flags: &Flags) -> bool {
    match index {
        0 => !flags.get_flag(CpuFlag::Z), // NZ
        1 => flags.get_flag(CpuFlag::Z),  // Z
        2 => !flags.get_flag(CpuFlag::C), // NC
        _ => flags.get_flag(CpuFlag::C),  // C
    }
}

// Both helpers receive p, the two-bit register-pair field.
fn register_pair(p: u8) -> RegisterName16 {
    match p {
        0 => RegisterName16::BC,
        1 => RegisterName16::DE,
        2 => RegisterName16::HL,
        _ => RegisterName16::SP,
    }
}

fn stack_pair(p: u8) -> RegisterName16 {
    match p {
        0 => RegisterName16::BC,
        1 => RegisterName16::DE,
        2 => RegisterName16::HL,
        _ => RegisterName16::AF,
    }
}

fn add_sp_offset(cpu: &mut Cpu) -> u16 {
    let sp = cpu.registers.sp;
    let offset = cpu.fetch_byte() as i8 as i16 as u16;
    alu::add_next_signed_byte_to_word(&mut cpu.registers.flags, sp, offset)
}
