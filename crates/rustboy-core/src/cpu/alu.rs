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
    //println!("{} - {} = {} flags: {:08b}",value1, value2, result, flag_register.get_raw());
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
    use super::{adc, add, add16, cp, dec, inc, sbc, sub};
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
}
