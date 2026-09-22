use crate::cpu::registers::*;

pub fn and(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let result = value1 & value2;
    flag_register.reset_flags();
    flag_register.set_flag(CpuFlag::Z, result == 0x00);
    flag_register.set_flag(CpuFlag::H, true);
    result
}

pub fn or(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let result = value1 | value2;
    flag_register.reset_flags();
    flag_register.set_flag(CpuFlag::Z, result == 0);
    result
}

pub fn xor(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let result = value1 ^ value2;
    flag_register.reset_flags();
    flag_register.set_flag(CpuFlag::Z, result == 0);
    result
}

pub fn cpl(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    let result = value ^ 0xFF;
    flag_register.set_flag(CpuFlag::N, true);
    flag_register.set_flag(CpuFlag::H, true);
    result
}

pub fn inc(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    let result = value.wrapping_add(1);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(CpuFlag::H, (value & 0x0F) + 1 > 0x0F); //a half carry occurs when the low nibble + 1 is greater than 0x0F
    result
}

pub fn dec(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    let result = value.wrapping_sub(1);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, true);
    flag_register.set_flag(CpuFlag::H, (value & 0x0F) == 0); //a half carry will occur when the low nibble is all zeros
    result
}

pub fn add(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let result = value1.wrapping_add(value2);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(
        CpuFlag::H,
        (((value1 & 0x0F) + (value2 & 0x0F)) & 0x10) == 0x10,
    );
    flag_register.set_flag(CpuFlag::C, value1 as usize + value2 as usize > 0xFF);
    result
}

pub fn add16(flag_register: &mut dyn FlagRegister, value1: u16, value2: u16) -> u16 {
    let result = value1.wrapping_add(value2);
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(CpuFlag::H, ((value1 & 0x0FFF) + (value2 & 0x0FFF)) > 0x0FFF);
    flag_register.set_flag(CpuFlag::C, value1 > 0xFFFF - value2);
    result
}

pub fn adc(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    //like add + carry flag
    let c: u8 = if flag_register.get_flag(CpuFlag::C) {
        1
    } else {
        0
    };
    let result = value1.wrapping_add(value2).wrapping_add(c);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(
        CpuFlag::H,
        (((value1 & 0x0F) + (value2 & 0x0F) + c) & 0x10) == 0x10,
    );
    flag_register.set_flag(CpuFlag::C, value1 as u16 + value2 as u16 + c as u16 > 0xFF);
    result
}

pub fn sub(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let result = value1.wrapping_sub(value2);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, true);
    flag_register.set_flag(CpuFlag::H, (value1 & 0x0F) < (value2 & 0x0F));
    flag_register.set_flag(CpuFlag::C, (value1 as u16) < (value2 as u16));
    result
}

pub fn sbc(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    let c: u8 = if flag_register.get_flag(CpuFlag::C) {
        1
    } else {
        0
    };
    let result = value1.wrapping_sub(value2).wrapping_sub(c);
    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::N, true);
    flag_register.set_flag(CpuFlag::H, (value1 & 0x0F) < (value2 & 0x0F) + c);
    flag_register.set_flag(CpuFlag::C, (value1 as u16) < (value2 as u16) + (c as u16));
    result
}

pub fn cp(flag_register: &mut dyn FlagRegister, value1: u8, value2: u8) -> u8 {
    sub(flag_register, value1, value2);
    value1
}

pub fn swap(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    flag_register.reset_flags();
    flag_register.set_flag(CpuFlag::Z, value == 0);
    value.rotate_right(4)
}

pub fn add_next_signed_byte_to_word(
    flag_register: &mut dyn FlagRegister,
    value1: u16,
    value2: u16,
) -> u16 {
    flag_register.reset_flags();

    flag_register.set_flag(CpuFlag::H, (value1 & 0x000F) + (value2 & 0x000F) > 0x000F);
    flag_register.set_flag(CpuFlag::C, (value1 & 0x00FF) + (value2 & 0x00FF) > 0x00FF);

    value1.wrapping_add(value2)
}

pub fn daa(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //i got no idea what i'm doing
    let mut adjust = if flag_register.get_flag(CpuFlag::C) {
        0x60
    } else {
        0x00
    };
    if flag_register.get_flag(CpuFlag::H) {
        adjust |= 0x06;
    }
    let result = if !flag_register.get_flag(CpuFlag::N) {
        if value & 0x0F > 0x09 {
            adjust |= 0x06;
        };
        if value > 0x99 {
            adjust |= 0x60;
        };
        value.wrapping_add(adjust)
    } else {
        value.wrapping_sub(adjust)
    };

    flag_register.set_flag(CpuFlag::Z, result == 0);
    flag_register.set_flag(CpuFlag::H, false);
    flag_register.set_flag(CpuFlag::C, adjust >= 0x60);
    result
}

fn shift_operation_flag_update_without_z(
    flag_register: &mut dyn FlagRegister,
    _result: u8,
    new_carry: bool,
) {
    flag_register.reset_flags();
    flag_register.set_flag(CpuFlag::C, new_carry);
}

fn shift_operation_flag_update(flag_register: &mut dyn FlagRegister, result: u8, new_carry: bool) {
    shift_operation_flag_update_without_z(flag_register, result, new_carry);
    flag_register.set_flag(CpuFlag::Z, result == 0);
}

fn rotate_left_through_carry(
    flag_register: &mut dyn FlagRegister,
    value: u8,
    flag_update_function: fn(&mut dyn FlagRegister, u8, bool),
) -> u8 {
    let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
    let result = (value << 1)
        | if flag_register.get_flag(CpuFlag::C) {
            0x01
        } else {
            0x00
        }; //push one to the right and add the carry to the right
    flag_update_function(flag_register, result, new_carry);
    result
}

pub fn rl(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate left through carry
    rotate_left_through_carry(flag_register, value, shift_operation_flag_update)
}

//rla, rlca, rra and rrca don't set the Z flag - different to the CB instructions
pub fn rla(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate left through carry
    rotate_left_through_carry(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_left(
    flag_register: &mut dyn FlagRegister,
    value: u8,
    flag_update_function: fn(&mut dyn FlagRegister, u8, bool),
) -> u8 {
    let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
    let result = (value << 1) | if new_carry { 0x01 } else { 0x00 }; //push one to the left and add the pushed out bit to the right
    flag_update_function(flag_register, result, new_carry);
    result
}

pub fn rlc(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate left
    rotate_left(flag_register, value, shift_operation_flag_update)
}

pub fn rlca(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate left
    rotate_left(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_right_through_carry(
    flag_register: &mut dyn FlagRegister,
    value: u8,
    flag_update_function: fn(&mut dyn FlagRegister, u8, bool),
) -> u8 {
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1)
        | if flag_register.get_flag(CpuFlag::C) {
            0x80
        } else {
            0x00
        };
    flag_update_function(flag_register, result, new_carry);
    result
}

pub fn rr(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate right through carry
    rotate_right_through_carry(flag_register, value, shift_operation_flag_update)
}

pub fn rra(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate right through carry
    rotate_right_through_carry(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_right(
    flag_register: &mut dyn FlagRegister,
    value: u8,
    flag_update_function: fn(&mut dyn FlagRegister, u8, bool),
) -> u8 {
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1) | if new_carry { 0x80 } else { 0x00 };
    flag_update_function(flag_register, result, new_carry);
    result
}

pub fn rrc(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate right
    rotate_right(flag_register, value, shift_operation_flag_update)
}

pub fn rrca(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //rotate right
    rotate_right(flag_register, value, shift_operation_flag_update_without_z)
}

//difference between shift and rotate is, that we don't add the pushed out bit on the other side
pub fn sla(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //shift left arithmetic (b0=0)
    let new_carry = (value & 0x80) == 0x80;
    let result = value << 1;
    shift_operation_flag_update(flag_register, result, new_carry);
    result
}

pub fn sra(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //shift left arithmetic (b0=0)
    let new_carry = (value & 0x01) == 0x01;
    let result = (value >> 1) | (value & 0x80);
    shift_operation_flag_update(flag_register, result, new_carry);
    result
}

pub fn srl(flag_register: &mut dyn FlagRegister, value: u8) -> u8 {
    //shift left arithmetic (b0=0)
    let new_carry = (value & 0x01) == 0x01;
    let result = value >> 1;
    shift_operation_flag_update(flag_register, result, new_carry);
    result
}

pub fn ccf(flag_register: &mut dyn FlagRegister) {
    //compliment carry flag
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(CpuFlag::H, false);
    flag_register.set_flag(CpuFlag::C, !flag_register.get_flag(CpuFlag::C));
}

pub fn scf(flag_register: &mut dyn FlagRegister) {
    //set carry flag
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(CpuFlag::H, false);
    flag_register.set_flag(CpuFlag::C, true);
}

pub fn bit(flag_register: &mut dyn FlagRegister, bit: u8, value: u8) {
    //check bit at
    flag_register.set_flag(CpuFlag::Z, (value & (1 << bit)) == 0);
    flag_register.set_flag(CpuFlag::N, false);
    flag_register.set_flag(CpuFlag::H, true);
}

#[cfg(test)]
mod tests {
    use super::{
        adc, add, add16, add_next_signed_byte_to_word, and, bit, ccf, cp, cpl, daa, dec, inc, or,
        rl, rla, rlc, rlca, rr, rra, rrc, rrca, sbc, scf, sla, sra, srl, sub, swap, xor,
    };
    use crate::cpu::registers::Registers;
    use crate::cpu::{BinaryOperation8, UnaryOperation8};

    // Use a signed, widened result for overflow/borrow and the operand/result
    // XOR identity for carry/borrow across bit 3. This avoids reproducing the
    // implementation's masked-nibble addition and comparison expressions.
    fn arithmetic_result(lhs: u8, rhs: u8, wide: i16, subtract: bool) -> (u8, u8) {
        let result = wide as u8;
        let flags = (u8::from(result == 0) << 7)
            | (u8::from(subtract) << 6)
            | (((lhs ^ rhs ^ result) & 0x10) << 1)
            | (u8::from(!(0..=255).contains(&wide)) << 4);
        (result, flags)
    }

    fn check_binary(operation: BinaryOperation8, expected: impl Fn(u8, u8, u8) -> (u8, u8)) {
        let mut registers = Registers::new();
        for lhs in 0..=u8::MAX {
            for rhs in 0..=u8::MAX {
                for flags in (0..=0xF0u8).step_by(0x10) {
                    registers.set_af(u16::from(lhs) << 8 | u16::from(flags));
                    let result = operation(&mut registers, lhs, rhs);
                    assert_eq!(
                        (result, registers.get_af() as u8),
                        expected(lhs, rhs, flags),
                        "lhs={lhs:02X}, rhs={rhs:02X}, initial flags={flags:02X}"
                    );
                }
            }
        }
    }

    fn check_unary(operation: UnaryOperation8, expected: impl Fn(u8, u8) -> (u8, u8)) {
        let mut registers = Registers::new();
        for value in 0..=u8::MAX {
            for flags in (0..=0xF0u8).step_by(0x10) {
                registers.set_af(u16::from(value) << 8 | u16::from(flags));
                let result = operation(&mut registers, value);
                assert_eq!(
                    (result, registers.get_af() as u8),
                    expected(value, flags),
                    "value={value:02X}, initial flags={flags:02X}"
                );
            }
        }
    }

    #[test]
    fn add_matches_widened_arithmetic_for_all_operands_and_flags() {
        check_binary(add, |lhs, rhs, _| {
            arithmetic_result(lhs, rhs, i16::from(lhs) + i16::from(rhs), false)
        });
    }

    #[test]
    fn adc_matches_widened_arithmetic_for_all_operands_and_flags() {
        check_binary(adc, |lhs, rhs, flags| {
            let carry = i16::from(flags & 0x10 != 0);
            arithmetic_result(lhs, rhs, i16::from(lhs) + i16::from(rhs) + carry, false)
        });
    }

    #[test]
    fn sub_matches_widened_arithmetic_for_all_operands_and_flags() {
        check_binary(sub, |lhs, rhs, _| {
            arithmetic_result(lhs, rhs, i16::from(lhs) - i16::from(rhs), true)
        });
    }

    #[test]
    fn sbc_matches_widened_arithmetic_for_all_operands_and_flags() {
        check_binary(sbc, |lhs, rhs, flags| {
            let borrow = i16::from(flags & 0x10 != 0);
            arithmetic_result(lhs, rhs, i16::from(lhs) - i16::from(rhs) - borrow, true)
        });
    }

    #[test]
    fn cp_returns_left_operand_and_sets_subtraction_flags_for_all_inputs() {
        check_binary(cp, |lhs, rhs, _| {
            let (_, flags) = arithmetic_result(lhs, rhs, i16::from(lhs) - i16::from(rhs), true);
            (lhs, flags)
        });
    }

    #[test]
    fn inc_updates_znh_and_preserves_carry_for_all_inputs() {
        check_unary(inc, |value, initial_flags| {
            let (result, flags) = arithmetic_result(value, 1, i16::from(value) + 1, false);
            (result, (flags & !0x10) | (initial_flags & 0x10))
        });
    }

    #[test]
    fn dec_updates_znh_and_preserves_carry_for_all_inputs() {
        check_unary(dec, |value, initial_flags| {
            let (result, flags) = arithmetic_result(value, 1, i16::from(value) - 1, true);
            (result, (flags & !0x10) | (initial_flags & 0x10))
        });
    }

    #[test]
    fn add16_updates_carry_flags_clears_n_and_preserves_z() {
        // ADD HL,rr sets H on carry from bit 11, not bit 10. Expected flag
        // bytes below contain H/C only; Z is preserved and N is always cleared.
        for (lhs, rhs, expected_result, expected_hc) in [
            (0x0800, 0x0800, 0x1000, 0x20), // Bit 11 carry without bit 10 carry.
            (0x07FF, 0x0001, 0x0800, 0x00), // Bit 10 carry alone must not set H.
            (0x0FFF, 0x0001, 0x1000, 0x20),
            (0x8000, 0x8000, 0x0000, 0x10), // Full carry without half-carry.
            (0xFFFF, 0x0001, 0x0000, 0x30), // Both carries and a wrapped result.
            (0xFFFF, 0xFFFF, 0xFFFE, 0x30),
            (0x1234, 0x0001, 0x1235, 0x00),
            (0x0000, 0x0000, 0x0000, 0x00), // A zero result must not change Z.
        ] {
            for initial_flags in (0..=0xF0).step_by(0x10) {
                let mut registers = Registers::new();
                registers.set_af(initial_flags);

                let result = add16(&mut registers, lhs, rhs);

                assert_eq!(result, expected_result, "{lhs:04X} + {rhs:04X}");
                assert_eq!(
                    registers.get_af() as u8,
                    (initial_flags as u8 & 0x80) | expected_hc,
                    "{lhs:04X} + {rhs:04X}, initial flags {initial_flags:02X}"
                );
            }
        }
    }

    // Flag contracts: https://gbdev.io/gb-opcodes/optables/
    #[test]
    fn logical_operations_set_zero_and_reset_flags_for_all_inputs() {
        check_binary(and, |lhs, rhs, _| {
            let result = lhs & rhs;
            (result, (u8::from(result == 0) << 7) | 0x20)
        });
        check_binary(or, |lhs, rhs, _| {
            let result = lhs | rhs;
            (result, u8::from(result == 0) << 7)
        });
        check_binary(xor, |lhs, rhs, _| {
            let result = lhs ^ rhs;
            (result, u8::from(result == 0) << 7)
        });
    }

    fn check_shift(
        operation: UnaryOperation8,
        expected: impl Fn(u8, bool) -> (u8, bool),
        accumulator: bool,
    ) {
        check_unary(operation, |value, flags| {
            let (result, carry) = expected(value, flags & 0x10 != 0);
            let flags = (u8::from(!accumulator && result == 0) << 7) | (u8::from(carry) << 4);
            (result, flags)
        });
    }

    #[test]
    fn circular_rotates_distinguish_accumulator_and_cb_zero_flags() {
        for (operation, accumulator) in [(rlc as UnaryOperation8, false), (rlca, true)] {
            check_shift(
                operation,
                |value, _| (value.rotate_left(1), value >= 128),
                accumulator,
            );
        }
        for (operation, accumulator) in [(rrc as UnaryOperation8, false), (rrca, true)] {
            check_shift(
                operation,
                |value, _| (value.rotate_right(1), value % 2 != 0),
                accumulator,
            );
        }
    }

    #[test]
    fn carry_rotates_distinguish_accumulator_and_cb_zero_flags() {
        for (operation, accumulator) in [(rl as UnaryOperation8, false), (rla, true)] {
            check_shift(
                operation,
                |value, carry| {
                    (
                        (u16::from(value) * 2 + u16::from(carry)) as u8,
                        value >= 128,
                    )
                },
                accumulator,
            );
        }
        for (operation, accumulator) in [(rr as UnaryOperation8, false), (rra, true)] {
            check_shift(
                operation,
                |value, carry| (value / 2 + u8::from(carry) * 128, value % 2 != 0),
                accumulator,
            );
        }
    }

    #[test]
    fn shifts_and_swap_match_bit_movement_for_all_inputs() {
        check_shift(
            sla,
            |value, _| ((u16::from(value) * 2) as u8, value >= 128),
            false,
        );
        check_shift(
            sra,
            |value, _| (((value as i8) >> 1) as u8, value % 2 != 0),
            false,
        );
        check_shift(srl, |value, _| (value / 2, value % 2 != 0), false);
        check_unary(swap, |value, _| {
            (value % 16 * 16 + value / 16, u8::from(value == 0) << 7)
        });
    }

    #[test]
    fn bit_sets_znh_and_preserves_carry_for_every_bit_and_input() {
        let mut registers = Registers::new();
        for index in 0..8 {
            for value in 0..=u8::MAX {
                for flags in (0..=0xF0u8).step_by(0x10) {
                    registers.set_af(u16::from(value) << 8 | u16::from(flags));
                    bit(&mut registers, index, value);
                    let zero = (value >> index) & 1 == 0;
                    let expected = (u8::from(zero) << 7) | 0x20 | (flags & 0x10);
                    assert_eq!(
                        registers.get_af(),
                        u16::from(value) << 8 | u16::from(expected),
                        "BIT {index}, value={value:02X}, flags={flags:02X}"
                    );
                }
            }
        }
    }

    #[test]
    fn complement_and_carry_controls_preserve_unaffected_flags() {
        check_unary(cpl, |value, flags| (!value, (flags & 0x90) | 0x60));
        // Adapt flag-only instructions to the unary harness.
        check_unary(
            |registers, value| {
                scf(registers);
                value
            },
            |value, flags| (value, (flags & 0x80) | 0x10),
        );
        check_unary(
            |registers, value| {
                ccf(registers);
                value
            },
            |value, flags| (value, (flags & 0x80) | ((!flags) & 0x10)),
        );
    }

    fn packed_bcd(decimal: i16) -> u8 {
        ((decimal / 10) * 16 + decimal % 10) as u8
    }

    #[test]
    fn daa_matches_decimal_arithmetic_for_all_valid_bcd_operands() {
        let mut registers = Registers::new();
        // Derive DAA inputs from binary arithmetic, but its expected output from
        // decimal arithmetic, independently of the implementation's corrections.
        for lhs in 0..100 {
            for rhs in 0..100 {
                for carry in 0..=1 {
                    for subtract in [false, true] {
                        let a = packed_bcd(lhs);
                        let b = packed_bcd(rhs);
                        let (binary, decimal) = if subtract {
                            (i16::from(a) - i16::from(b) - carry, lhs - rhs - carry)
                        } else {
                            (i16::from(a) + i16::from(b) + carry, lhs + rhs + carry)
                        };
                        let (input, flags) = arithmetic_result(a, b, binary, subtract);
                        let expected = packed_bcd(decimal.rem_euclid(100));
                        let expected_flags = (u8::from(expected == 0) << 7)
                            | (u8::from(subtract) << 6)
                            | (u8::from(!(0..100).contains(&decimal)) << 4);
                        for initial_z in [0, 0x80] {
                            registers.set_af(u16::from((flags & !0x80) | initial_z));
                            let result = daa(&mut registers, input);
                            assert_eq!((result, registers.get_af() as u8), (expected, expected_flags),
                                "BCD {lhs}, {rhs}, carry={carry}, subtract={subtract}, Z={initial_z:02X}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn daa_handles_correction_boundaries_and_non_bcd_inputs() {
        // Explicit SM83 correction-rule cases, including states not normally
        // produced by valid packed-BCD arithmetic. Tuples are A, F, result, F'.
        for (value, flags, expected, expected_flags) in [
            (0x09, 0x00, 0x09, 0x00),
            (0x0A, 0x00, 0x10, 0x00),
            (0x99, 0x00, 0x99, 0x00),
            (0x9A, 0x00, 0x00, 0x90),
            (0xFA, 0x00, 0x60, 0x10),
            (0x00, 0x20, 0x06, 0x00),
            (0x00, 0x30, 0x66, 0x10),
            (0xA0, 0x10, 0x00, 0x90),
            (0xFF, 0x40, 0xFF, 0x40),
            (0x00, 0x60, 0xFA, 0x40),
            (0x00, 0x50, 0xA0, 0x50),
            (0x00, 0x70, 0x9A, 0x50),
            (0x66, 0x70, 0x00, 0xD0),
        ] {
            let mut registers = Registers::new();
            registers.set_af(flags);
            assert_eq!(
                (daa(&mut registers, value), registers.get_af() as u8),
                (expected, expected_flags),
                "A={value:02X}, F={flags:02X}"
            );
        }
    }

    #[test]
    fn signed_word_addition_checks_low_byte_carries_and_wraps_at_boundaries() {
        let mut registers = Registers::new();
        for sp in [
            0x0000u16, 0x0001, 0x000F, 0x0010, 0x007F, 0x0080, 0x00FF, 0x0100, 0x0FFF, 0x7FFF,
            0x8000, 0xFF00, 0xFF7F, 0xFFF0, 0xFFFF,
        ] {
            for offset in i8::MIN..=i8::MAX {
                let operand = i16::from(offset) as u16;
                let expected = (i32::from(sp) + i32::from(offset)) as u16;
                let carries = sp ^ operand ^ expected;
                let expected_flags = (((carries & 0x10) << 1) | ((carries & 0x100) >> 4)) as u8;
                for flags in (0..=0xF0).step_by(0x10) {
                    registers.set_af(flags);
                    let result = add_next_signed_byte_to_word(&mut registers, sp, operand);
                    assert_eq!(
                        (result, registers.get_af() as u8),
                        (expected, expected_flags),
                        "SP={sp:04X}, offset={offset}, flags={flags:02X}"
                    );
                }
            }
        }
    }
}
