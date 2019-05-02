use crate::cpu::registers::*;

pub fn and(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let result = value1 & value2;
  flag_register.reset_flags();
  flag_register.set_flag(CpuFlag::Z, result == 0x00);
  flag_register.set_flag(CpuFlag::H, true);
  result
}

pub fn or(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let result = value1 | value2;
  flag_register.reset_flags();
  flag_register.set_flag(CpuFlag::Z, result == 0);
  result
}

pub fn xor(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let result = value1 ^ value2;
  flag_register.reset_flags();
  flag_register.set_flag(CpuFlag::Z, result == 0);
  result
}

pub fn cpl(flag_register: &mut FlagRegister, value: u8) -> u8 {
  let result = value ^ 0xFF;
  flag_register.set_flag(CpuFlag::N, true);
  flag_register.set_flag(CpuFlag::H, true);
  result
}

pub fn inc(flag_register: &mut FlagRegister, value: u8) -> u8 {
  let result = value.wrapping_add(1);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, (value & 0x0F) + 1 > 0x0F); //a half carry occurs when the low nibble + 1 is greater than 0x0F
  result
}

pub fn dec(flag_register: &mut FlagRegister, value: u8) -> u8 {
  let result = value.wrapping_sub(1);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, true);
  flag_register.set_flag(CpuFlag::H, (value & 0x0F) == 0); //a half carry will occur when the low nibble is all zeros
  result
}

pub fn add(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let result = value1.wrapping_add(value2);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, (((value1 & 0x0F) + (value2 & 0x0F)) & 0x10) == 0x10);
  flag_register.set_flag(CpuFlag::C, value1 as usize + value2 as usize > 0xFF);
  result
}

pub fn add16(flag_register: &mut FlagRegister, value1: u16, value2: u16) -> u16 {
  let result = value1.wrapping_add(value2);
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, ((value1 & 0x07FF) + (value2 & 0x07FF)) > 0x07FF);
  flag_register.set_flag(CpuFlag::C, value1 > 0xFFFF - value2);
  result
}

pub fn adc(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 { //like add + carry flag
  let c: u8 = if flag_register.get_flag(CpuFlag::C) { 1 } else { 0 };
  let result = value1.wrapping_add(value2).wrapping_add(c);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, (((value1 & 0x0F) + (value2 & 0x0F) + c) & 0x10) == 0x10);
  flag_register.set_flag(CpuFlag::C, value1 as u16 + value2 as u16 + c as u16 > 0xFF);
  result
}

pub fn sub(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let result = value1.wrapping_sub(value2);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, true);
  flag_register.set_flag(CpuFlag::H, (value1 & 0x0F) < (value2 & 0x0F));
  flag_register.set_flag(CpuFlag::C, (value1 as u16) < (value2 as u16));
  //println!("{} - {} = {} flags: {:08b}",value1, value2, result, flag_register.get_raw());
  result
}

pub fn sbc(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  let c: u8 = if flag_register.get_flag(CpuFlag::C) { 1 } else { 0 };
  let result = value1.wrapping_sub(value2).wrapping_sub(c);
  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::N, true);
  flag_register.set_flag(CpuFlag::H, (value1 & 0x0F) < (value2 & 0x0F) + c);
  flag_register.set_flag(CpuFlag::C, (value1 as u16) < (value2 as u16) + (c as u16));
  result
}

pub fn cp(flag_register: &mut FlagRegister, value1: u8, value2: u8) -> u8 {
  sub(flag_register, value1, value2);
  //println!("CP {:#4X} - {:#4X} - flags: {:08b}", value1, value2, flag_register.get_raw());
  value1
}

pub fn swap(flag_register: &mut FlagRegister, value: u8) -> u8 {
  flag_register.reset_flags();
  flag_register.set_flag(CpuFlag::Z, value == 0);
  (value << 4) | (value >> 4)
}

pub fn add_next_signed_byte_to_word(flag_register: &mut FlagRegister, value1: u16, value2: u16) -> u16 {
  flag_register.reset_flags();

  flag_register.set_flag(CpuFlag::H, (value1 & 0x000F) + (value2 & 0x000F) > 0x000F);
  flag_register.set_flag(CpuFlag::C, (value1 & 0x00FF) + (value2 & 0x00FF) > 0x00FF);

  value1.wrapping_add(value2)
}

pub fn daa(flag_register: &mut FlagRegister, value: u8) -> u8 {  //i got no idea what i'm doing
  let mut adjust = if flag_register.get_flag(CpuFlag::C) { 0x60 } else { 0x00 };
  if flag_register.get_flag(CpuFlag::H) { adjust |= 0x06; }
  let result = if !flag_register.get_flag(CpuFlag::N) {
    if value & 0x0F > 0x09 { adjust |= 0x06; };
    if value > 0x99 { adjust |= 0x60; };
    value.wrapping_add(adjust)
  } else {
    value.wrapping_sub(adjust)
  };

  flag_register.set_flag(CpuFlag::Z, result == 0);
  flag_register.set_flag(CpuFlag::H, false);
  flag_register.set_flag(CpuFlag::C, adjust >= 0x60);
  result
}

fn shift_operation_flag_update_without_z(flag_register: &mut FlagRegister, _result:u8, new_carry: bool) {
  flag_register.reset_flags();
  flag_register.set_flag(CpuFlag::C, new_carry);
}

fn shift_operation_flag_update(flag_register: &mut FlagRegister, result: u8, new_carry: bool) {
  shift_operation_flag_update_without_z(flag_register, result, new_carry);
  flag_register.set_flag(CpuFlag::Z, result == 0);
}

fn rotate_left_through_carry(flag_register: &mut FlagRegister, value: u8, flag_update_function: fn(&mut FlagRegister, u8, bool)) -> u8 {
  let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
  let result = (value << 1) | if flag_register.get_flag(CpuFlag::C) { 0x01 } else { 0x00 }; //push one to the right and add the carry to the right
  flag_update_function(flag_register, result, new_carry);
  result
}

pub fn rl(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate left through carry
  rotate_left_through_carry(flag_register, value, shift_operation_flag_update)
}

//rla, rlca, rra and rrca don't set the Z flag - different to the CB instructions
pub fn rla(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate left through carry
  rotate_left_through_carry(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_left(flag_register: &mut FlagRegister, value: u8, flag_update_function: fn(&mut FlagRegister, u8, bool)) -> u8 {
  let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
  let result = (value << 1) | if new_carry { 0x01 } else { 0x00 }; //push one to the left and add the pushed out bit to the right
  flag_update_function(flag_register, result, new_carry);
  result
}

pub fn rlc(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate left
  rotate_left(flag_register, value, shift_operation_flag_update)
}

pub fn rlca(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate left
  rotate_left(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_right_through_carry(flag_register: &mut FlagRegister, value: u8, flag_update_function: fn(&mut FlagRegister, u8, bool)) -> u8 {
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | if flag_register.get_flag(CpuFlag::C) { 0x80 } else { 0x00 };
  flag_update_function(flag_register, result, new_carry);
  result
}

pub fn rr(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate right through carry
  rotate_right_through_carry(flag_register, value, shift_operation_flag_update)
}

pub fn rra(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate right through carry
  rotate_right_through_carry(flag_register, value, shift_operation_flag_update_without_z)
}

fn rotate_right(flag_register: &mut FlagRegister, value: u8, flag_update_function: fn(&mut FlagRegister, u8, bool)) -> u8 {
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | if new_carry { 0x80 } else { 0x00 };
  flag_update_function(flag_register, result, new_carry);
  result
}

pub fn rrc(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate right
  rotate_right(flag_register, value, shift_operation_flag_update)
}

pub fn rrca(flag_register: &mut FlagRegister, value: u8) -> u8 { //rotate right
  rotate_right(flag_register, value, shift_operation_flag_update_without_z)
}

//difference between shift and rotate is, that we don't add the pushed out bit on the other side
pub fn sla(flag_register: &mut FlagRegister, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x80) == 0x80;
  let result = value << 1;
  shift_operation_flag_update(flag_register, result, new_carry);
  result
}

pub fn sra(flag_register: &mut FlagRegister, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | (value & 0x80);
  shift_operation_flag_update(flag_register, result, new_carry);
  result
}

pub fn srl(flag_register: &mut FlagRegister, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x01) == 0x01;
  let result = value >> 1;
  shift_operation_flag_update(flag_register, result, new_carry);
  result
}

pub fn ccf(flag_register: &mut FlagRegister) { //compliment carry flag
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, false);
  flag_register.set_flag(CpuFlag::C, !flag_register.get_flag(CpuFlag::C));
}

pub fn scf(flag_register: &mut FlagRegister) { //set carry flag
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, false);
  flag_register.set_flag(CpuFlag::C, true);
}

pub fn bit(flag_register: &mut FlagRegister, bit: u8, value: u8) { //check bit at
  flag_register.set_flag(CpuFlag::Z, (value & (1 << bit)) == 0 );
  flag_register.set_flag(CpuFlag::N, false);
  flag_register.set_flag(CpuFlag::H, true);
}