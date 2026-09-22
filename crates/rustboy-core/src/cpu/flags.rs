#[derive(Debug, Copy, Clone)]
pub enum CpuFlag {
    Z = 0b1000_0000, // zero
    N = 0b0100_0000, // subtract
    H = 0b0010_0000, // half carry
    C = 0b0001_0000, // carry
}

/// The F register. Its private storage always has a zero low nibble.
#[derive(Debug, Copy, Clone)]
pub struct Flags {
    bits: u8,
}

impl Flags {
    /// Discards the unused low nibble, including when loading F through POP AF.
    pub fn from_bits(bits: u8) -> Self {
        Self { bits: bits & 0xF0 }
    }

    pub fn bits(&self) -> u8 {
        self.bits
    }

    pub fn get_flag(&self, flag: CpuFlag) -> bool {
        self.bits & flag as u8 != 0
    }

    pub fn set_flag(&mut self, flag: CpuFlag, value: bool) {
        if value {
            self.bits |= flag as u8;
        } else {
            self.bits &= !(flag as u8);
        }
    }

    pub fn reset_flags(&mut self) {
        self.bits = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{CpuFlag, Flags};

    #[test]
    fn construction_and_mutation_preserve_the_flag_register_invariant() {
        for raw in 0..=u8::MAX {
            let initial = Flags::from_bits(raw);
            assert_eq!(initial.bits(), raw & 0xF0);

            for flag in [CpuFlag::Z, CpuFlag::N, CpuFlag::H, CpuFlag::C] {
                for value in [false, true] {
                    let mut flags = initial;
                    flags.set_flag(flag, value);
                    assert_eq!(flags.get_flag(flag), value);
                    assert_eq!(flags.bits() & 0x0F, 0);
                    assert_eq!(flags.bits() & !(flag as u8), initial.bits() & !(flag as u8));
                    flags.reset_flags();
                    assert_eq!(flags.bits(), 0);
                }
            }
        }
    }
}
