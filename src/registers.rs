#[derive(Debug, Copy, Clone)]
pub enum CpuFlag {
  Z = 0b10000000,
  N = 0b01000000,
  H = 0b00100000,
  C = 0b00010000
}

#[derive(Debug, Copy, Clone)]
pub struct Registers {
  pub a: u8,
  f: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub h: u8,
  pub l: u8,
  pub sp: u16,
  pub pc: u16,
}

impl Registers {
  pub fn new() -> Registers {
    Registers {
      a: 0x01,
      f: 0xB0,
      b: 0x00,
      c: 0x13,
      d: 0x00,
      e: 0xD8,
      h: 0x01,
      l: 0x4D,
      sp: 0xFFFE,
      pc: 0x0100
    }
  }

  pub fn get_af(&self) -> u16 {
    (self.a as u16) << 8 | self.f as u16
  }

  pub fn get_bc(&self) -> u16 {
    (self.b as u16) << 8 | self.c as u16
  }

  pub fn get_de(&self) -> u16 {
    (self.d as u16) << 8 | self.e as u16
  }

  pub fn get_hl(&self) -> u16 {
    (self.h as u16) << 8 | self.l as u16
  }

  pub fn get_hli(&mut self) -> u16 {
    let res = self.get_hl();
    self.set_hl(res + 1);
    res
  }

  pub fn get_hld(&mut self) -> u16 {
    let res = self.get_hl();
    self.set_hl(res - 1);
    res
  }

  pub fn set_af(&mut self, w: u16) {
    self.a = (w >> 8) as u8;
  }

  pub fn set_bc(&mut self, w: u16) {
    self.b = (w >> 8) as u8;
    self.c = w as u8;
  }

  pub fn set_de(&mut self, w: u16) {
    self.d = (w >> 8) as u8;
    self.e = w as u8;
  }

  pub fn set_hl(&mut self, w: u16) {
    self.h = (w >> 8) as u8;
    self.l = w as u8;
  }

  pub fn get_flag(&self, cpu_flag: CpuFlag) -> bool {
    (self.f & cpu_flag as u8) > 0
  }

  pub fn set_flag(&mut self, cpu_flag: CpuFlag, value: bool) {
    match value {
      true => self.f |= cpu_flag as u8,
      false => self.f &= !(cpu_flag as u8)
    }
    self.f &= 0xF0; // the lower bits are always 0
  }

  pub fn reset_flags(&mut self) {
    self.f = 0x00;
  }
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn wide_registers()
  {
    let mut test_registers = Registers::new();

    test_registers.a = 0x14;
    test_registers.b = 0x15;
    test_registers.c = 0x16;
    test_registers.d = 0x17;
    test_registers.e = 0x18;
    test_registers.h = 0x19;
    test_registers.l = 0x20;

    assert_eq!(test_registers.get_af(), 0x14B0);
    assert_eq!(test_registers.get_bc(), 0x1516);
    assert_eq!(test_registers.get_de(), 0x1718);
    assert_eq!(test_registers.get_hl(), 0x1920);

    test_registers.set_af(0x2200);
    test_registers.set_bc(0x3333);
    test_registers.set_de(0x4444);
    test_registers.set_hl(0x5555);

    assert_eq!(test_registers.get_af(), 0x22B0);
    assert_eq!(test_registers.get_bc(), 0x3333);
    assert_eq!(test_registers.get_de(), 0x4444);
    assert_eq!(test_registers.get_hl(), 0x5555);
  }

  #[test]
  fn test_hl_sepcial() {
    let mut test_registers = Registers::new();

    test_registers.set_hl(0x1234);
    assert_eq!(test_registers.get_hld(), 0x1234);
    assert_eq!(test_registers.get_hld(), 0x1233);
    assert_eq!(test_registers.get_hli(), 0x1232);
    assert_eq!(test_registers.get_hli(), 0x1233);
    assert_eq!(test_registers.get_hl(), 0x1234);
  }
}