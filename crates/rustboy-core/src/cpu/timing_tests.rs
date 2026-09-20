use super::{op_codes, tests::cpu_with_program, Cpu, OpCodeResult};

// SM83 instruction durations in T-cycles, with conditional branches NOT taken.
// Reference: https://gbdev.io/gb-opcodes/optables/ (normal-speed DMG).
// Zero marks an illegal opcode or the CB prefix, which is tested separately.
const BASE_CYCLES: [[usize; 16]; 16] = [
    [4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4],
    [4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4],
    [8, 12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4],
    [8, 12, 8, 8, 12, 12, 12, 4, 8, 8, 8, 8, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4],
    [8, 12, 12, 16, 12, 16, 8, 16, 8, 16, 12, 0, 12, 24, 8, 16],
    [8, 12, 12, 0, 12, 16, 8, 16, 8, 16, 12, 0, 12, 0, 8, 16],
    [12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16],
    [12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16],
];

fn cpu(program: &[u8], flags: u8) -> Cpu {
    let mut cpu = cpu_with_program(program);
    cpu.registers.set_af(0x1200 | u16::from(flags));
    cpu.registers.set_bc(0xC080);
    cpu.registers.set_de(0xC100);
    cpu.registers.set_hl(0xC200);
    cpu.registers.sp = 0xC300;
    cpu.mmu.write_word(0xC300, 0xC400);
    cpu.mmu.write_byte(0xC200, 0x81);
    cpu.mmu.write_byte(0xFFFF, 0);
    cpu.mmu.write_byte(0xFF0F, 0);
    cpu
}

#[test]
fn every_base_opcode_has_the_documented_duration_for_all_flag_combinations() {
    let mut failures = Vec::new();
    for opcode in 0..=u8::MAX {
        let base = BASE_CYCLES[usize::from(opcode >> 4)][usize::from(opcode & 15)];
        if opcode == 0xCB {
            continue;
        }
        for flags in (0..=u8::MAX).step_by(16) {
            // Immediate word targets WRAM; STOP consumes its mandatory zero byte.
            let mut cpu = cpu(&[opcode, 0, 0xC4], flags);
            if base == 0 {
                assert!(matches!(
                    op_codes::execute(opcode, &mut cpu),
                    OpCodeResult::UnknownOpCode
                ));
                continue;
            }
            let condition = match opcode {
                0x20 | 0xC0 | 0xC2 | 0xC4 => flags & 0x80 == 0, // NZ
                0x28 | 0xC8 | 0xCA | 0xCC => flags & 0x80 != 0, // Z
                0x30 | 0xD0 | 0xD2 | 0xD4 => flags & 0x10 == 0, // NC
                0x38 | 0xD8 | 0xDA | 0xDC => flags & 0x10 != 0, // C
                _ => false,
            };
            let expected = base
                + if condition {
                    match opcode {
                        0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xC4 | 0xCC | 0xD4 | 0xDC => 12,
                        _ => 4,
                    }
                } else {
                    0
                };
            let result = cpu.tick();
            assert_eq!(result.opcode, Some(opcode));
            if result.cycles != expected {
                failures.push(format!(
                    "opcode {opcode:02X}, flags {flags:02X}: expected {expected}, got {}",
                    result.cycles
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_cb_opcode_includes_prefix_and_memory_access_cycles() {
    let mut failures = Vec::new();
    for opcode in 0..=u8::MAX {
        // Register operations take 8 cycles; BIT (HL) reads memory (12),
        // while the other (HL) operations read AND write memory (16).
        let expected = if opcode & 7 != 6 {
            8
        } else if (0x40..0x80).contains(&opcode) {
            12
        } else {
            16
        };
        for flags in (0..=u8::MAX).step_by(16) {
            let mut cpu = cpu(&[0xCB, opcode], flags);
            let result = cpu.tick();
            assert_eq!(result.opcode, Some(0xCB));
            assert_eq!(cpu.registers.pc, 0x102);
            if result.cycles != expected {
                failures.push(format!(
                    "CB {opcode:02X}, flags {flags:02X}: expected {expected}, got {}",
                    result.cycles
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
