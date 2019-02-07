mod hardware;
mod cpu;

use hardware::init_hardware;
use cpu::CPU;

use std::fs;
use std::io::Read;

fn main() {
  let (mut input, mut display) = init_hardware(2 * 166, 2 * 144);

  let mut buffer: [u8;0xFFFF] = [0; 0xFFFF];

  let mut f = fs::File::open("roms/drm.gb").unwrap(); //"roms/cpu_instrs/cpu_instrs.gb"
  f.read(&mut buffer).unwrap();

  let mut cpu = CPU::new(buffer);
  while let Ok(input_state) = input.process_input() {
    cpu.tick(input_state);

    if cpu.get_screen_updated() {
      display.draw_screen(cpu.get_screen_buffer());
    }
  }
}
