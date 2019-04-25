use crate::cpu::registers::*;

type Operation8 = fn(&mut Registers, u8) -> u8;
type Operation16 = fn(&mut Registers, u16) -> u16;

fn execute8(registers: &mut Registers, name: RegisterName8, op: Operation8) {
  let value = op(registers, registers.get(name));
  registers.set(name, value);
}

fn execute16(registers: &mut Registers, name: RegisterName16, op: Operation16) {
  let value = op(registers, registers.get16(name));
  registers.set16(name, value);
}

pub fn and(registers: &mut Registers, name: RegisterName8) {
  andv(registers, registers.get(name))
}

pub fn andv(registers: &mut Registers, value: u8) {
  registers.a &= value;

  registers.reset_flags();
  registers.set_flag(CpuFlag::Z, registers.a == 0x00);
  registers.set_flag(CpuFlag::H, true);
}

pub fn or(registers: &mut Registers, value: u8) {
  registers.a |= value;

  registers.reset_flags();
  registers.set_flag(CpuFlag::Z, registers.a == 0);
}

pub fn xor(registers: &mut Registers, value: u8) {
  registers.a ^= value;

  registers.reset_flags();
  registers.set_flag(CpuFlag::Z, registers.a == 0);
}

pub fn cpl(registers: &mut Registers) {
  registers.a ^= 0xFF;

  registers.set_flag(CpuFlag::N, true);
  registers.set_flag(CpuFlag::H, true);
}

pub fn inc(registers: &mut Registers, name: RegisterName8) {
  execute8(registers, name, incv);
}

pub fn incv(registers: &mut Registers, value: u8) -> u8 {
  let result = value.wrapping_add(1);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, (value & 0x0F) + 1 > 0x0F); //a half carry occurs when the low nibble + 1 is greater than 0x0F
  result
}

pub fn dec(registers: &mut Registers, name: RegisterName8) {
  execute8(registers, name, decv);
}

pub fn decv(registers: &mut Registers, value: u8) -> u8 {
  let result = value.wrapping_sub(1);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, true);
  registers.set_flag(CpuFlag::H, (value & 0x0F) == 0); //a half carry will occur when the low nibble is all zeros
  result
}

pub fn add(registers: &mut Registers, name: RegisterName8) {
  addv(registers, registers.get(name))
}

pub fn addv(registers: &mut Registers, value: u8) {
  let result = registers.a.wrapping_add(value);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, (((registers.a & 0x0F) + (value & 0x0F)) & 0x10) == 0x10);
  registers.set_flag(CpuFlag::C, registers.a as usize + value as usize > 0xFF);
  registers.a = result;
}

pub fn add16(registers: &mut Registers, value: u16) {
  let result = registers.get_hl().wrapping_add(value);
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, (((registers.get_hl() & 0x00FF) + (value & 0x00FF)) & 0x0100) == 0x0100);
  registers.set_flag(CpuFlag::C, registers.get_hl() as usize + value as usize > 0xFFFF);
  registers.set_hl(result); //16bit add goes to hl
}

pub fn adc(registers: &mut Registers, value: u8) { //like add + carry flag
  let c: u8 = if registers.get_flag(CpuFlag::C) { 1 } else { 0 };
  let result = registers.a.wrapping_add(value).wrapping_add(c);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, (((registers.a & 0x0F) + (value & 0x0F) + c) & 0x10) == 0x10);
  registers.set_flag(CpuFlag::C, registers.a as u16 + value as u16 + c as u16 > 0xFF);

}

pub fn sub(registers: &mut Registers, value: u8) {
  let result = registers.a.wrapping_sub(value);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, true);
  registers.set_flag(CpuFlag::H, (registers.a & 0x0F) < (value & 0x0F));
  registers.set_flag(CpuFlag::C, registers.a < value);
  registers.a = result;
}

pub fn sbc(registers: &mut Registers, value: u8) {
  let c: u8 = if registers.get_flag(CpuFlag::C) { 1 } else { 0 };
  let result = registers.a.wrapping_sub(value).wrapping_sub(c);
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::N, true);
  registers.set_flag(CpuFlag::H, (registers.a & 0x0F) < (value & 0x0F) + c);
  registers.set_flag(CpuFlag::C, registers.a < value + c);
  registers.a = result;
}

pub fn cp(registers: &mut Registers, value: u8) {
  let temp = registers.a;
  sub(registers, value);
  registers.a = temp;
}

pub fn swap(registers: &mut Registers, value: u8) -> u8 {
  registers.reset_flags();
  registers.set_flag(CpuFlag::Z, value == 0);
  (value << 4) | (value >> 4)
}

pub fn add_next_signed_byte_to_word(registers: &mut Registers, value1: u16, value2: u16) -> u16 {
  registers.reset_flags();

  registers.set_flag(CpuFlag::H, (value1 & 0x000F) + (value2 & 0x000F) > 0x000F);
  registers.set_flag(CpuFlag::C, (value1 & 0x00FF) + (value2 & 0x00FF) > 0x00FF);

  value1.wrapping_add(value2)
}

pub fn daa(registers: &mut Registers) {  //i got no idea what i'm doing
  let mut adjust = if registers.get_flag(CpuFlag::C) { 0x60 } else { 0x00 };
  if registers.get_flag(CpuFlag::H) { adjust |= 0x06; }
  if !registers.get_flag(CpuFlag::N) {
    if registers.a & 0x0F > 0x09 { adjust |= 0x06; };
    if registers.a > 0x99 { adjust |= 0x60; };
    registers.a = registers.a.wrapping_add(adjust);
  } else {
    registers.a = registers.a.wrapping_sub(adjust);
  }

  registers.set_flag(CpuFlag::Z, registers.a == 0);
  registers.set_flag(CpuFlag::H, false);
  registers.set_flag(CpuFlag::C, adjust >= 0x60);
}

pub fn shift_operation_flag_update(registers: &mut Registers, result: u8, new_carry: bool) {
  registers.reset_flags();
  registers.set_flag(CpuFlag::Z, result == 0);
  registers.set_flag(CpuFlag::C, new_carry);
}

pub fn rl(registers: &mut Registers, value: u8) -> u8 { //rotate left through carry
  let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
  let result = (value << 1) | if registers.get_flag(CpuFlag::C) { 0x01 } else { 0x00 }; //push one to the right and add the carry to the right
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn rlc(registers: &mut Registers, name: RegisterName8) {
  execute8(registers, name, rlcv);
}

pub fn rlcv(registers: &mut Registers, value: u8) -> u8 { //rotate left
  let new_carry = (value & 0x80) == 0x80; //left most bit that gets pushed out
  let result = (value << 1) | if new_carry { 0x01 } else { 0x00 }; //push one to the left and add the pushed out bit to the right
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn rr(registers: &mut Registers, value: u8) -> u8 { //rotate right through carry
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | if registers.get_flag(CpuFlag::C) { 0x80 } else { 0x00 };
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn rrc(registers: &mut Registers, value: u8) -> u8 { //rotate right
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | if new_carry { 0x80 } else { 0x00 };
  shift_operation_flag_update(registers, result, new_carry);
  result
}

//difference between shift and rotate is, that we don't add the pushed out bit on the other side
pub fn sla(registers: &mut Registers, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x80) == 0x80;
  let result = value << 1;
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn sra(registers: &mut Registers, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x01) == 0x01;
  let result = (value >> 1) | (value & 0x80);
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn srl(registers: &mut Registers, value: u8) -> u8 { //shift left arithmetic (b0=0)
  let new_carry = (value & 0x01) == 0x01;
  let result = value >> 1;
  shift_operation_flag_update(registers, result, new_carry);
  result
}

pub fn ccf(registers: &mut Registers) { //compliment carry flag
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, false);
  registers.set_flag(CpuFlag::C, registers.get_flag(CpuFlag::C));
}

pub fn scf(registers: &mut Registers) { //set carry flag
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, false);
  registers.set_flag(CpuFlag::C, true);
}

pub fn bit(registers: &mut Registers, bit: u8, value: u8) { //check bit at
  registers.set_flag(CpuFlag::Z, (value & (1 << bit)) == 0 );
  registers.set_flag(CpuFlag::N, false);
  registers.set_flag(CpuFlag::H, true);
}