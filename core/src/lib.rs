pub mod cpu;
pub mod mbc;

mod mmu;
mod joypad;
mod timer;
mod ppu;
mod apu;
mod serial;

pub const CPU_FREQUENCY: usize = 4_194_304; //4.194304 MHz

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const AUDIO_OUTPUT_FREQUENCY: usize = 44_100;
pub const AUDIO_SAMPLE_SIZE: usize = size_of::<i16>() * 2;

use std::sync::mpsc::{Sender, Receiver};
use std::time::{Instant, Duration};

use crate::cpu::Cpu;
use std::thread::sleep;
use std::mem::size_of;

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

pub fn main_loop(mut cpu: Cpu, input_receiver: Receiver<GBEvent>) {
  let mut last_frame = Instant::now();
  let one_frame = Duration::from_micros(16666); //60Hz
  let mut ticks_per_frame : usize = 0;
  let target_ticks_per_frame =  CPU_FREQUENCY / 60 as usize;

  'running: loop {
    for event in input_receiver.try_iter() {
      match event {
        GBEvent::KeyEvent(key_event) => cpu.process_input_event(key_event),
        GBEvent::Quit => break 'running,
      }
    }

    ticks_per_frame += cpu.tick();

    if last_frame.elapsed() < one_frame {
      if ticks_per_frame >= target_ticks_per_frame {
        sleep(one_frame - last_frame.elapsed());
        ticks_per_frame = 0;
        last_frame = Instant::now();
      }
    } else {
      println!("CPU slow! {} ticks should be {}", ticks_per_frame, target_ticks_per_frame);
      ticks_per_frame = 0;
      last_frame = Instant::now();
    }
  }
}