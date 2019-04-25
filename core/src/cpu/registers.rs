#[derive(Debug, Copy, Clone)]
pub enum CpuFlag {
  Z = 0b10000000, //zero
  N = 0b01000000, //subtract
  H = 0b00100000, //half carry
  C = 0b00010000 //carry
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterName8 {
  A,
  B,
  C,
  D,
  E,
  H,
  L
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterName16 {
  AF,
  BC,
  DE,
  HL,
  SP,
  PC,
  HLI,
  HLD
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
      sp: 0xFFFF, //first stack address is 0xFFFE
      pc: 0x0100
    }
  }

  pub fn get(&self, name: RegisterName8) -> u8 {
    match name {
      RegisterName8::A => { self.a },
      RegisterName8::B => { self.b },
      RegisterName8::C => { self.c },
      RegisterName8::D => { self.d },
      RegisterName8::E => { self.e },
      RegisterName8::H => { self.h },
      RegisterName8::L => { self.l }
    }
  }

  pub fn get16(&mut self, name: RegisterName16) -> u16 {
    match name {
      RegisterName16::AF => { self.get_af() },
      RegisterName16::BC => { self.get_bc() },
      RegisterName16::DE => { self.get_de() },
      RegisterName16::HL => { self.get_hl() },
      RegisterName16::HLI => { self.get_hli() },
      RegisterName16::HLD => { self.get_hld() },
      RegisterName16::SP => { self.sp },
      RegisterName16::PC => { self.pc }
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
    let hl = self.get_hl();
    self.set_hl(hl.wrapping_add(1));
    hl
  }

  pub fn get_hld(&mut self) -> u16 {
    let hl = self.get_hl();
    self.set_hl(hl.wrapping_sub(1));
    hl
  }

  pub fn set(&mut self, name: RegisterName8, value: u8) {
    match name {
      RegisterName8::A => { self.a = value },
      RegisterName8::B => { self.b = value },
      RegisterName8::C => { self.c = value },
      RegisterName8::D => { self.d = value },
      RegisterName8::E => { self.e = value },
      RegisterName8::H => { self.h = value },
      RegisterName8::L => { self.l = value },
    }
  }

  pub fn set16(&mut self, name: RegisterName16, value: u16) {
    match name {
      RegisterName16::AF => { self.set_af(value); },
      RegisterName16::BC => { self.set_bc(value); },
      RegisterName16::DE => { self.set_de(value); },
      RegisterName16::HL => { self.set_hl(value); },
      RegisterName16::SP => { self.sp = value; },
      RegisterName16::PC => { self.pc = value; },
      _ => { panic!("Cannot set hli/hld"); }
    }
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

  //if you have to clear more than one flag, this way is more efficient
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