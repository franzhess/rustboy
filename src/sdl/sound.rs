use sdl2::audio::*;
use sdl2::Sdl;
use core::AUDIO_OUTPUT_FREQUENCY;
use core::AUDIO_BUFFER_SIZE;
use std::collections::vec_deque::VecDeque;

struct SoundBuffer {
  queue: VecDeque<i16>
}

impl SoundBuffer {
  pub fn new() -> SoundBuffer {
    SoundBuffer {
      queue: VecDeque::new()
    }
  }

  pub fn queue(&mut self, data: Vec<i16>) {
    self.queue.append(&mut VecDeque::from(data));
  }
}

impl AudioCallback for SoundBuffer {
  type Channel = i16;

  fn callback(&mut self, out: &mut [i16]) {
    for x in out.iter_mut() {
      *x = match self.queue.pop_front() {
        Some(x) => x,
        None => { /*println!("Audio buffer underflow!");*/ 0 }
      };
    }
  }
}

pub struct Sound {
  device: AudioDevice<SoundBuffer>
}

impl Sound {
  pub fn new(sdl: &Sdl) -> Sound {
    let audio_subsystem = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
      freq: Some(AUDIO_OUTPUT_FREQUENCY as i32),
      channels: Some(2), // stereo
      samples: Some(AUDIO_BUFFER_SIZE as u16),
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, | _spec | {
      SoundBuffer::new()
    }).unwrap();

    Sound {
      device,
    }
  }

  pub fn queue(&mut self, data: Vec<i16>) {
    self.device.lock().queue(data);
  }

  pub fn queue_size(&mut self) -> usize {
    self.device.lock().queue.len()
  }

  pub fn play(&mut self) {
    match self.device.status() {
      AudioStatus::Paused | AudioStatus::Stopped => self.device.resume(),
      _ => ()
    }
  }

  pub fn stop(&mut self) {
    match self.device.status() {
      AudioStatus::Playing => self.device.pause(),
      _ => ()
    }
  }
}