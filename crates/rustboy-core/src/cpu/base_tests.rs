use super::{tests::cpu_with_program, Cpu, CpuState};

const MEMORY: u16 = 0xC080;

fn cpu(program: &[u8], flags: u8) -> Cpu {
    let mut cpu = cpu_with_program(program);
    seed(
        &mut cpu,
        [0x12, 0x34, 0x56, 0x78, 0xC0, 0x80, 0xE1, 0x9A],
        flags,
    );
    cpu.registers.sp = 0xC100;
    cpu
}

// Reference operand order is explicit here, independent of Operand8::decode.
fn values(cpu: &Cpu) -> [u8; 8] {
    [
        cpu.registers.b,
        cpu.registers.c,
        cpu.registers.d,
        cpu.registers.e,
        cpu.registers.h,
        cpu.registers.l,
        cpu.mmu.read_byte(MEMORY),
        cpu.registers.a,
    ]
}

fn seed(cpu: &mut Cpu, values: [u8; 8], flags: u8) {
    cpu.registers.b = values[0];
    cpu.registers.c = values[1];
    cpu.registers.d = values[2];
    cpu.registers.e = values[3];
    cpu.registers.h = values[4];
    cpu.registers.l = values[5];
    cpu.mmu.write_byte(MEMORY, values[6]);
    cpu.registers
        .set_af(u16::from(values[7]) << 8 | u16::from(flags));
    cpu.registers.pc = 0x100;
}

fn step(cpu: &mut Cpu, opcode: u8, cycles: usize, pc: u16) {
    let result = cpu.tick();
    assert_eq!(
        (result.opcode, result.cycles, cpu.registers.pc),
        (Some(opcode), cycles, pc),
        "opcode={opcode:02X}"
    );
}

#[test]
fn register_load_matrix_preserves_sources_flags_and_original_hl_address() {
    for opcode in 0x40..=0x7Fu8 {
        if opcode == 0x76 {
            continue;
        }
        for flags in (0..=0xF0).step_by(16) {
            let mut cpu = cpu(&[opcode], flags);
            let mut expected = values(&cpu);
            let source = usize::from(opcode % 8);
            let destination = usize::from((opcode - 0x40) / 8);
            expected[destination] = expected[source];
            step(
                &mut cpu,
                opcode,
                if source == 6 || destination == 6 {
                    8
                } else {
                    4
                },
                0x101,
            );
            assert_eq!(values(&cpu), expected, "opcode={opcode:02X}");
            assert_eq!(cpu.registers.flags.bits(), flags);
            assert_eq!(cpu.registers.sp, 0xC100);
        }
    }
}

#[test]
fn byte_increment_decrement_and_immediate_load_route_every_operand() {
    for operand in 0..8u8 {
        for operation in [4, 5, 6] {
            let opcode = operand * 8 + operation;
            let mut cpu = cpu(&[opcode, 0xA5], 0);
            for value in 0..=u8::MAX {
                for flags in (0..=0xF0).step_by(16) {
                    let mut expected = [0x12, 0x34, 0x56, 0x78, 0xC0, 0x80, 0xE1, 0x9A];
                    expected[usize::from(operand)] = value;
                    seed(&mut cpu, expected, flags);
                    let result = match operation {
                        4 => value.wrapping_add(1),
                        5 => value.wrapping_sub(1),
                        _ => 0xA5,
                    };
                    expected[usize::from(operand)] = result;
                    let expected_flags = if operation == 6 {
                        flags
                    } else {
                        (flags & 0x10)
                            | (u8::from(result == 0) << 7)
                            | (u8::from(operation == 5) << 6)
                            | (((value ^ 1 ^ result) & 0x10) << 1)
                    };
                    let cycles = if operand == 6 {
                        12
                    } else if operation == 6 {
                        8
                    } else {
                        4
                    };
                    step(
                        &mut cpu,
                        opcode,
                        cycles,
                        if operation == 6 { 0x102 } else { 0x101 },
                    );
                    assert_eq!(
                        (values(&cpu), cpu.registers.flags.bits()),
                        (expected, expected_flags),
                        "opcode={opcode:02X}, value={value:02X}, flags={flags:02X}"
                    );
                }
            }
        }
    }
}

fn alu_result(operation: usize, a: u8, b: u8, flags: u8) -> (u8, u8) {
    if (4..=6).contains(&operation) {
        let result = match operation {
            4 => a & b,
            5 => a ^ b,
            _ => a | b,
        };
        return (
            result,
            (u8::from(result == 0) << 7) | if operation == 4 { 0x20 } else { 0 },
        );
    }
    let carry = i16::from(matches!(operation, 1 | 3) && flags & 0x10 != 0);
    let subtract = matches!(operation, 2 | 3 | 7);
    let wide = if subtract {
        i16::from(a) - i16::from(b) - carry
    } else {
        i16::from(a) + i16::from(b) + carry
    };
    let result = wide as u8;
    let flags = (u8::from(result == 0) << 7)
        | (u8::from(subtract) << 6)
        | (((a ^ b ^ result) & 0x10) << 1)
        | (u8::from(!(0..=255).contains(&wide)) << 4);
    (if operation == 7 { a } else { result }, flags)
}

#[test]
fn accumulator_alu_routes_register_memory_and_immediate_operands() {
    for operation in 0..8 {
        for source in 0..9 {
            let opcode = if source == 8 {
                [0xC6, 0xCE, 0xD6, 0xDE, 0xE6, 0xEE, 0xF6, 0xFE][operation]
            } else {
                0x80 + operation as u8 * 8 + source as u8
            };
            for a in [0, 0x0F, 0x7F, 0x80, 0xFF] {
                for flags in (0..=0xF0).step_by(16) {
                    let mut cpu = cpu(&[opcode, 0x81], flags);
                    cpu.registers.a = a;
                    let mut expected = values(&cpu);
                    let rhs = if source == 8 { 0x81 } else { expected[source] };
                    let (result, expected_flags) = alu_result(operation, a, rhs, flags);
                    expected[7] = result;
                    step(
                        &mut cpu,
                        opcode,
                        if source >= 8 || source == 6 { 8 } else { 4 },
                        if source == 8 { 0x102 } else { 0x101 },
                    );
                    assert_eq!(
                        (values(&cpu), cpu.registers.flags.bits()),
                        (expected, expected_flags),
                        "opcode={opcode:02X}, A={a:02X}, flags={flags:02X}"
                    );
                }
            }
        }
    }
}

fn pairs(cpu: &Cpu) -> [u16; 4] {
    [
        cpu.registers.get_bc(),
        cpu.registers.get_de(),
        cpu.registers.get_hl(),
        cpu.registers.sp,
    ]
}

fn set_pair(cpu: &mut Cpu, pair: usize, value: u16) {
    match pair {
        0 => cpu.registers.set_bc(value),
        1 => cpu.registers.set_de(value),
        2 => cpu.registers.set_hl(value),
        _ => cpu.registers.sp = value,
    }
}

#[test]
fn word_load_increment_decrement_and_add_select_the_correct_pair() {
    for pair in 0..4 {
        for base in [0x01, 0x03, 0x0B, 0x09] {
            let opcode = base + pair as u8 * 16;
            for value in [0, 0x07FF, 0x0800, 0x0FFF, 0x8000, 0xFFFF] {
                for flags in (0..=0xF0).step_by(16) {
                    let mut cpu = cpu(&[opcode, 0x34, 0x12], flags);
                    set_pair(&mut cpu, pair, value);
                    let mut expected = pairs(&cpu);
                    let mut expected_flags = flags;
                    match base {
                        0x01 => expected[pair] = 0x1234,
                        0x03 => expected[pair] = value.wrapping_add(1),
                        0x0B => expected[pair] = value.wrapping_sub(1),
                        _ => {
                            let hl = expected[2];
                            let sum = u32::from(hl) + u32::from(value);
                            expected[2] = sum as u16;
                            expected_flags = (flags & 0x80)
                                | (((hl ^ value ^ expected[2]) & 0x1000) >> 7) as u8
                                | (u8::from(sum > 0xFFFF) << 4);
                        }
                    }
                    step(
                        &mut cpu,
                        opcode,
                        if base == 1 { 12 } else { 8 },
                        if base == 1 { 0x103 } else { 0x101 },
                    );
                    assert_eq!(
                        (pairs(&cpu), cpu.registers.flags.bits()),
                        (expected, expected_flags),
                        "opcode={opcode:02X}"
                    );
                    assert_eq!(cpu.registers.a, 0x9A);
                }
            }
        }
    }
}

#[test]
fn indirect_accumulator_loads_use_the_original_address_and_update_only_hl_when_requested() {
    for (store, load, pair, delta) in [
        (0x02, 0x0A, 0, 0),
        (0x12, 0x1A, 1, 0),
        (0x22, 0x2A, 2, 1),
        (0x32, 0x3A, 2, -1),
    ] {
        for opcode in [store, load] {
            for address in [0xC000, 0xCFFF, 0xFFFF] {
                let mut cpu = cpu(&[opcode], 0xF0);
                set_pair(&mut cpu, pair, address);
                cpu.mmu.write_byte(address, 0x35);
                let mut expected_pairs = pairs(&cpu);
                if pair == 2 {
                    expected_pairs[2] = (i32::from(address) + delta) as u16;
                }
                step(&mut cpu, opcode, 8, 0x101);
                assert_eq!(pairs(&cpu), expected_pairs);
                assert_eq!(cpu.registers.a, if opcode == store { 0x9A } else { 0x35 });
                assert_eq!(
                    cpu.mmu.read_byte(address),
                    if opcode == store { 0x9A } else { 0x35 }
                );
                assert_eq!(cpu.registers.flags.bits(), 0xF0);
            }
        }
    }
    let mut cpu = cpu(&[0x3A], 0xF0);
    cpu.registers.set_hl(0);
    step(&mut cpu, 0x3A, 8, 0x101);
    assert_eq!((cpu.registers.get_hl(), cpu.registers.a), (0xFFFF, 0));
}

#[test]
fn conditional_control_flow_checks_each_condition_and_preserves_stack_on_untaken_paths() {
    for (jr, jp, call, ret, mask, required) in [
        (0x20, 0xC2, 0xC4, 0xC0, 0x80, false),
        (0x28, 0xCA, 0xCC, 0xC8, 0x80, true),
        (0x30, 0xD2, 0xD4, 0xD0, 0x10, false),
        (0x38, 0xDA, 0xDC, 0xD8, 0x10, true),
    ] {
        for flags in (0..=0xF0).step_by(16) {
            let taken = (flags & mask != 0) == required;
            for (opcode, taken_cycles, idle_cycles) in [(jp, 16, 12), (call, 24, 12), (ret, 20, 8)]
            {
                let mut cpu = cpu(&[opcode, 0x34, 0x12], flags);
                cpu.mmu.write_word(0xC0FE, 0xBEEF);
                cpu.mmu.write_word(0xC100, 0x5678);
                let pc = if taken {
                    if opcode == ret {
                        0x5678
                    } else {
                        0x1234
                    }
                } else if opcode == ret {
                    0x101
                } else {
                    0x103
                };
                step(
                    &mut cpu,
                    opcode,
                    if taken { taken_cycles } else { idle_cycles },
                    pc,
                );
                assert_eq!(
                    cpu.registers.sp,
                    if taken && opcode == call {
                        0xC0FE
                    } else if taken && opcode == ret {
                        0xC102
                    } else {
                        0xC100
                    }
                );
                assert_eq!(
                    cpu.mmu.read_word(0xC0FE),
                    if taken && opcode == call {
                        0x103
                    } else {
                        0xBEEF
                    }
                );
                assert_eq!(cpu.mmu.read_word(0xC100), 0x5678);
                assert_eq!(cpu.registers.flags.bits(), flags);
            }
            for offset in [0, 0x7F, 0x80, 0xFF] {
                let mut cpu = cpu(&[jr, offset], flags);
                let pc = if taken {
                    (0x102i32 + i32::from(offset as i8)) as u16
                } else {
                    0x102
                };
                step(&mut cpu, jr, if taken { 12 } else { 8 }, pc);
                assert_eq!(cpu.registers.flags.bits(), flags);
                assert_eq!(cpu.registers.sp, 0xC100);
            }
        }
    }
}

#[test]
fn stack_pairs_and_restart_vectors_preserve_return_addresses_and_mask_af() {
    for (index, push, pop) in [
        (0, 0xC5, 0xC1),
        (1, 0xD5, 0xD1),
        (2, 0xE5, 0xE1),
        (3, 0xF5, 0xF1),
    ] {
        let mut cpu = cpu(&[push, pop], 0xF0);
        let original = [
            cpu.registers.get_bc(),
            cpu.registers.get_de(),
            cpu.registers.get_hl(),
            cpu.registers.get_af(),
        ];
        step(&mut cpu, push, 16, 0x101);
        assert_eq!(cpu.registers.sp, 0xC0FE);
        assert_eq!(cpu.mmu.read_word(0xC0FE), original[index]);
        cpu.mmu.write_word(0xC0FE, 0x12FF);
        step(&mut cpu, pop, 12, 0x102);
        let mut expected = original;
        expected[index] = if index == 3 { 0x12F0 } else { 0x12FF };
        assert_eq!(
            [
                cpu.registers.get_bc(),
                cpu.registers.get_de(),
                cpu.registers.get_hl(),
                cpu.registers.get_af()
            ],
            expected
        );
        assert_eq!(cpu.registers.sp, 0xC100);
    }
    for (opcode, vector) in [
        (0xC7, 0x00),
        (0xCF, 0x08),
        (0xD7, 0x10),
        (0xDF, 0x18),
        (0xE7, 0x20),
        (0xEF, 0x28),
        (0xF7, 0x30),
        (0xFF, 0x38),
    ] {
        let mut cpu = cpu(&[opcode], 0xF0);
        step(&mut cpu, opcode, 16, vector);
        assert_eq!(cpu.registers.sp, 0xC0FE);
        assert_eq!(cpu.mmu.read_word(0xC0FE), 0x101);
        assert_eq!(cpu.registers.flags.bits(), 0xF0);
    }
}

#[test]
fn special_loads_use_high_memory_absolute_addresses_and_little_endian_words() {
    for (opcode, program, cycles) in [
        (0xE0, vec![0xE0, 0x80], 12),
        (0xF0, vec![0xF0, 0x80], 12),
        (0xE2, vec![0xE2], 8),
        (0xF2, vec![0xF2], 8),
        (0xEA, vec![0xEA, 0x80, 0xFF], 16),
        (0xFA, vec![0xFA, 0x80, 0xFF], 16),
    ] {
        let mut cpu = cpu(&program, 0xF0);
        cpu.registers.c = 0x80;
        cpu.mmu.write_byte(0xFF80, 0x35);
        step(&mut cpu, opcode, cycles, 0x100 + program.len() as u16);
        let store = matches!(opcode, 0xE0 | 0xE2 | 0xEA);
        assert_eq!(
            (cpu.registers.a, cpu.mmu.read_byte(0xFF80)),
            if store { (0x9A, 0x9A) } else { (0x35, 0x35) }
        );
        assert_eq!(cpu.registers.flags.bits(), 0xF0);
    }
    let mut cpu = cpu(&[0x08, 0xFF, 0xC1, 0xF9], 0xF0);
    step(&mut cpu, 0x08, 20, 0x103);
    assert_eq!(
        (cpu.mmu.read_byte(0xC1FF), cpu.mmu.read_byte(0xC200)),
        (0x00, 0xC1)
    );
    step(&mut cpu, 0xF9, 8, 0x104);
    assert_eq!(
        (cpu.registers.sp, cpu.registers.flags.bits()),
        (MEMORY, 0xF0)
    );
}

#[test]
fn accumulator_miscellaneous_opcodes_select_the_correct_operation() {
    for (opcode, a, flags, expected_a, expected_flags) in [
        (0x07, 0x80, 0xF0, 0x01, 0x10),
        (0x0F, 0x01, 0xF0, 0x80, 0x10),
        (0x17, 0x80, 0xF0, 0x01, 0x10),
        (0x1F, 0x01, 0xF0, 0x80, 0x10),
        (0x27, 0x9A, 0x00, 0x00, 0x90),
        (0x2F, 0x55, 0x90, 0xAA, 0xF0),
        (0x37, 0x55, 0xE0, 0x55, 0x90),
        (0x3F, 0x55, 0xF0, 0x55, 0x80),
    ] {
        let mut cpu = cpu(&[opcode], flags);
        cpu.registers.a = a;
        let mut expected = values(&cpu);
        expected[7] = expected_a;
        step(&mut cpu, opcode, 4, 0x101);
        assert_eq!(
            (values(&cpu), cpu.registers.flags.bits()),
            (expected, expected_flags)
        );
    }
}

#[test]
fn unconditional_control_flow_and_stop_keep_their_special_semantics() {
    let mut cpu = cpu(&[0xCD, 0x34, 0x12], 0xF0);
    step(&mut cpu, 0xCD, 24, 0x1234);
    assert_eq!(
        (cpu.registers.sp, cpu.mmu.read_word(0xC0FE)),
        (0xC0FE, 0x103)
    );
    // Execute RET from WRAM, where the fixture can supply a handler byte.
    cpu.registers.pc = 0xC200;
    cpu.mmu.write_byte(0xC200, 0xC9);
    step(&mut cpu, 0xC9, 16, 0x103);
    assert_eq!(cpu.registers.sp, 0xC100);
    cpu.registers.pc = 0xC200;
    cpu.mmu.write_byte(0xC200, 0xE9);
    step(&mut cpu, 0xE9, 4, MEMORY);
    cpu.registers.pc = 0xC200;
    cpu.mmu.write_byte(0xC200, 0x18);
    cpu.mmu.write_byte(0xC201, 0xFE);
    step(&mut cpu, 0x18, 12, 0xC200);
    cpu.mmu.write_byte(0xC200, 0x10);
    cpu.mmu.write_byte(0xC201, 0);
    step(&mut cpu, 0x10, 4, 0xC202);
    assert_eq!(cpu.state, CpuState::Stopped);
    assert_eq!(cpu.registers.flags.bits(), 0xF0);
}

#[test]
fn halt_is_not_a_memory_load_and_alu_memory_sources_are_read_only() {
    for opcode in [0x76, 0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE] {
        let mut cpu = cpu(&[opcode], 0);
        cpu.registers.set_hl(0xFF04);
        cpu.mmu.do_ticks(256);
        step(&mut cpu, opcode, if opcode == 0x76 { 4 } else { 8 }, 0x101);
        assert_eq!(cpu.mmu.read_byte(0xFF04), 1, "opcode={opcode:02X}");
        assert_eq!(
            cpu.state,
            if opcode == 0x76 {
                CpuState::Halted
            } else {
                CpuState::Running
            }
        );
    }
}
