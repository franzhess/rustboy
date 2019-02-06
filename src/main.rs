mod hardware;
mod cpu;

use hardware::init_hardware;
use cpu::CPU;

use std::fs;
use std::io::Read;
use std::time::Instant;
use std::time::Duration;
use std::thread;

fn main() {
  let (mut input, mut display) = init_hardware(4 * 166, 4 * 144);

  let mut buffer: [u8;0xFFFF] = [0; 0xFFFF];

  let mut f = fs::File::open("roms/cpu_instrs/cpu_instrs.gb").unwrap();
  f.read(&mut buffer).unwrap();

  let mut cpu = CPU::new(buffer);

  let mut ticks: usize = 0;
  let mut last_ticks: usize = 0;
  let mut now = Instant::now();
  while let Ok(input_state) = input.process_input() {
    ticks += cpu.tick(input_state);
    //let ten_millis = Duration::from_millis(100);
    //thread::sleep(ten_millis);

    if now.elapsed() >= Duration::from_millis(1000) {
      display.draw_screen(cpu.get_screen_buffer());
      println!("{} ticks in the last second", ticks - last_ticks);
      last_ticks = ticks;
      now = Instant::now();
    }
  }
}
