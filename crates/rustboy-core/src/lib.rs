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
        let opcode = self.cpu.next_opcode();
        let cycles = self.cpu.tick();
        StepResult {
            cycles,
            opcode,
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

    pub fn next_opcode(&self) -> Option<u8> {
        self.cpu.next_opcode()
    }

    pub fn registers(&self) -> RegisterValues {
        self.cpu.registers()
    }
}
