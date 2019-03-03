pub mod mbc;
pub mod gameboy;
pub mod cpu;
pub mod joypad;

mod gpu;
mod mmu;
mod timer;

pub const VRAM_SIZE: usize = 0x2000; //8kB vram
pub const VOAM_SIZE: usize = 0xA0;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

use crate::cpu::CPU;
use crate::joypad::GBKeyEvent;

use std::sync::mpsc::{Sender, Receiver};

pub enum GBEvent {
  KeyEvent(GBKeyEvent),
  Quit
}

pub fn main_loop(mut cpu: CPU, input_receiver: Receiver<GBEvent>, screen_sender: Sender<Vec<u8>>) {
  'running: loop {
    for event in input_receiver.try_iter() {
      match event {
        GBEvent::KeyEvent(key_event) => cpu.process_input_event(key_event),
        GBEvent::Quit => break 'running,
      }
    }

    cpu.tick();

    if cpu.get_screen_updated() {
      screen_sender.send(cpu.get_screen_buffer());
    }
  }
}