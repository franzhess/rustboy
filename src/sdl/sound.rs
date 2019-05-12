use sdl2::audio::*;
use sdl2::Sdl;
use core::AUDIO_OUTPUT_FREQUENCY;
use core::AUDIO_SAMPLE_SIZE;

pub struct Sound {
  device: AudioQueue<i16>
}

impl Sound {
  pub fn new(sdl: &Sdl) -> Sound {
    let audio_subsystem = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
      freq: Some(AUDIO_OUTPUT_FREQUENCY as i32),
      channels: Some(1), // stereo
      samples: Some(AUDIO_SAMPLE_SIZE as u16),
    };

    let device = audio_subsystem.open_queue(None, &desired_spec).unwrap();

    Sound {
      device
    }
  }

  pub fn queue(&mut self, data: Vec<i16>) {
    self.device.queue(data.as_slice());
  }

  pub fn queue_size(&self) -> u32 {
    self.device.size()
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