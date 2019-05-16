use crate::AUDIO_BUFFER_SIZE;

pub struct Wave {
  enabled: bool,
  pub sound_buffer: [i16; AUDIO_BUFFER_SIZE],
  sound_length: u8,
  wave_pattern: [u8; 16]
}

impl Wave {
  pub fn new() -> Wave {
    Wave {
      enabled: false,
      sound_buffer: [0; AUDIO_BUFFER_SIZE],
      sound_length: 0,
      wave_pattern: [0; 16],
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    0
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF1A => self.enabled = value & 0b1000_000 == 0b1000_0000,
      0xFF1B => self.sound_length = value,
      0xFF30 ... 0xFF3F => self.wave_pattern[address as usize - 0xFF30] = value,
      _ => ()
    }
  }

  pub fn run(&mut self) {
    self.sound_buffer = [0; AUDIO_BUFFER_SIZE];

    if self.enabled {

    }
  }
}