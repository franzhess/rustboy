pub mod mbc;
pub mod cpu;
pub mod mmu;

pub const VRAM_SIZE: usize = 0x2000; //8kB vram
pub const VOAM_SIZE: usize = 0xA0;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

use std::sync::mpsc::{Sender, Receiver};
use std::time::{Instant, Duration};

use crate::cpu::Cpu;
use std::fs;
use std::io::Read;

pub enum GBEvent {
  KeyEvent(GBKeyEvent),
  Quit
}

pub enum GBKeyCode {
  Up = 0,
  Down,
  Left,
  Right,
  A,
  B,
  Start,
  Select
}

#[derive(PartialEq)]
pub enum GBKeyState {
  KeyUp,
  KeyDown
}

pub struct GBKeyEvent {
  pub key_code: GBKeyCode,
  pub state: GBKeyState
}

pub fn create_cpu(rom_file_name: &str) -> Cpu {
  let mut rom = fs::File::open(rom_file_name).unwrap();
  let mut buffer: [u8;0xFFFF] = [0; 0xFFFF];

  rom.read(&mut buffer).unwrap();
  Cpu::new(buffer)
}

pub fn main_loop(mut cpu: Cpu, input_receiver: Receiver<GBEvent>, screen_sender: Sender<Vec<u8>>) {
  let mut last_update = Instant::now();
  let mut ticks: usize = 0;
  let one_second = Duration::from_secs(1);

  'running: loop {
    for event in input_receiver.try_iter() {
      match event {
        GBEvent::KeyEvent(key_event) => cpu.process_input_event(key_event),
        GBEvent::Quit => break 'running,
      }
    }

    ticks += cpu.tick();

    if cpu.get_screen_updated() {
      screen_sender.send(cpu.get_screen_buffer());
    }

    if last_update.elapsed() >= one_second {
      println!("{} ticks", ticks);
      ticks = 0;
      last_update = Instant::now();
    }
  }
}