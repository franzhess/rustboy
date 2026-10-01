mod alu;
mod bus;
mod flags;
mod opcodes;
mod operand;
mod registers;
mod state;

pub use state::{CpuDiagnostic, CpuState};

#[cfg(test)]
mod timing_tests;

#[cfg(test)]
mod cb_tests;

#[cfg(test)]
mod base_tests;

use crate::cpu::flags::Flags;
use crate::cpu::registers::Registers;
use crate::mmu::Mmu;
use bus::CpuBus;

pub enum OpcodeResult {
    Executed(usize),
    UnknownOpcode,
}

pub struct CpuStepResult {
    pub cycles: usize,
    pub opcode: Option<u8>,
    pub diagnostic: Option<CpuDiagnostic>,
}

type UnaryOperation8 = fn(&mut Flags, u8) -> u8;
type BinaryOperation8 = fn(&mut Flags, u8, u8) -> u8;

pub struct Cpu {
    registers: Registers,
    state: CpuState,
    halt_bug: bool,
    ime: bool,        // Interrupt master enable, also modified by RETI and interrupt entry.
    ei_requested: u8, // Instruction completions remaining before EI takes effect.
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterValues {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            state: CpuState::Running,
            halt_bug: false,
            ime: false,
            ei_requested: 0,
        }
    }

    /// Executes one CPU step. Instructions advance devices through their bus and
    /// internal M-cycles; interrupt entry and idle states retain their existing
    /// instruction-sized timing until their phase ordering is modeled separately.
    pub fn tick(&mut self, mmu: &mut Mmu) -> CpuStepResult {
        if self.handle_irq(mmu) {
            // Interrupt entry is a five-M-cycle hardware sequence: acknowledge IF, push the
            // current PC, and load the vector. Devices continue running for all 20 T-cycles.
            mmu.do_ticks(20);
            CpuStepResult {
                cycles: 20,
                opcode: None,
                diagnostic: None,
            }
        } else if self.state == CpuState::Running {
            let mut bus = CpuBus::new(mmu);
            let result = self.do_cycle(&mut bus);
            bus.finish(result.cycles);
            // EI itself consumes the first completion; the following instruction consumes
            // the second. Interrupt entry and idle steps are not instruction completions.
            if self.ei_requested != 0 {
                self.ei_requested -= 1;
                if self.ei_requested == 0 {
                    self.ime = true;
                }
            }
            result
        } else {
            mmu.do_ticks(4);
            CpuStepResult {
                cycles: 4,
                opcode: None,
                diagnostic: None,
            }
        }
    }

    pub fn next_opcode(&self, mmu: &Mmu) -> Option<u8> {
        (self.state == CpuState::Running).then(|| mmu.read_byte(self.registers.pc))
    }

    pub fn state(&self) -> CpuState {
        self.state
    }

    pub fn registers(&self) -> RegisterValues {
        RegisterValues {
            a: self.registers.a,
            b: self.registers.b,
            c: self.registers.c,
            d: self.registers.d,
            e: self.registers.e,
            f: self.registers.get_af() as u8,
            h: self.registers.h,
            l: self.registers.l,
        }
    }

    fn handle_irq(&mut self, mmu: &mut Mmu) -> bool {
        let irq_requested = mmu.read_byte(0xFF0F);
        let irq = self.pending_interrupts(mmu);
        if irq == 0 {
            return false;
        }

        // Preserve the existing wake behavior for every idle state, including the
        // STOP and illegal-opcode approximations. Hardware-specific differences are
        // separate work. Fetch suppression for the HALT bug is handled separately.
        self.state = CpuState::Running;
        if !self.ime {
            return false;
        }

        self.ime = false;
        self.ei_requested = 0;
        let irq_num = irq.trailing_zeros(); //0 vblank, 1 stat, 2 timer, 3 serial, 4 joypad

        if self.halt_bug {
            // EI; HALT with an already pending interrupt returns to HALT. The suppressed
            // increment belongs to interrupt entry, not the handler's first opcode fetch.
            self.registers.pc = self.registers.pc.wrapping_sub(1);
            self.halt_bug = false;
        }
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        mmu.write_word(self.registers.sp, self.registers.pc);
        self.registers.pc = (0x0040 + 8 * irq_num) as u16;
        mmu.write_byte(0xFF0F, irq_requested & !(1 << irq_num));
        true
    }

    fn do_cycle(&mut self, bus: &mut CpuBus<'_>) -> CpuStepResult {
        let current_address = self.registers.pc;
        let opcode = self.fetch_byte(bus);

        let (cycles, diagnostic) = match opcodes::execute(opcode, self, bus) {
            OpcodeResult::Executed(ticks) => (ticks, None),
            OpcodeResult::UnknownOpcode => {
                self.state = CpuState::IllegalOpcode;
                (
                    4,
                    Some(CpuDiagnostic::IllegalOpcode {
                        address: current_address,
                        opcode,
                    }),
                )
            }
        };
        CpuStepResult {
            cycles,
            opcode: Some(opcode),
            diagnostic,
        }
    }

    fn fetch_byte(&mut self, bus: &mut CpuBus<'_>) -> u8 {
        let res = bus.read_byte(self.registers.pc);
        if self.halt_bug {
            // HALT with IME clear and an enabled pending interrupt reuses the opcode byte as
            // the next instruction's first byte. The suppression applies to one fetch only.
            self.halt_bug = false;
        } else {
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
        res
    }

    fn fetch_word(&mut self, bus: &mut CpuBus<'_>) -> u16 {
        u16::from(self.fetch_byte(bus)) | (u16::from(self.fetch_byte(bus)) << 8)
    }

    fn push(&mut self, value: u16, bus: &mut CpuBus<'_>) {
        bus.internal_mcycle();
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_byte(self.registers.sp, (value >> 8) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_byte(self.registers.sp, value as u8);
    }

    fn pop(&mut self, bus: &mut CpuBus<'_>) -> u16 {
        let low = bus.read_byte(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = bus.read_byte(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        u16::from(low) | (u16::from(high) << 8)
    }

    fn call(&mut self, address: u16, bus: &mut CpuBus<'_>) {
        self.push(self.registers.pc, bus); // PC already follows the immediate operand.
        self.registers.pc = address;
    }

    fn return_from_call(&mut self, bus: &mut CpuBus<'_>) {
        self.registers.pc = self.pop(bus);
    }

    fn jump_r(&mut self, bus: &mut CpuBus<'_>) {
        let offset = self.fetch_byte(bus);
        self.registers.pc = self.registers.pc.wrapping_add(offset as i8 as i16 as u16);
    }

    fn pending_interrupts(&self, mmu: &Mmu) -> u8 {
        mmu.read_byte(0xFFFF) & mmu.read_byte(0xFF0F) & 0x1F
    }

    fn pending_interrupts_on_bus(&self, bus: &CpuBus<'_>) -> u8 {
        bus.peek_byte(0xFFFF) & bus.peek_byte(0xFF0F) & 0x1F
    }
}

#[cfg(test)]
mod tests {
    use super::{opcodes, Cpu, CpuBus, CpuDiagnostic, CpuState};
    use crate::cpu::flags::CpuFlag;
    use crate::mbc::Mbc;
    use crate::mmu::Mmu;
    use crate::Machine;

    struct TestMbc(Vec<u8>);

    impl Mbc for TestMbc {
        fn read_rom(&self, address: u16) -> u8 {
            self.0.get(address as usize).copied().unwrap_or(0)
        }

        fn read_ram(&self, _address: u16) -> u8 {
            0
        }

        fn write_rom(&mut self, _address: u16, _value: u8) {}

        fn write_ram(&mut self, _address: u16, _value: u8) {}
    }

    fn machine_with_rom(rom: Vec<u8>) -> Machine {
        Machine {
            cpu: Cpu::new(),
            mmu: Mmu::new(Box::new(TestMbc(rom))),
        }
    }

    pub(super) fn machine_with_program(program: &[u8]) -> Machine {
        let mut rom = vec![0; 0x100 + program.len()];
        rom[0x100..].copy_from_slice(program);
        rom[0x40] = 0x04; // VBlank handler: INC B; RETI
        rom[0x41] = 0xD9;
        machine_with_rom(rom)
    }

    #[test]
    fn cpu_execution_advances_devices_one_mcycle_per_nop() {
        let mut machine = machine_with_program(&[0, 0, 0, 0]);
        machine.mmu.write_byte(0xFF07, 0x05);
        for _ in 0..3 {
            let result = machine.cpu.tick(&mut machine.mmu);
            assert_eq!((result.cycles, result.opcode), (4, Some(0)));
            assert_eq!(machine.mmu.read_byte(0xFF05), 0);
        }
        let result = machine.cpu.tick(&mut machine.mmu);
        assert_eq!((result.cycles, result.opcode), (4, Some(0)));
        assert_eq!(machine.cpu.registers.pc, 0x104);
        assert_eq!(machine.mmu.read_byte(0xFF05), 1);
    }

    #[test]
    fn pop_reads_stack_bytes_on_separate_mcycles() {
        let mut machine = machine_with_program(&[0xC1]); // POP BC
        machine.cpu.registers.sp = 0xFF03; // Unused IO byte, then DIV.
        machine.mmu.do_ticks(248);
        assert_eq!(machine.mmu.read_byte(0xFF04), 0);

        assert_step(&mut machine, 12, Some(0xC1));

        // Opcode fetch reaches cycle 252. The low-byte read then advances DIV
        // through 256, so the following high-byte read observes one.
        assert_eq!(machine.cpu.registers.get_bc(), 0x01FF);
        assert_eq!(machine.cpu.registers.sp, 0xFF05);
    }

    #[test]
    fn push_writes_its_low_byte_during_the_final_mcycle() {
        let mut machine = machine_with_program(&[0xC5, 0x00]); // PUSH BC; NOP
        machine.cpu.registers.set_bc(0x12C0);
        machine.cpu.registers.sp = 0xFF48;
        machine.mmu.write_byte(0xC000, 0x42);
        machine.mmu.write_byte(0xFE00, 0x11);

        assert_step(&mut machine, 16, Some(0xC5));

        assert_eq!(machine.cpu.registers.sp, 0xFF46);
        assert_eq!(machine.mmu.read_byte(0xFF46), 0xC0);
        assert_eq!(machine.mmu.read_cpu_byte(0xFE00), 0x11);

        assert_step(&mut machine, 4, Some(0x00));
        assert_eq!(machine.mmu.read_cpu_byte(0xFE00), 0xFF);
    }

    #[test]
    fn taken_conditional_return_delays_before_reading_the_stack() {
        let mut machine = machine_with_program(&[0xC0]); // RET NZ
        machine.cpu.registers.flags.set_flag(CpuFlag::Z, false);
        machine.cpu.registers.sp = 0xFF04; // DIV low byte, then TIMA high byte.
        machine.mmu.write_byte(0xFF05, 0xC0);
        machine.mmu.do_ticks(248);

        assert_step(&mut machine, 20, Some(0xC0));

        // Fetch reaches cycle 252 and the leading internal cycle reaches 256,
        // so the low-byte DIV read observes one.
        assert_eq!(machine.cpu.registers.pc, 0xC001);
        assert_eq!(machine.cpu.registers.sp, 0xFF06);
    }

    fn assert_step(machine: &mut Machine, cycles: usize, opcode: Option<u8>) {
        let result = machine.step();
        assert_eq!(result.cycles, cycles);
        assert_eq!(result.opcode, opcode);
        assert_eq!(result.diagnostic, None);
    }

    #[test]
    fn illegal_opcodes_report_once_and_idle_without_fetching_another_instruction() {
        for opcode in [
            0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
        ] {
            let mut machine = machine_with_program(&[opcode, 0x04]);
            let result = machine.step();
            assert_eq!((result.cycles, result.opcode), (4, Some(opcode)));
            assert_eq!(
                result.diagnostic,
                Some(CpuDiagnostic::IllegalOpcode {
                    address: 0x100,
                    opcode
                })
            );
            assert_eq!(machine.cpu.state(), CpuState::IllegalOpcode);
            assert_eq!(machine.next_opcode(), None);
            let registers = machine.cpu.registers();
            for _ in 0..3 {
                assert_step(&mut machine, 4, None);
                assert_eq!(machine.cpu.state(), CpuState::IllegalOpcode);
                assert_eq!(machine.cpu.registers.pc, 0x101);
                assert_eq!(machine.cpu.registers(), registers);
            }
        }
    }

    #[test]
    fn illegal_opcode_diagnostic_keeps_the_fetch_address_when_pc_wraps() {
        let mut machine = machine_with_program(&[]);
        machine.cpu.registers.pc = 0xFFFF;
        machine.mmu.write_byte(0xFFFF, 0xD3);
        let result = machine.step();
        assert_eq!(machine.cpu.registers.pc, 0);
        assert_eq!(
            result.diagnostic,
            Some(CpuDiagnostic::IllegalOpcode {
                address: 0xFFFF,
                opcode: 0xD3
            })
        );
    }

    #[test]
    fn distinct_idle_states_preserve_device_ticks_and_existing_interrupt_wake_behavior() {
        // STOP and illegal instructions currently wake like HALT. This is a
        // compatibility regression, not a claim about their hardware behavior.
        for (program, expected_state, resume_pc) in [
            ([0x76, 0x04, 0], CpuState::Halted, 0x101),
            ([0x10, 0, 0x04], CpuState::Stopped, 0x102),
            ([0xD3, 0x04, 0], CpuState::IllegalOpcode, 0x101),
        ] {
            for ime in [false, true] {
                let mut machine = machine_with_program(&program);
                machine.cpu.ime = ime;
                machine.mmu.write_byte(0xFF07, 0x05);
                let entry = machine.step();
                assert_eq!(entry.opcode, Some(program[0]));
                assert_eq!(machine.cpu.state(), expected_state);
                assert_eq!(machine.cpu.registers.pc, resume_pc);
                assert_eq!(machine.next_opcode(), None);
                for _ in 0..3 {
                    assert_step(&mut machine, 4, None);
                }
                assert_eq!(machine.mmu.read_byte(0xFF05), 1); // Sixteen T-cycles reached the timer.
                let previous_b = machine.cpu.registers.b;
                machine.mmu.write_byte(0xFFFF, 1);
                machine.mmu.write_byte(0xFF0F, 1);

                if ime {
                    assert_step(&mut machine, 20, None);
                    assert_eq!(machine.cpu.registers.pc, 0x40);
                    assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), resume_pc);
                    assert_eq!(machine.cpu.registers.b, previous_b);
                    assert_eq!(machine.mmu.read_byte(0xFF0F) & 1, 0);
                } else {
                    assert_step(&mut machine, 4, Some(0x04));
                    assert_eq!(machine.cpu.registers.pc, resume_pc + 1);
                    assert_eq!(machine.cpu.registers.b, previous_b.wrapping_add(1));
                    assert_eq!(machine.mmu.read_byte(0xFF0F) & 1, 1);
                }
                assert_eq!(machine.cpu.state(), CpuState::Running);
                assert!(machine.next_opcode().is_some());
            }
        }
    }

    #[test]
    fn absolute_jump_consumes_sixteen_cycles_and_advances_the_timer() {
        let mut machine = machine_with_program(&[0xC3, 0x34, 0x12]);
        machine.mmu.write_byte(0xFF04, 0);
        machine.mmu.write_byte(0xFF07, 0x05);

        assert_step(&mut machine, 16, Some(0xC3));
        assert_eq!(machine.cpu.registers.pc, 0x1234);
        assert_eq!(machine.mmu.read_byte(0xFF05), 1);
    }

    #[test]
    fn add_indirect_hl_consumes_eight_cycles_per_memory_operand() {
        let mut machine = machine_with_program(&[0x86, 0x86]);
        machine.cpu.registers.a = 1;
        machine.cpu.registers.set_hl(0xC000);
        machine.mmu.write_byte(0xC000, 2);
        machine.mmu.write_byte(0xFF04, 0);
        machine.mmu.write_byte(0xFF07, 0x05);

        assert_step(&mut machine, 8, Some(0x86));
        assert_eq!(machine.cpu.registers.a, 3);
        assert_eq!(machine.mmu.read_byte(0xFF05), 0);
        assert_step(&mut machine, 8, Some(0x86));
        assert_eq!(machine.cpu.registers.a, 5);
        assert_eq!(machine.cpu.registers.pc, 0x102);
        assert_eq!(machine.mmu.read_byte(0xFF05), 1);
    }

    #[test]
    fn instruction_fetch_wraps_the_program_counter() {
        let mut machine = machine_with_rom(Vec::new());
        machine.cpu.registers.pc = 0xFFFF;

        let mut bus = CpuBus::new(&mut machine.mmu);
        machine.cpu.fetch_byte(&mut bus);
        assert_eq!(machine.cpu.registers.pc, 0);

        machine.cpu.registers.pc = 0xFFFE;
        machine.cpu.fetch_word(&mut bus);
        assert_eq!(machine.cpu.registers.pc, 0);
    }

    #[test]
    fn signed_sp_instructions_extend_offsets_and_write_the_correct_destination() {
        for (sp, immediate, expected, flags) in [
            (0x0000, 0xFF, 0xFFFF, 0x00), // -1 wraps without low-byte carries.
            (0x0001, 0xFF, 0x0000, 0x30),
            (0x0080, 0x80, 0x0000, 0x10), // -128, with byte carry but no half-carry.
            (0xFFFF, 0x01, 0x0000, 0x30),
            (0x000F, 0x01, 0x0010, 0x20),
            (0xFF81, 0x7F, 0x0000, 0x30), // +127 wraps at the top of memory.
        ] {
            for (opcode, cycles) in [(0xE8, 16), (0xF8, 12)] {
                let mut machine = machine_with_program(&[opcode, immediate]);
                machine.cpu.registers.sp = sp;
                machine.cpu.registers.set_hl(0x1234);
                machine.cpu.registers.set_af(0x5AF0);

                assert_step(&mut machine, cycles, Some(opcode));

                let (expected_sp, expected_hl) = if opcode == 0xE8 {
                    (expected, 0x1234)
                } else {
                    (sp, expected)
                };
                assert_eq!(
                    (
                        machine.cpu.registers.sp,
                        machine.cpu.registers.get_hl(),
                        machine.cpu.registers.get_af(),
                        machine.cpu.registers.pc
                    ),
                    (expected_sp, expected_hl, 0x5A00 | flags, 0x102),
                    "opcode={opcode:02X}, SP={sp:04X}, immediate={immediate:02X}"
                );
            }
        }
    }

    #[test]
    fn not_taken_conditional_jump_wraps_while_skipping_its_operand() {
        let mut machine = machine_with_rom(Vec::new());
        machine.cpu.registers.pc = 0xFFFF;
        machine.cpu.registers.flags.set_flag(CpuFlag::Z, true);

        // PC already points to JR NZ's one-byte operand after its opcode was fetched.
        let mut bus = CpuBus::new(&mut machine.mmu);
        opcodes::execute(0x20, &mut machine.cpu, &mut bus);

        assert_eq!(machine.cpu.registers.pc, 0);
    }

    #[test]
    fn highest_priority_interrupt_consumes_twenty_cycles_and_ticks_devices() {
        let mut machine = machine_with_rom(Vec::new());
        machine.cpu.registers.pc = 0x1234;
        machine.cpu.registers.sp = 0xFFFE;
        machine.cpu.ime = true;
        machine.mmu.write_byte(0xFFFF, 0b0000_0101);
        machine.mmu.write_byte(0xFF0F, 0b0000_0101);
        machine.mmu.write_byte(0xFF07, 0x05);

        let result = machine.step();
        assert_eq!(result.cycles, 20);
        assert_eq!(result.opcode, None);
        assert!(!machine.cpu.ime);

        // VBlank (bit 0) wins over the simultaneously pending timer interrupt (bit 2).
        assert_eq!(machine.cpu.registers.pc, 0x0040);
        assert_eq!(machine.cpu.registers.sp, 0xFFFC);
        assert_eq!(machine.mmu.read_word(0xFFFC), 0x1234);
        assert_eq!(machine.mmu.read_byte(0xFF0F), 0b1110_0100);

        // Twenty T-cycles include the timer's first bit-3 falling edge at cycle 16.
        assert_eq!(machine.mmu.read_byte(0xFF05), 1);

        // The handler's NOP runs on the next step, without servicing the pending timer IRQ.
        let result = machine.step();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x00));
        assert_eq!(machine.cpu.registers.pc, 0x0041);
        assert_eq!(machine.cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn halt_idle_reports_no_opcode_but_waking_execution_does() {
        let mut machine = machine_with_rom(Vec::new());
        machine.cpu.registers.pc = 0xC000;
        machine.mmu.write_byte(0xC000, 0x76); // HALT
        machine.mmu.write_byte(0xC001, 0x04); // INC B

        assert_eq!(machine.step().opcode, Some(0x76));
        let result = machine.step();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, None);
        assert_eq!(machine.cpu.registers.pc, 0xC001);

        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);
        let result = machine.step();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x04));
        assert_eq!(machine.cpu.registers.pc, 0xC002);
    }

    #[test]
    fn ei_enables_interrupts_after_exactly_one_following_instruction() {
        let mut machine = machine_with_program(&[0xFB, 0x00, 0x00]); // EI; NOP; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0xFB));
        assert!(!machine.cpu.ime);
        assert_step(&mut machine, 4, Some(0x00));
        assert!(machine.cpu.ime);

        assert_step(&mut machine, 20, None);
        assert_eq!(machine.cpu.registers.pc, 0x0040);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0102);
    }

    #[test]
    fn repeated_ei_does_not_delay_an_already_scheduled_enable() {
        let mut machine = machine_with_program(&[0xFB, 0xFB, 0x00]); // EI; EI; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0xFB));
        assert_step(&mut machine, 4, Some(0xFB));
        assert_step(&mut machine, 20, None);
        assert_eq!(machine.cpu.registers.pc, 0x0040);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0102);
    }

    #[test]
    fn di_cancels_a_pending_ei_enable() {
        let mut machine = machine_with_program(&[0xFB, 0xF3, 0x00, 0x00]); // EI; DI; NOP; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0xFB));
        assert_step(&mut machine, 4, Some(0xF3));
        assert_step(&mut machine, 4, Some(0x00));
        assert_step(&mut machine, 4, Some(0x00));
        assert!(!machine.cpu.ime);
        assert_eq!(machine.cpu.registers.pc, 0x0104);
        assert_eq!(machine.mmu.read_byte(0xFF0F) & 1, 1);
    }

    #[test]
    fn halt_bug_reuses_the_opcode_as_an_immediate_operand_once() {
        let mut machine = machine_with_program(&[0x76, 0x3E, 0x00]); // HALT; LD A,n; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0x76));
        assert_eq!(machine.cpu.state, CpuState::Running);
        assert_step(&mut machine, 8, Some(0x3E));
        assert_eq!(machine.cpu.registers.a, 0x3E);
        assert_eq!(machine.cpu.registers.pc, 0x0102);
        assert_step(&mut machine, 4, Some(0x00));
        assert_eq!(machine.cpu.registers.pc, 0x0103);
    }

    #[test]
    fn halt_bug_executes_a_single_byte_instruction_twice() {
        let mut machine = machine_with_program(&[0x76, 0x3C, 0x00]); // HALT; INC A; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);
        let previous_a = machine.cpu.registers.a;

        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 4, Some(0x3C));
        assert_eq!(machine.cpu.registers.pc, 0x0101);
        assert_step(&mut machine, 4, Some(0x3C));
        assert_eq!(machine.cpu.registers.pc, 0x0102);
        assert_eq!(machine.cpu.registers.a, previous_a.wrapping_add(2));
    }

    #[test]
    fn halt_bug_followed_by_rst_saves_the_rst_address() {
        let mut machine = machine_with_program(&[0x76, 0xFF]); // HALT; RST $38
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 16, Some(0xFF));
        assert_eq!(machine.cpu.registers.pc, 0x0038);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0101);
    }

    #[test]
    fn interrupt_entry_cancels_an_ei_scheduled_while_ime_was_enabled() {
        let mut machine = machine_with_program(&[0xFB, 0x00]);
        machine.cpu.ime = true;
        machine.mmu.write_byte(0xFFFF, 0x05);
        assert_step(&mut machine, 4, Some(0xFB));

        machine.mmu.write_byte(0xFF0F, 0x05);
        assert_step(&mut machine, 20, None);
        assert_step(&mut machine, 4, Some(0x04)); // INC B, not a nested timer interrupt.
        assert!(!machine.cpu.ime);
        assert_eq!(machine.cpu.registers.pc, 0x0041);
        assert_eq!(machine.cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn reti_enables_interrupt_service_at_the_next_boundary() {
        let mut machine = machine_with_program(&[0xD9]); // RETI
        machine.cpu.registers.sp = 0xFFFC;
        machine.mmu.write_word(0xFFFC, 0x1234);
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 16, Some(0xD9));
        assert!(machine.cpu.ime);
        assert_eq!(machine.cpu.registers.pc, 0x1234);
        assert_eq!(machine.cpu.registers.sp, 0xFFFE);
        assert_step(&mut machine, 20, None);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x1234);
    }

    #[test]
    fn halt_with_ime_enabled_wakes_into_interrupt_entry() {
        let mut machine = machine_with_program(&[0x76, 0x00]);
        machine.cpu.ime = true;
        machine.mmu.write_byte(0xFFFF, 1);

        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 4, None);
        assert_eq!(machine.cpu.state, CpuState::Halted);
        machine.mmu.write_byte(0xFF0F, 1);
        assert_step(&mut machine, 20, None);
        assert_eq!(machine.cpu.state, CpuState::Running);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0101);
        assert_step(&mut machine, 4, Some(0x04));
        assert_eq!(machine.cpu.registers.pc, 0x0041);
    }

    #[test]
    fn disabled_and_unused_interrupt_bits_do_not_wake_halt() {
        let mut machine = machine_with_program(&[0x76, 0x00]);
        machine.mmu.write_byte(0xFFFF, 0xE1);
        machine.mmu.write_byte(0xFF0F, 0xE4);

        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 4, None);
        assert_eq!(machine.cpu.state, CpuState::Halted);
        assert_eq!(machine.cpu.registers.pc, 0x0101);
    }

    #[test]
    fn ei_halt_with_pending_interrupt_returns_to_halt_without_bugging_handler() {
        let mut machine = machine_with_program(&[0xFB, 0x76, 0x00]); // EI; HALT; NOP
        machine.mmu.write_byte(0xFFFF, 1);
        machine.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut machine, 4, Some(0xFB));
        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 20, None);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0101);

        let previous_b = machine.cpu.registers.b;
        assert_step(&mut machine, 4, Some(0x04));
        assert_eq!(machine.cpu.registers.b, previous_b.wrapping_add(1));
        assert_eq!(machine.cpu.registers.pc, 0x0041);
        assert_step(&mut machine, 16, Some(0xD9));
        assert_eq!(machine.cpu.registers.pc, 0x0101);
        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 4, None);
        assert_eq!(machine.cpu.state, CpuState::Halted);
    }

    #[test]
    fn ei_halt_without_pending_interrupt_returns_after_halt() {
        let mut machine = machine_with_program(&[0xFB, 0x76, 0x00]);
        machine.mmu.write_byte(0xFFFF, 1);

        assert_step(&mut machine, 4, Some(0xFB));
        assert_step(&mut machine, 4, Some(0x76));
        assert_step(&mut machine, 4, None);
        machine.mmu.write_byte(0xFF0F, 1);
        assert_step(&mut machine, 20, None);
        assert_eq!(machine.mmu.read_word(machine.cpu.registers.sp), 0x0102);
        assert_step(&mut machine, 4, Some(0x04));
        assert_step(&mut machine, 16, Some(0xD9));
        assert_step(&mut machine, 4, Some(0x00));
        assert_eq!(machine.cpu.registers.pc, 0x0103);
    }
}
