const WAVE_PATTERN: [[i16;8];4] = [[-1,1,1,1,1,1,1,1],[-1,-1,1,1,1,1,1,1],[-1,-1,-1,-1,1,1,1,1],[-1,-1,-1,-1,-1,-1,1,1]];

pub struct Tone {
  enabled: bool,
  duty: usize,
  length_enabled: bool,
  length: usize, //Sound Length = (64-t1)*(1/256) seconds
  duration: usize,
  frequency: u16, //Frequency = 131072/(2048-x) Hz
  counter: usize,
  phase: usize,   //which position in the waveform array
  period: usize, //ticks per period
}

impl Tone {
  pub fn new() -> Tone {
    Tone {
      enabled: false,
      duty: 2,
      length_enabled: false,
      length: 0,
      duration: 0,
      frequency: 2048,
      counter: 0,
      phase: 0,
      period: 1,
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
      0xFF11 | 0xFF16 => {
        self.length = 64 - (value & 0b0011_1111) as usize;
        self.duty = ((value & 0b1100_0000) >> 6) as usize;
      },
      0xFF13 | 0xFF18 => {
        self.frequency = (self.frequency & 0xFF00) | value as u16;
        self.period = Tone::frequency_to_period(self.frequency);
        self.duration = self.length;
      },
      0xFF14 | 0xFF19 => {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.period = Tone::frequency_to_period(self.frequency);
        self.length_enabled = value & 0b0100_0000 == 0b0100_0000;

        if value & 0b1000_0000 == 0b1000_0000 {
          self.enabled = true;
          self.duration = self.length;
        }
      },
      _ => ()
    }
  }

  pub fn timer_step(&mut self) {
    if self.enabled && self.length_enabled {
      self.duration -= 1;
      self.enabled = self.duration > 0;
    }
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.counter += ticks;

    while self.counter >= self.period {
      self.counter -= self.period;
      self.phase = (self.phase + 1) % 8;
    }
  }

  pub fn get_sample(&self) -> i16 {
    if self.enabled {
      WAVE_PATTERN[self.duty][self.phase]
    } else {
      0
    }
  }

  fn frequency_to_period(frequency: u16) -> usize {
    if frequency >= 2048 { 1 } else { (2048 - frequency as usize) * 4 }
  }
}