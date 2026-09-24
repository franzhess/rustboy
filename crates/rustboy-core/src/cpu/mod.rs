mod alu;
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
use crate::mbc::Mbc;
use crate::mmu::Mmu;
use crate::ButtonEvent;

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
    mmu: Mmu,
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
    pub fn new(rom: Box<dyn Mbc>) -> Cpu {
        Cpu {
            registers: Registers::new(),
            mmu: Mmu::new(rom),
            state: CpuState::Running,
            halt_bug: false,
            ime: false,
            ei_requested: 0,
        }
    }

    pub fn tick(&mut self) -> CpuStepResult {
        let result = if self.handle_irq() {
            // Interrupt entry is a five-M-cycle hardware sequence: acknowledge IF, push the
            // current PC, and load the vector. Devices continue running for all 20 T-cycles.
            CpuStepResult {
                cycles: 20,
                opcode: None,
                diagnostic: None,
            }
        } else if self.state == CpuState::Running {
            let result = self.do_cycle();
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
            CpuStepResult {
                cycles: 4,
                opcode: None,
                diagnostic: None,
            }
        };

        self.mmu.do_ticks(result.cycles);

        result
    }

    pub fn next_opcode(&self) -> Option<u8> {
        (self.state == CpuState::Running).then(|| self.mmu.read_byte(self.registers.pc))
    }

    pub fn state(&self) -> CpuState {
        self.state
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mmu.read_byte(address)
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

    pub fn take_frame(&mut self) -> Option<Vec<u8>> {
        self.mmu.take_frame()
    }

    pub fn take_audio_buffers(&mut self) -> Vec<Vec<i16>> {
        self.mmu.take_audio_buffers()
    }

    fn handle_irq(&mut self) -> bool {
        self.mmu.process_irq_requests(); //loads the irq requests into 0xFF0F

        let irq_requested = self.mmu.read_byte(0xFF0F);
        let irq = self.pending_interrupts();
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
        self.push(self.registers.pc);
        self.registers.pc = (0x0040 + 8 * irq_num) as u16;
        self.mmu.write_byte(0xFF0F, irq_requested & !(1 << irq_num));
        true
    }

    pub fn process_input_event(&mut self, event: ButtonEvent) {
        self.mmu.joypad.receive_event(event);
    }

    fn do_cycle(&mut self) -> CpuStepResult {
        let current_address = self.registers.pc;
        let opcode = self.fetch_byte();

        let (cycles, diagnostic) = match opcodes::execute(opcode, self) {
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

    fn fetch_byte(&mut self) -> u8 {
        let res = self.mmu.read_byte(self.registers.pc);
        if self.halt_bug {
            // HALT with IME clear and an enabled pending interrupt reuses the opcode byte as
            // the next instruction's first byte. The suppression applies to one fetch only.
            self.halt_bug = false;
        } else {
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
        res
    }

    fn fetch_word(&mut self) -> u16 {
        let res = self.mmu.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        res
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2); //stack grows down from 0xFFFE and stores words
        self.mmu.write_word(self.registers.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let result = self.mmu.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
    }

    fn call(&mut self, address: u16) {
        self.push(self.registers.pc); //it's not pc + 1 because after fetching the address the pc is already at the next instruction
        self.registers.pc = address;
    }

    fn return_from_call(&mut self) {
        self.registers.pc = self.pop();
    }

    fn jump_r(&mut self) {
        let offset = self.fetch_byte();
        self.registers.pc = self.registers.pc.wrapping_add(offset as i8 as i16 as u16);
    }

    fn pending_interrupts(&self) -> u8 {
        self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0x1F
    }
}

#[cfg(test)]
mod tests {
    use super::{opcodes, Cpu, CpuDiagnostic, CpuState};
    use crate::cpu::flags::CpuFlag;
    use crate::mbc::Mbc;

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

    pub(super) fn cpu_with_program(program: &[u8]) -> Cpu {
        let mut rom = vec![0; 0x100 + program.len()];
        rom[0x100..].copy_from_slice(program);
        rom[0x40] = 0x04; // VBlank handler: INC B; RETI
        rom[0x41] = 0xD9;
        let mut cpu = Cpu::new(Box::new(TestMbc(rom)));
        cpu.registers.sp = 0xFFFE;
        cpu
    }

    fn assert_step(cpu: &mut Cpu, cycles: usize, opcode: Option<u8>) {
        let result = cpu.tick();
        assert_eq!(result.cycles, cycles);
        assert_eq!(result.opcode, opcode);
        assert_eq!(result.diagnostic, None);
    }

    #[test]
    fn illegal_opcodes_report_once_and_idle_without_fetching_another_instruction() {
        for opcode in [
            0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
        ] {
            let mut cpu = cpu_with_program(&[opcode, 0x04]);
            let result = cpu.tick();
            assert_eq!((result.cycles, result.opcode), (4, Some(opcode)));
            assert_eq!(
                result.diagnostic,
                Some(CpuDiagnostic::IllegalOpcode {
                    address: 0x100,
                    opcode
                })
            );
            assert_eq!(cpu.state(), CpuState::IllegalOpcode);
            assert_eq!(cpu.next_opcode(), None);
            let registers = cpu.registers();
            for _ in 0..3 {
                assert_step(&mut cpu, 4, None);
                assert_eq!(cpu.state(), CpuState::IllegalOpcode);
                assert_eq!(cpu.registers.pc, 0x101);
                assert_eq!(cpu.registers(), registers);
            }
        }
    }

    #[test]
    fn illegal_opcode_diagnostic_keeps_the_fetch_address_when_pc_wraps() {
        let mut cpu = cpu_with_program(&[]);
        cpu.registers.pc = 0xFFFF;
        cpu.mmu.write_byte(0xFFFF, 0xD3);
        let result = cpu.tick();
        assert_eq!(cpu.registers.pc, 0);
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
                let mut cpu = cpu_with_program(&program);
                cpu.ime = ime;
                cpu.mmu.write_byte(0xFF07, 0x05);
                let entry = cpu.tick();
                assert_eq!(entry.opcode, Some(program[0]));
                assert_eq!(cpu.state(), expected_state);
                assert_eq!(cpu.registers.pc, resume_pc);
                assert_eq!(cpu.next_opcode(), None);
                for _ in 0..3 {
                    assert_step(&mut cpu, 4, None);
                }
                assert_eq!(cpu.mmu.read_byte(0xFF05), 1); // Sixteen T-cycles reached the timer.
                let previous_b = cpu.registers.b;
                cpu.mmu.write_byte(0xFFFF, 1);
                cpu.mmu.write_byte(0xFF0F, 1);

                if ime {
                    assert_step(&mut cpu, 20, None);
                    assert_eq!(cpu.registers.pc, 0x40);
                    assert_eq!(cpu.mmu.read_word(cpu.registers.sp), resume_pc);
                    assert_eq!(cpu.registers.b, previous_b);
                    assert_eq!(cpu.mmu.read_byte(0xFF0F) & 1, 0);
                } else {
                    assert_step(&mut cpu, 4, Some(0x04));
                    assert_eq!(cpu.registers.pc, resume_pc + 1);
                    assert_eq!(cpu.registers.b, previous_b.wrapping_add(1));
                    assert_eq!(cpu.mmu.read_byte(0xFF0F) & 1, 1);
                }
                assert_eq!(cpu.state(), CpuState::Running);
                assert!(cpu.next_opcode().is_some());
            }
        }
    }

    #[test]
    fn absolute_jump_consumes_sixteen_cycles_and_advances_the_timer() {
        let mut cpu = cpu_with_program(&[0xC3, 0x34, 0x12]);
        cpu.mmu.write_byte(0xFF04, 0);
        cpu.mmu.write_byte(0xFF07, 0x05);

        assert_step(&mut cpu, 16, Some(0xC3));
        assert_eq!(cpu.registers.pc, 0x1234);
        assert_eq!(cpu.mmu.read_byte(0xFF05), 1);
    }

    #[test]
    fn add_indirect_hl_consumes_eight_cycles_per_memory_operand() {
        let mut cpu = cpu_with_program(&[0x86, 0x86]);
        cpu.registers.a = 1;
        cpu.registers.set_hl(0xC000);
        cpu.mmu.write_byte(0xC000, 2);
        cpu.mmu.write_byte(0xFF04, 0);
        cpu.mmu.write_byte(0xFF07, 0x05);

        assert_step(&mut cpu, 8, Some(0x86));
        assert_eq!(cpu.registers.a, 3);
        assert_eq!(cpu.mmu.read_byte(0xFF05), 0);
        assert_step(&mut cpu, 8, Some(0x86));
        assert_eq!(cpu.registers.a, 5);
        assert_eq!(cpu.registers.pc, 0x102);
        assert_eq!(cpu.mmu.read_byte(0xFF05), 1);
    }

    #[test]
    fn instruction_fetch_wraps_the_program_counter() {
        let mut cpu = Cpu::new(Box::new(TestMbc(Vec::new())));
        cpu.registers.pc = 0xFFFF;

        cpu.fetch_byte();
        assert_eq!(cpu.registers.pc, 0);

        cpu.registers.pc = 0xFFFE;
        cpu.fetch_word();
        assert_eq!(cpu.registers.pc, 0);
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
                let mut cpu = cpu_with_program(&[opcode, immediate]);
                cpu.registers.sp = sp;
                cpu.registers.set_hl(0x1234);
                cpu.registers.set_af(0x5AF0);

                assert_step(&mut cpu, cycles, Some(opcode));

                let (expected_sp, expected_hl) = if opcode == 0xE8 {
                    (expected, 0x1234)
                } else {
                    (sp, expected)
                };
                assert_eq!(
                    (
                        cpu.registers.sp,
                        cpu.registers.get_hl(),
                        cpu.registers.get_af(),
                        cpu.registers.pc
                    ),
                    (expected_sp, expected_hl, 0x5A00 | flags, 0x102),
                    "opcode={opcode:02X}, SP={sp:04X}, immediate={immediate:02X}"
                );
            }
        }
    }

    #[test]
    fn not_taken_conditional_jump_wraps_while_skipping_its_operand() {
        let mut cpu = Cpu::new(Box::new(TestMbc(Vec::new())));
        cpu.registers.pc = 0xFFFF;
        cpu.registers.flags.set_flag(CpuFlag::Z, true);

        // PC already points to JR NZ's one-byte operand after its opcode was fetched.
        opcodes::execute(0x20, &mut cpu);

        assert_eq!(cpu.registers.pc, 0);
    }

    #[test]
    fn highest_priority_interrupt_consumes_twenty_cycles_and_ticks_devices() {
        let mut cpu = Cpu::new(Box::new(TestMbc(Vec::new())));
        cpu.registers.pc = 0x1234;
        cpu.registers.sp = 0xFFFE;
        cpu.ime = true;
        cpu.mmu.write_byte(0xFFFF, 0b0000_0101);
        cpu.mmu.write_byte(0xFF0F, 0b0000_0101);
        cpu.mmu.write_byte(0xFF07, 0x05);

        let result = cpu.tick();
        assert_eq!(result.cycles, 20);
        assert_eq!(result.opcode, None);
        assert!(!cpu.ime);

        // VBlank (bit 0) wins over the simultaneously pending timer interrupt (bit 2).
        assert_eq!(cpu.registers.pc, 0x0040);
        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(cpu.mmu.read_word(0xFFFC), 0x1234);
        assert_eq!(cpu.mmu.read_byte(0xFF0F), 0b1110_0100);

        // Twenty T-cycles include the timer's first bit-3 falling edge at cycle 16.
        assert_eq!(cpu.mmu.read_byte(0xFF05), 1);

        // The handler's NOP runs on the next step, without servicing the pending timer IRQ.
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x00));
        assert_eq!(cpu.registers.pc, 0x0041);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn halt_idle_reports_no_opcode_but_waking_execution_does() {
        let mut cpu = Cpu::new(Box::new(TestMbc(Vec::new())));
        cpu.registers.pc = 0xC000;
        cpu.mmu.write_byte(0xC000, 0x76); // HALT
        cpu.mmu.write_byte(0xC001, 0x04); // INC B

        assert_eq!(cpu.tick().opcode, Some(0x76));
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, None);
        assert_eq!(cpu.registers.pc, 0xC001);

        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x04));
        assert_eq!(cpu.registers.pc, 0xC002);
    }

    #[test]
    fn ei_enables_interrupts_after_exactly_one_following_instruction() {
        let mut cpu = cpu_with_program(&[0xFB, 0x00, 0x00]); // EI; NOP; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0xFB));
        assert!(!cpu.ime);
        assert_step(&mut cpu, 4, Some(0x00));
        assert!(cpu.ime);

        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.registers.pc, 0x0040);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0102);
    }

    #[test]
    fn repeated_ei_does_not_delay_an_already_scheduled_enable() {
        let mut cpu = cpu_with_program(&[0xFB, 0xFB, 0x00]); // EI; EI; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0xFB));
        assert_step(&mut cpu, 4, Some(0xFB));
        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.registers.pc, 0x0040);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0102);
    }

    #[test]
    fn di_cancels_a_pending_ei_enable() {
        let mut cpu = cpu_with_program(&[0xFB, 0xF3, 0x00, 0x00]); // EI; DI; NOP; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0xFB));
        assert_step(&mut cpu, 4, Some(0xF3));
        assert_step(&mut cpu, 4, Some(0x00));
        assert_step(&mut cpu, 4, Some(0x00));
        assert!(!cpu.ime);
        assert_eq!(cpu.registers.pc, 0x0104);
        assert_eq!(cpu.mmu.read_byte(0xFF0F) & 1, 1);
    }

    #[test]
    fn halt_bug_reuses_the_opcode_as_an_immediate_operand_once() {
        let mut cpu = cpu_with_program(&[0x76, 0x3E, 0x00]); // HALT; LD A,n; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0x76));
        assert_eq!(cpu.state, CpuState::Running);
        assert_step(&mut cpu, 8, Some(0x3E));
        assert_eq!(cpu.registers.a, 0x3E);
        assert_eq!(cpu.registers.pc, 0x0102);
        assert_step(&mut cpu, 4, Some(0x00));
        assert_eq!(cpu.registers.pc, 0x0103);
    }

    #[test]
    fn halt_bug_executes_a_single_byte_instruction_twice() {
        let mut cpu = cpu_with_program(&[0x76, 0x3C, 0x00]); // HALT; INC A; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);
        let previous_a = cpu.registers.a;

        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 4, Some(0x3C));
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_step(&mut cpu, 4, Some(0x3C));
        assert_eq!(cpu.registers.pc, 0x0102);
        assert_eq!(cpu.registers.a, previous_a.wrapping_add(2));
    }

    #[test]
    fn halt_bug_followed_by_rst_saves_the_rst_address() {
        let mut cpu = cpu_with_program(&[0x76, 0xFF]); // HALT; RST $38
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 16, Some(0xFF));
        assert_eq!(cpu.registers.pc, 0x0038);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0101);
    }

    #[test]
    fn interrupt_entry_cancels_an_ei_scheduled_while_ime_was_enabled() {
        let mut cpu = cpu_with_program(&[0xFB, 0x00]);
        cpu.ime = true;
        cpu.mmu.write_byte(0xFFFF, 0x05);
        assert_step(&mut cpu, 4, Some(0xFB));

        cpu.mmu.write_byte(0xFF0F, 0x05);
        assert_step(&mut cpu, 20, None);
        assert_step(&mut cpu, 4, Some(0x04)); // INC B, not a nested timer interrupt.
        assert!(!cpu.ime);
        assert_eq!(cpu.registers.pc, 0x0041);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn reti_enables_interrupt_service_at_the_next_boundary() {
        let mut cpu = cpu_with_program(&[0xD9]); // RETI
        cpu.registers.sp = 0xFFFC;
        cpu.mmu.write_word(0xFFFC, 0x1234);
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 16, Some(0xD9));
        assert!(cpu.ime);
        assert_eq!(cpu.registers.pc, 0x1234);
        assert_eq!(cpu.registers.sp, 0xFFFE);
        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x1234);
    }

    #[test]
    fn halt_with_ime_enabled_wakes_into_interrupt_entry() {
        let mut cpu = cpu_with_program(&[0x76, 0x00]);
        cpu.ime = true;
        cpu.mmu.write_byte(0xFFFF, 1);

        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 4, None);
        assert_eq!(cpu.state, CpuState::Halted);
        cpu.mmu.write_byte(0xFF0F, 1);
        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.state, CpuState::Running);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0101);
        assert_step(&mut cpu, 4, Some(0x04));
        assert_eq!(cpu.registers.pc, 0x0041);
    }

    #[test]
    fn disabled_and_unused_interrupt_bits_do_not_wake_halt() {
        let mut cpu = cpu_with_program(&[0x76, 0x00]);
        cpu.mmu.write_byte(0xFFFF, 0xE1);
        cpu.mmu.write_byte(0xFF0F, 0xE4);

        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 4, None);
        assert_eq!(cpu.state, CpuState::Halted);
        assert_eq!(cpu.registers.pc, 0x0101);
    }

    #[test]
    fn ei_halt_with_pending_interrupt_returns_to_halt_without_bugging_handler() {
        let mut cpu = cpu_with_program(&[0xFB, 0x76, 0x00]); // EI; HALT; NOP
        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);

        assert_step(&mut cpu, 4, Some(0xFB));
        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0101);

        let previous_b = cpu.registers.b;
        assert_step(&mut cpu, 4, Some(0x04));
        assert_eq!(cpu.registers.b, previous_b.wrapping_add(1));
        assert_eq!(cpu.registers.pc, 0x0041);
        assert_step(&mut cpu, 16, Some(0xD9));
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 4, None);
        assert_eq!(cpu.state, CpuState::Halted);
    }

    #[test]
    fn ei_halt_without_pending_interrupt_returns_after_halt() {
        let mut cpu = cpu_with_program(&[0xFB, 0x76, 0x00]);
        cpu.mmu.write_byte(0xFFFF, 1);

        assert_step(&mut cpu, 4, Some(0xFB));
        assert_step(&mut cpu, 4, Some(0x76));
        assert_step(&mut cpu, 4, None);
        cpu.mmu.write_byte(0xFF0F, 1);
        assert_step(&mut cpu, 20, None);
        assert_eq!(cpu.mmu.read_word(cpu.registers.sp), 0x0102);
        assert_step(&mut cpu, 4, Some(0x04));
        assert_step(&mut cpu, 16, Some(0xD9));
        assert_step(&mut cpu, 4, Some(0x00));
        assert_eq!(cpu.registers.pc, 0x0103);
    }
}
