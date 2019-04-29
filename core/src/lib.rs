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
use std::thread::sleep;

#[derive(Debug)]
pub enum GBEvent {
  KeyEvent(GBKeyEvent),
  Quit
}

#[derive(Debug)]
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

#[derive(PartialEq, Debug)]
pub enum GBKeyState {
  KeyUp,
  KeyDown
}

#[derive(Debug)]
pub struct GBKeyEvent {
  pub key_code: GBKeyCode,
  pub state: GBKeyState
}

pub fn main_loop(mut cpu: Cpu, input_receiver: Receiver<GBEvent>, screen_sender: Sender<Vec<u8>>) {
  let mut last_frame = Instant::now();
  let one_frame = Duration::from_micros(16666); //60Hz
  let mut ticks_per_frame : usize = 0;
  let target_ticks_per_frame = 4194304 as usize / 60 as usize; //4.194304 MHz

  'running: loop {
    for event in input_receiver.try_iter() {
      match event {
        GBEvent::KeyEvent(key_event) => cpu.process_input_event(key_event),
        GBEvent::Quit => break 'running,
      }
    }

    ticks_per_frame += cpu.tick();

    if cpu.get_screen_updated() {
      screen_sender.send(cpu.get_screen_buffer()).expect("failed to send video data!");
    }

    if ticks_per_frame >= target_ticks_per_frame && last_frame.elapsed() < one_frame {
      sleep(one_frame - last_frame.elapsed());
      ticks_per_frame = 0;
      last_frame = Instant::now();
    }
  }
}