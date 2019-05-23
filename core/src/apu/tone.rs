use crate::apu::VolumeEnvelope;

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
  volume_envelope: VolumeEnvelope,
  sweep: Sweep
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
      volume_envelope: VolumeEnvelope::new(),
      sweep: Sweep::new()
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
      0xFF10 => self.sweep.write_byte(value),
      0xFF11 | 0xFF16 => {
        self.length = 64 - (value & 0b0011_1111) as usize;
        self.duty = ((value & 0b1100_0000) >> 6) as usize;
      },
      0xFF12 | 0xFF17 => self.volume_envelope.write_byte(value),
      0xFF13 | 0xFF18 => {
        self.frequency = (self.frequency & 0xFF00) | value as u16;
        self.update_period();
        self.duration = self.length;
      },
      0xFF14 | 0xFF19 => {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.update_period();
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

  pub fn timer_step(&mut self) {
    if self.enabled && self.length_enabled {
      self.duration -= 1;
      self.enabled = self.duration > 0;
    }
  }

  pub fn envelope_step(&mut self) {
    self.volume_envelope.step();
  }

  pub fn sweep_step(&mut self) {
      self.frequency = self.sweep.get_frequency(self.frequency);
      self.update_period();
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
      self.volume_envelope.get_volume() * WAVE_PATTERN[self.duty][self.phase]
    } else {
      0
    }
  }

  fn update_period(&mut self) {
    self.period = Tone::frequency_to_period(self.frequency);
  }

  fn frequency_to_period(frequency: u16) -> usize {
    if frequency >= 2048 { 1 } else { (2048 - frequency as usize) * 4 }
  }
}

struct Sweep {
  counter: usize,
  period: usize,
  subtraction: bool,
  shift: usize
}

impl Sweep {
  pub fn new() -> Sweep {
    Sweep {
      counter: 0,
      period: 0,
      subtraction: false,
      shift: 0
    }
  }

  pub fn write_byte(&mut self, value: u8) {
    self.period = ((value & 0b0111_0000) as usize) >> 4;
    self.subtraction = value & 0b0000_1000 == 0b0000_1000;
    self.shift = (value & 0b0000_0111) as usize;
  }

  pub fn get_frequency(&mut self, frequency: u16) -> u16 {
    if self.period > 0 {
      self.counter += 1;

      if self.counter >= self.period {
        let offset = frequency >> self.shift;
        let new_frequency = if self.subtraction {  //X(t) = X(t-1) +/- X(t-1)/2^n
          frequency - offset
        } else {
          frequency + offset
        };

        if new_frequency < 0 {
          return 0
        } else if new_frequency > 2048 {
          return 2048
        } else {
          return new_frequency
        }
      }
    }
    frequency
  }
}