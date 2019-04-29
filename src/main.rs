mod hardware;

use hardware::init_hardware;
use std::sync::mpsc;
use std::thread;

use core::*;

fn main() {
  let (video_sender, video_receiver) = mpsc::channel::<Vec<u8>>();
  let (input_sender, input_receiver) = mpsc::channel::<GBEvent>();

  let cpu = create_cpu("Tetris.gb");

  let (mut input, mut display) = init_hardware(2 * SCREEN_WIDTH as u32, 2 * SCREEN_HEIGHT as u32, input_sender);
  let cpu_thread = thread::spawn(move|| main_loop(cpu, input_receiver, video_sender));

  while input.process_input() {
    match video_receiver.recv() {
      Ok(screen_buffer) => display.draw_screen(screen_buffer),
      Err(..) => break
    }
  }

  cpu_thread.join().expect("failed to end cpu thread!");
}
