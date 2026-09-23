use super::registers::RegisterName8;
use super::Cpu;

#[derive(Debug, Clone, Copy)]
pub enum Operand8 {
    Register(RegisterName8),
    IndirectHl,
}

impl Operand8 {
    /// Decodes the low three bits in SM83 operand order: B, C, D, E, H, L, (HL), A.
    pub fn decode(bits: u8) -> Self {
        match bits & 7 {
            0 => Self::Register(RegisterName8::B),
            1 => Self::Register(RegisterName8::C),
            2 => Self::Register(RegisterName8::D),
            3 => Self::Register(RegisterName8::E),
            4 => Self::Register(RegisterName8::H),
            5 => Self::Register(RegisterName8::L),
            6 => Self::IndirectHl,
            _ => Self::Register(RegisterName8::A),
        }
    }

    pub fn read(self, cpu: &Cpu) -> u8 {
        match self {
            Self::Register(register) => cpu.registers.get(register),
            Self::IndirectHl => cpu.mmu.read_byte(cpu.registers.get_hl()),
        }
    }

    pub fn write(self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::Register(register) => cpu.registers.set(register, value),
            Self::IndirectHl => cpu.mmu.write_byte(cpu.registers.get_hl(), value),
        }
    }
}
