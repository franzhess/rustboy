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

pub use cpu::RegisterValues;

use crate::cpu::Cpu;
use crate::mbc::Cartridge;

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
    /// The opcode executed by this step; absent during interrupt entry or HALT idle.
    pub opcode: Option<u8>,
    pub frame: Option<Vec<u8>>,
    pub audio_buffers: Vec<Vec<i16>>,
}

pub struct Machine {
    cpu: Cpu,
}

impl Machine {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: Cpu::new(cartridge.into_controller()),
        }
    }

    pub fn step(&mut self) -> StepResult {
        let result = self.cpu.tick();
        StepResult {
            cycles: result.cycles,
            opcode: result.opcode,
            frame: self.cpu.take_frame(),
            audio_buffers: self.cpu.take_audio_buffers(),
        }
    }

    pub fn process_input(&mut self, event: ButtonEvent) {
        self.cpu.process_input_event(event);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.cpu.read_byte(address)
    }

    /// Peeks at the current PC when awake; interrupt entry may precede its execution.
    pub fn next_opcode(&self) -> Option<u8> {
        self.cpu.next_opcode()
    }

    pub fn registers(&self) -> RegisterValues {
        self.cpu.registers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
