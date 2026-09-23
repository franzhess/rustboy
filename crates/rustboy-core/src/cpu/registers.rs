use super::flags::Flags;

#[derive(Debug, Copy, Clone)]
pub enum RegisterName8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterName16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, Copy, Clone)]
pub struct Registers {
    pub a: u8,
    pub flags: Flags,
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
            flags: Flags::from_bits(0xB0),
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    pub fn get(&self, name: RegisterName8) -> u8 {
        match name {
            RegisterName8::A => self.a,
            RegisterName8::B => self.b,
            RegisterName8::C => self.c,
            RegisterName8::D => self.d,
            RegisterName8::E => self.e,
            RegisterName8::H => self.h,
            RegisterName8::L => self.l,
        }
    }

    pub fn get16(&self, name: RegisterName16) -> u16 {
        match name {
            RegisterName16::AF => self.get_af(),
            RegisterName16::BC => self.get_bc(),
            RegisterName16::DE => self.get_de(),
            RegisterName16::HL => self.get_hl(),
            RegisterName16::SP => self.sp,
        }
    }

    pub fn set16(&mut self, name: RegisterName16, value: u16) {
        match name {
            RegisterName16::AF => self.set_af(value),
            RegisterName16::BC => self.set_bc(value),
            RegisterName16::DE => self.set_de(value),
            RegisterName16::HL => self.set_hl(value),
            RegisterName16::SP => self.sp = value,
        }
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.flags.bits() as u16
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

    /// Increments HL with 16-bit wraparound and returns its original value.
    pub fn post_increment_hl(&mut self) -> u16 {
        let hl = self.get_hl();
        self.set_hl(hl.wrapping_add(1));
        hl
    }

    /// Decrements HL with 16-bit wraparound and returns its original value.
    pub fn post_decrement_hl(&mut self) -> u16 {
        let hl = self.get_hl();
        self.set_hl(hl.wrapping_sub(1));
        hl
    }

    pub fn set(&mut self, name: RegisterName8, value: u8) {
        match name {
            RegisterName8::A => self.a = value,
            RegisterName8::B => self.b = value,
            RegisterName8::C => self.c = value,
            RegisterName8::D => self.d = value,
            RegisterName8::E => self.e = value,
            RegisterName8::H => self.h = value,
            RegisterName8::L => self.l = value,
        }
    }

    pub fn set_af(&mut self, w: u16) {
        self.a = (w >> 8) as u8;
        self.flags = Flags::from_bits(w as u8);
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loading_af_masks_unused_flag_bits_and_preserves_the_accumulator() {
        let mut registers = Registers::new();
        for raw_flags in 0..=u8::MAX {
            registers.set_af(0xA500 | u16::from(raw_flags));
            assert_eq!(registers.a, 0xA5);
            assert_eq!(registers.get_af(), 0xA500 | u16::from(raw_flags & 0xF0));
        }
    }

    #[test]
    fn wide_registers() {
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

        assert_eq!(test_registers.get_af(), 0x2200);
        assert_eq!(test_registers.get_bc(), 0x3333);
        assert_eq!(test_registers.get_de(), 0x4444);
        assert_eq!(test_registers.get_hl(), 0x5555);
    }

    #[test]
    fn dmg_initial_registers_match_the_post_boot_state() {
        let registers = Registers::new();

        assert_eq!(registers.get_af(), 0x01B0);
        assert_eq!(registers.get_bc(), 0x0013);
        assert_eq!(registers.get_de(), 0x00D8);
        assert_eq!(registers.get_hl(), 0x014D);
        assert_eq!(registers.sp, 0xFFFE);
        assert_eq!(registers.pc, 0x0100);
    }

    #[test]
    fn hl_post_increment_and_decrement_return_the_original_address() {
        let mut test_registers = Registers::new();

        test_registers.set_hl(0x1234);
        assert_eq!(test_registers.post_decrement_hl(), 0x1234);
        assert_eq!(test_registers.post_decrement_hl(), 0x1233);
        assert_eq!(test_registers.post_increment_hl(), 0x1232);
        assert_eq!(test_registers.post_increment_hl(), 0x1233);
        assert_eq!(test_registers.get_hl(), 0x1234);
    }
}
