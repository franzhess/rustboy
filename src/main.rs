extern crate rustboy;

use rustboy::hardware::init_hardware;
use rustboy::cpu::CPU;

use std::fs;
use std::io::Read;

fn main() {
  let (mut input, _display) = init_hardware(160, 144);

  let mut buffer: [u8;0xFFFF] = [0; 0xFFFF];

  let mut f = fs::File::open("roms/Tetris.gb").unwrap();
  f.read(&mut buffer).unwrap();

  let mut cpu = CPU::new(buffer);

  while let Ok(input_state) = input.process_input() {
    cpu.tick(input_state);
  }
}
