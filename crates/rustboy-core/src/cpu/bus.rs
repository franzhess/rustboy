use crate::mmu::Mmu;

const M_CYCLE_T_CYCLES: usize = 4;

/// CPU-facing bus for one instruction.
///
/// An access is sampled at the start of its M-cycle, then devices advance through
/// that M-cycle. Opcode handlers fill remaining instruction time with internal
/// M-cycles.
pub(super) struct CpuBus<'a> {
    mmu: &'a mut Mmu,
    elapsed_cycles: usize,
}

impl<'a> CpuBus<'a> {
    pub(super) fn new(mmu: &'a mut Mmu) -> Self {
        Self {
            mmu,
            elapsed_cycles: 0,
        }
    }

    pub(super) fn read_byte(&mut self, address: u16) -> u8 {
        let value = self.mmu.read_cpu_byte(address);
        self.idle_mcycle();
        value
    }

    pub(super) fn write_byte(&mut self, address: u16, value: u8) {
        self.mmu.write_cpu_byte(address, value);
        self.idle_mcycle();
    }

    pub(super) fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, value as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
    }

    pub(super) fn peek_byte(&self, address: u16) -> u8 {
        self.mmu.read_byte(address)
    }

    pub(super) fn finish(&mut self, total_cycles: usize) {
        assert!(
            self.elapsed_cycles <= total_cycles,
            "CPU scheduled {} T-cycles for a {total_cycles}-T-cycle instruction",
            self.elapsed_cycles
        );
        while self.elapsed_cycles < total_cycles {
            self.idle_mcycle();
        }
    }

    fn idle_mcycle(&mut self) {
        self.advance(M_CYCLE_T_CYCLES);
    }

    fn advance(&mut self, cycles: usize) {
        self.mmu.do_ticks(cycles);
        self.elapsed_cycles += cycles;
    }
}
