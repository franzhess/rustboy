extern crate rand;
use crate::apu::VolumeEnvelope;
use self::rand::Rng;

pub struct Noise {
  enabled: bool,
  counter: usize,
  period: usize,
  lfsr: u16, //gameboy has a 15bit lfsr - 16bit is good enough :)
  short: bool, //gameboy has 15bit/7bit - 16/8 for us
  length: u16,
  duration: u16,
  length_enabled: bool,
  volume_envelope: VolumeEnvelope
}

impl Noise {
  pub fn new() -> Noise {
    Noise {
      enabled: false,
      counter: 0,
      period: 1,
      lfsr: rand::thread_rng().gen(),
      short: false,
      length: 0,
      duration: 0,
      length_enabled: false,
      volume_envelope: VolumeEnvelope::new()
    }
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    0
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF20 => self.length = (value & 0b0001_1111) as u16,
      0xFF21 => self.volume_envelope.write_byte(value),
      0xFF22 => {
        self.short = value & 0b0000_1000 == 0b0000_1000;
        let divider = match value & 0b0000_0111 {
          0 => 8,
          n => n as usize * 16
        };
        self.period = divider << (value >> 4);
      },
      0xFF23 => {
        self.length_enabled = value & 0b0100_0000 == 0b0100_0000;

        if value & 0b1000_0000 == 0b1000_0000 {
          self.enabled = true;
          self.duration = self.length;
          self.volume_envelope.reset();
        }
      },
      _ => ()
    }
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.counter += ticks;

    while self.counter >= self.period {
      self.counter -= self.period;

      let bit = self.lfsr & 1 ^ (self.lfsr >> 1) & 1;
      self.lfsr = self.lfsr >> 1 | bit << 15;
      if self.short {
        self.lfsr |= bit << 6;
      }
    }
  }

  pub fn get_sample(&self) -> i16 {
    if self.enabled {
      if self.lfsr & 1 == 1 {
        -self.volume_envelope.get_volume()
      } else {
        self.volume_envelope.get_volume()
      }
    } else {
      0
    }
  }

  pub fn timer_step(&mut self) {
    if self.enabled && self.length_enabled {
      self.duration -= 1;
      self.enabled = self.duration > 0;
    }
  }

  pub fn envelope_step(&mut self) {
    self.volume_envelope.step();
  }
}