use super::{tests::cpu_with_program, Cpu};

const MEMORY_OPERAND: u16 = 0xC080;

// Opcode operand order from https://gbdev.io/gb-opcodes/optables/.
// Keep test setup independent of the production operand decoder.
fn operands(cpu: &Cpu) -> [u8; 8] {
    [
        cpu.registers.b,
        cpu.registers.c,
        cpu.registers.d,
        cpu.registers.e,
        cpu.registers.h,
        cpu.registers.l,
        cpu.mmu.read_byte(MEMORY_OPERAND),
        cpu.registers.a,
    ]
}

fn expected_result(opcode: u8, value: u8, flags: u8) -> (u8, u8) {
    let carry_in = u8::from(flags & 0x10 != 0);
    let (result, carry) = match opcode {
        0x00..=0x07 => (value.rotate_left(1), value >= 128),
        0x08..=0x0F => (value.rotate_right(1), value % 2 != 0),
        0x10..=0x17 => (
            (u16::from(value) * 2 + u16::from(carry_in)) as u8,
            value >= 128,
        ),
        0x18..=0x1F => (value / 2 + carry_in * 128, value % 2 != 0),
        0x20..=0x27 => ((u16::from(value) * 2) as u8, value >= 128),
        0x28..=0x2F => (((value as i8) >> 1) as u8, value % 2 != 0),
        0x30..=0x37 => (value % 16 * 16 + value / 16, false),
        0x38..=0x3F => (value / 2, value % 2 != 0),
        0x40..=0x7F => {
            let bit = (opcode - 0x40) / 8;
            return (
                value,
                (u8::from(value & (1 << bit) == 0) << 7) | 0x20 | (flags & 0x10),
            );
        }
        0x80..=0xBF => return (value & !(1 << ((opcode - 0x80) / 8)), flags),
        0xC0..=0xFF => return (value | (1 << ((opcode - 0xC0) / 8)), flags),
    };
    (
        result,
        (u8::from(result == 0) << 7) | (u8::from(carry) << 4),
    )
}

#[test]
fn every_cb_opcode_has_the_expected_effect_for_all_values_and_flags() {
    for opcode in 0..=u8::MAX {
        let mut cpu = cpu_with_program(&[0xCB, opcode]);
        let target = usize::from(opcode % 8);
        let expected_cycles = if target != 6 {
            8
        } else if (0x40..=0x7F).contains(&opcode) {
            12
        } else {
            16
        };

        for value in 0..=u8::MAX {
            for flags in (0..=0xF0u8).step_by(0x10) {
                let mut expected = [0x12, 0x34, 0x56, 0x78, 0xC0, 0x80, 0xA5, 0x9A];
                expected[target] = value;
                cpu.registers.b = expected[0];
                cpu.registers.c = expected[1];
                cpu.registers.d = expected[2];
                cpu.registers.e = expected[3];
                cpu.registers.h = expected[4];
                cpu.registers.l = expected[5];
                cpu.mmu.write_byte(MEMORY_OPERAND, expected[6]);
                cpu.registers
                    .set_af(u16::from(expected[7]) << 8 | u16::from(flags));
                cpu.registers.pc = 0x100;

                let (expected_value, expected_flags) = expected_result(opcode, value, flags);
                expected[target] = expected_value;
                let step = cpu.tick();

                assert_eq!(
                    (
                        operands(&cpu),
                        cpu.registers.flags.bits(),
                        cpu.registers.pc,
                        cpu.registers.sp,
                        step.cycles,
                        step.opcode,
                        cpu.halted
                    ),
                    (
                        expected,
                        expected_flags,
                        0x102,
                        0xFFFE,
                        expected_cycles,
                        Some(0xCB),
                        false
                    ),
                    "CB {opcode:02X}, value={value:02X}, flags={flags:02X}"
                );
            }
        }
    }
}

#[test]
fn bit_indirect_hl_does_not_write_back_to_a_side_effecting_register() {
    for opcode in (0x46..=0x7Eu8).step_by(8) {
        let mut cpu = cpu_with_program(&[0xCB, opcode]);
        cpu.registers.set_hl(0xFF04);
        cpu.mmu.do_ticks(256);
        assert_eq!(cpu.mmu.read_byte(0xFF04), 1);

        cpu.tick();

        // A write of even the same value to DIV would reset it to zero. Testing
        // ordinary RAM alone cannot distinguish no write from same-value writeback.
        assert_eq!(cpu.mmu.read_byte(0xFF04), 1, "CB {opcode:02X}");
    }
}

#[test]
fn modifying_indirect_hl_writes_back_even_when_the_value_does_not_change() {
    let opcodes = (0x06..=0x3Eu8).step_by(8).chain((0x86..=0xFEu8).step_by(8));
    for opcode in opcodes {
        let mut cpu = cpu_with_program(&[0xCB, opcode]);
        cpu.registers.set_hl(0xFF04);
        cpu.mmu.do_ticks(256);

        cpu.tick();

        // Every rotate/shift/RES/SET (HL) writes DIV, which resets it. For
        // example RES 7 leaves the read value (1) unchanged but must still write.
        assert_eq!(cpu.mmu.read_byte(0xFF04), 0, "CB {opcode:02X}");
    }
}
