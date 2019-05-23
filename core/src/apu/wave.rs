pub struct Wave {
  enabled: bool,
  length: usize,
  duration: usize,
  volume: usize, //shift right
  frequency: u16,
  counter: usize,
  period: usize,
  length_enabled: bool,
  cursor: usize,
  wave_pattern: [i16; 32]
}

impl Wave {
  pub fn new() -> Wave {
    Wave {
      enabled: false,
      length: 0,
      duration: 0,
      wave_pattern: [0; 32],
      volume: 0,
      frequency: 2048,
      counter: 0,
      period: 1,
      cursor: 0,
      length_enabled: false,
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
      0xFF1A => self.enabled = value & 0b1000_000 == 0b1000_0000,
      0xFF1B => self.length = value as usize,
      0xFF1C => self.volume = ((value & 0b0110_0000) as usize) >> 5,
      0xFF1D => {
        self.frequency = (self.frequency & 0xFF00) | value as u16;
        self.update_period();
        self.duration = self.length;
      },
      0xFF1E => {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.update_period();
        self.length_enabled = value & 0b0100_0000 == 0b0100_0000;

        if value & 0b1000_0000 == 0b1000_0000 {
          self.enabled = true;
          self.duration = self.length;
        }
      },
      0xFF30 ... 0xFF3F => {
        let offset = address as usize - 0xFF30;
        self.wave_pattern[offset * 2] = (((value & 0xF0) >> 4) as i16) - 8;
        self.wave_pattern[(offset * 2) + 1] = ((value & 0xF) as i16) - 8;
      },
      _ => ()
    }
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.counter += ticks;

    while self.counter >= self.period {
      self.counter -= self.period;
      self.cursor = (self.cursor + 1) % 32;
    }
  }

  pub fn timer_step(&mut self) {
    if self.enabled && self.length_enabled {
      self.duration -= 1;
      self.enabled = self.duration > 0;
    }
  }

  pub fn get_sample(&self) -> i16 {
    if self.enabled && self.volume > 0 {
      self.wave_pattern[self.cursor] >> self.volume
    } else {
      0
    }
  }

  fn update_period(&mut self) {
    self.period = Wave::frequency_to_period(self.frequency);
  }

  fn frequency_to_period(frequency: u16) -> usize {
    if frequency >= 2048 { 1 } else { (2048 - frequency as usize) * 2 }
  }
}