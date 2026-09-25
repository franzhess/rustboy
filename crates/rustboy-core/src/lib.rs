mod apu;
mod cpu;
mod joypad;
pub mod mbc;
mod mmu;
mod ppu;
mod serial;
mod timer;

pub const CPU_FREQUENCY: usize = 4_194_304;
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const AUDIO_OUTPUT_FREQUENCY: usize = 48_000;

pub use cpu::{CpuDiagnostic, CpuState, RegisterValues};

use crate::cpu::Cpu;
use crate::mbc::Cartridge;
use crate::mmu::Mmu;

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Up = 0,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Released,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonEvent {
    pub button: Button,
    pub state: ButtonState,
}

pub struct StepResult {
    pub cycles: usize,
    /// The opcode fetched for execution; absent during interrupt entry or any idle step.
    pub opcode: Option<u8>,
    /// A diagnostic from this step, emitted once per encounter rather than during idle.
    pub diagnostic: Option<CpuDiagnostic>,
    pub frame: Option<Vec<u8>>,
    pub audio_buffers: Vec<Vec<i16>>,
}

pub struct Machine {
    cpu: Cpu,
    mmu: Mmu,
}

impl Machine {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: Cpu::new(),
            mmu: Mmu::new(cartridge.into_controller()),
        }
    }

    pub fn step(&mut self) -> StepResult {
        // Requests from the preceding step are collected before CPU dispatch.
        // Devices then advance by this step's total, exactly once, before output
        // collection. Bus accesses are still instruction-batched, not M-cycle scheduled.
        self.mmu.process_irq_requests();
        let result = self.cpu.tick(&mut self.mmu);
        self.mmu.do_ticks(result.cycles);
        StepResult {
            cycles: result.cycles,
            opcode: result.opcode,
            diagnostic: result.diagnostic,
            frame: self.mmu.take_frame(),
            audio_buffers: self.mmu.take_audio_buffers(),
        }
    }

    pub fn process_input(&mut self, event: ButtonEvent) {
        self.mmu.process_input_event(event);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mmu.read_byte(address)
    }

    /// Peeks at the current PC when awake; interrupt entry may precede its execution.
    pub fn next_opcode(&self) -> Option<u8> {
        self.cpu.next_opcode(&self.mmu)
    }

    pub fn registers(&self) -> RegisterValues {
        self.cpu.registers()
    }

    pub fn cpu_state(&self) -> CpuState {
        self.cpu.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine_with_program(program: &[u8]) -> Machine {
        let mut rom = vec![0; 0x8000];
        rom[0x100..0x100 + program.len()].copy_from_slice(program);
        Machine::new(Cartridge::from_bytes(rom).expect("valid ROM"))
    }

    #[test]
    fn device_requests_are_collected_at_the_next_step_before_operand_reads() {
        let mut machine = machine_with_program(&[0x00, 0xF0, 0x0F]); // NOP; LDH A,(IF)
        machine.mmu.write_byte(0xFF05, 0xFF);
        machine.mmu.write_byte(0xFF07, 0x05);
        machine.mmu.do_ticks(16); // TIMA overflow; reload is four cycles away.

        let first = machine.step();
        assert_eq!((first.cycles, first.opcode), (4, Some(0x00)));
        // The NOP's device advancement requests an interrupt, but it is not
        // collected into IF until the next step starts.
        assert_eq!(machine.read_byte(0xFF0F) & 4, 0);
        let second = machine.step();
        assert_eq!((second.cycles, second.opcode), (12, Some(0xF0)));
        assert_eq!(machine.registers().a & 4, 4);
        assert_eq!(machine.read_byte(0xFF0F) & 4, 4);
    }

    #[test]
    fn devices_advance_after_cpu_bus_accesses() {
        let mut machine = machine_with_program(&[0xF0, 0x04]); // LDH A,(DIV)
        machine.mmu.do_ticks(252);
        assert_eq!(machine.read_byte(0xFF04), 0);

        let result = machine.step();

        assert_eq!((result.cycles, result.opcode), (12, Some(0xF0)));
        assert_eq!(machine.registers().a, 0); // Read before the divider advances.
        assert_eq!(machine.read_byte(0xFF04), 1);
    }

    #[test]
    fn input_reaches_the_joypad_without_executing_a_cpu_step() {
        let mut machine = machine_with_program(&[0]);
        let registers = machine.registers();
        assert_eq!(machine.read_byte(0xFF00) & 1, 1);
        machine.process_input(ButtonEvent {
            button: Button::Right,
            state: ButtonState::Pressed,
        });
        assert_eq!(machine.read_byte(0xFF00) & 1, 0);
        machine.process_input(ButtonEvent {
            button: Button::Right,
            state: ButtonState::Released,
        });
        assert_eq!(machine.read_byte(0xFF00) & 1, 1);
        assert_eq!(machine.registers(), registers);
    }

    #[test]
    fn machine_exposes_cpu_state_and_one_shot_structured_diagnostics() {
        let mut rom = vec![0; 0x148];
        rom[0x100] = 0xD3;
        let mut machine = Machine::new(Cartridge::from_bytes(rom).expect("valid ROM"));
        assert_eq!(machine.cpu_state(), CpuState::Running);
        assert_eq!(machine.next_opcode(), Some(0xD3));

        let result = machine.step();
        let diagnostic = CpuDiagnostic::IllegalOpcode {
            address: 0x100,
            opcode: 0xD3,
        };
        assert_eq!(result.diagnostic, Some(diagnostic));
        assert_eq!(diagnostic.to_string(), "Illegal opcode 0xD3 at 0x0100");
        assert_eq!((result.cycles, result.opcode), (4, Some(0xD3)));
        assert_eq!(machine.cpu_state(), CpuState::IllegalOpcode);
        assert_eq!(machine.next_opcode(), None);

        let idle = machine.step();
        assert_eq!((idle.cycles, idle.opcode, idle.diagnostic), (4, None, None));
    }

    #[test]
    fn cartridge_can_wait_for_vblank_before_initializing_the_lcd() {
        let mut rom = vec![0; 0x8000];
        rom[0x100..0x110].copy_from_slice(&[
            0xF0, 0x44, // LDH A,(LY)
            0xFE, 0x90, // CP 144
            0x38, 0xFA, // JR C,0100
            0xAF, // XOR A
            0xE0, 0x40, // LDH (LCDC),A: disable LCD after reaching VBlank
            0x3E, 0x42, // LD A,42
            0xEA, 0x00, 0xC0, // LD (C000),A: startup completed
            0x18, 0xFE, // JR -2
        ]);
        let mut machine = Machine::new(Cartridge::from_bytes(rom).expect("valid ROM"));
        let mut cycles = 0;
        let mut saw_frame = false;
        while cycles < 70_224 && machine.read_byte(0xC000) != 0x42 {
            let step = machine.step();
            cycles += step.cycles;
            saw_frame |= step.frame.is_some();
        }

        assert_eq!(machine.read_byte(0xC000), 0x42, "startup wait must finish");
        assert!(saw_frame, "the initially enabled LCD must produce a frame");
        assert_ne!(
            machine.read_byte(0xFF0F) & 1,
            0,
            "VBlank must request an IRQ"
        );
        assert_eq!(machine.read_byte(0xFF40), 0);
        assert_eq!(machine.read_byte(0xFF44), 0);
    }

    #[test]
    fn machine_starts_with_post_boot_lcd_control() {
        let machine = Machine::new(Cartridge::from_bytes(vec![0; 0x8000]).expect("valid ROM"));

        assert_eq!(machine.read_byte(0xFF40), 0x91);
    }
}
