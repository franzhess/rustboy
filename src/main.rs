mod sdl;

use sdl::init_hardware;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use core::*;
use core::cpu::Cpu;
use core::mbc::load_rom;

fn main() {
  let rom = load_rom("roms/tetris.gb"); //cgb_sound/rom_singles/01-registers.gb");
  println!("Successfully loaded: {}", rom.name());

  let (video_sender, video_receiver) = mpsc::channel::<Vec<u8>>();
  let (audio_sender, audio_receiver) = mpsc::channel::<Vec<i16>>();
  let (input_sender, input_receiver) = mpsc::channel::<GBEvent>();

  let (mut input, mut display, mut sound) = init_hardware(2 * SCREEN_WIDTH as u32, 2 * SCREEN_HEIGHT as u32, input_sender);

  let cpu = Cpu::new(rom, video_sender, audio_sender);
  let cpu_thread = thread::Builder::new().name("cpu".to_string()).spawn(move|| main_loop(cpu, input_receiver)).unwrap();

  sound.play();

  while input.process_input() {
    match video_receiver.try_recv() {
      Ok(screen_buffer) => display.draw_screen(screen_buffer),
      Err(TryRecvError::Disconnected) => break,
      Err(TryRecvError::Empty) => ()
    }

    match audio_receiver.try_recv() {
      Ok(sound_buffer) => sound.queue(sound_buffer),
      Err(TryRecvError::Disconnected) => break,
      Err(TryRecvError::Empty) => ()
    }

    if sound.queue_size() <= AUDIO_BUFFER_SIZE {
      cpu_thread.thread().unpark();
    }

    sleep(Duration::from_millis(1));  //sleep for a ms to not have this thread run at 100% cpu
  }

  sound.stop();

  cpu_thread.thread().unpark(); // unpark if we exit while the cpu thread is parked
  cpu_thread.join().expect("failed to end cpu thread!");
}
