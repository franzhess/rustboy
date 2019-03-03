mod hardware;

use hardware::init_hardware;

use std::fs;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

use core::cpu::CPU;
use core::joypad::GBKeyEvent;
use core::SCREEN_WIDTH;
use core::SCREEN_HEIGHT;
use core::main_loop;
use core::GBEvent;

fn main() {

  let mut buffer: [u8;0xFFFF] = [0; 0xFFFF];

  let (video_sender, video_receiver) = mpsc::channel::<Vec<u8>>();
  let (input_sender, input_receiver) = mpsc::channel::<GBEvent>();

  let mut rom = fs::File::open("roms/tetris.gb").unwrap(); //"roms/cpu_instrs/cpu_instrs.gb"
  rom.read(&mut buffer).unwrap();
  let mut cpu = CPU::new(buffer);

  let (mut input, mut display) = init_hardware(2 * SCREEN_WIDTH as u32, 2 * SCREEN_HEIGHT as u32, input_sender);
  let cpu_thread = thread::spawn(move|| main_loop(cpu, input_receiver, video_sender));

  while input.process_input() {
    match video_receiver.recv() {
      Ok(screen_buffer) => display.draw_screen(screen_buffer),
      Err(..) => break
    }
  }

  cpu_thread.join();
}
