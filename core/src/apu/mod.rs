mod tone;
mod noise;
mod wave;

use crate::CPU_FREQUENCY;
use crate::AUDIO_OUTPUT_FREQUENCY;
use crate::AUDIO_BUFFER_SIZE;
use crate::apu::wave::Wave;
use crate::apu::tone::Tone;
use crate::apu::noise::Noise;

use std::sync::mpsc::Sender;
use std::thread::park;

const SAMPLE_TICKS: usize = CPU_FREQUENCY / AUDIO_OUTPUT_FREQUENCY;
const TIMER_TICKS: usize = CPU_FREQUENCY / 512; //timer clock is at 512hz

pub struct Apu {
  enabled: bool,
  audio_sender: Sender<Vec<i16>>,
  counter: usize,
  buffer: Vec<i16>,
  timer_counter: usize,
  timer_step: usize,

  channel_1: Tone,
  channel_2: Tone,
  channel_3: Wave,
  channel_4: Noise,

  mixer: Mixer
}

impl Apu {
  pub fn new(audio_sender: Sender<Vec<i16>>) -> Apu {
    Apu {
      enabled: true,
      audio_sender,
      counter: 0,
      buffer: vec![],
      timer_counter: 0,
      timer_step: 0,
      channel_1: Tone::new(),
      channel_2: Tone::new(),
      channel_3: Wave::new(),
      channel_4: Noise::new(),
      mixer: Mixer::new()
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0xFF26 => {
        let mut ret = 0;
        if self.enabled { ret |= 0b1000_0000; }
        if self.channel_4.is_enabled() { ret |= 0b0000_1000; }
        if self.channel_3.is_enabled() { ret |= 0b0000_0100; }
        if self.channel_2.is_enabled() { ret |= 0b0000_0010; }
        if self.channel_1.is_enabled() { ret |= 0b0000_0001; }
        ret
      },
      _ => 0
    }
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF10 ... 0xFF14 => self.channel_1.write_byte(address, value),
      0xFF16 ... 0xFF19 => self.channel_2.write_byte(address, value),
      0xFF1A ... 0xFF1E => self.channel_3.write_byte(address, value),
      0xFF20 ... 0xFF23 => self.channel_4.write_byte(address, value),
      0xFF24 ... 0xFF25 => self.mixer.write_byte(address, value),
      0xFF26 => self.set_enabled(value & 0b1000_0000 == 0b1000_0000),
      0xFF30 ... 0xFF3F => self.channel_3.write_byte(address, value),
      _ => ()
    }
  }

  fn set_enabled(&mut self, play: bool) {
    self.enabled = play;
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.do_timer(ticks);

    self.counter += ticks;

    self.channel_1.do_ticks(ticks);
    self.channel_2.do_ticks(ticks);
    self.channel_3.do_ticks(ticks);
    self.channel_4.do_ticks(ticks);

    while self.counter >= SAMPLE_TICKS {
      self.counter -= SAMPLE_TICKS;

      let ch1 = self.channel_1.get_sample();
      let ch2 = self.channel_2.get_sample();
      let ch3 = self.channel_3.get_sample();
      let ch4 = self.channel_4.get_sample();

      let (left, right) = self.mixer.mix(ch1, ch2, ch3, ch4);

      self.buffer.push(left);
      self.buffer.push(right);

      if self.buffer.len() >= AUDIO_BUFFER_SIZE {
        self.audio_sender.send(self.buffer.clone()).expect("Failed to send audio buffer");
        self.buffer.clear();
        park(); //wait until the main thread tells us to continue
      }
    }
  }

  fn do_timer(&mut self, ticks: usize) {
    self.timer_counter += ticks;

    while self.timer_counter >= TIMER_TICKS {
      self.timer_counter -= TIMER_TICKS;

      self.timer_step = ( self.timer_step + 1) % 8;

      if self.timer_step % 2 == 0 {
        self.channel_1.timer_step();
        self.channel_2.timer_step();
        self.channel_3.timer_step();
        self.channel_4.timer_step();
      }

      if self.timer_step == 2 || self.timer_step == 6 {
        self.channel_1.sweep_step();
      }

      if self.timer_step == 7 {
        self.channel_1.envelope_step();
        self.channel_2.envelope_step();
        self.channel_4.envelope_step();
      }
    }

  }
}

struct VolumeEnvelope {
  volume: i16,
  initial_volume: i16,
  counter: usize,
  period: usize,
  increase: bool,
}

impl VolumeEnvelope {
  pub fn new() -> VolumeEnvelope {
    VolumeEnvelope {
      volume: 0,
      initial_volume: 0,
      counter: 0,
      period: 0,
      increase: false
    }
  }

  pub fn write_byte(&mut self, value: u8) {
    self.initial_volume = ((value & 0xF0) >> 4) as i16;
    self.increase = value & 0b0000_1000 == 0b0000_1000;
    self.period = (value & 0b0000_0111) as usize;
  }

  pub fn get_volume(&self) -> i16 {
    self.volume
  }

  pub fn reset(&mut self) {
    self.counter = 0;
    self.volume = self.initial_volume;
  }

  pub fn step(&mut self) {
    if self.period > 0 {
      self.counter += 1;

      if self.counter >= self.period {
        self.counter = 0;

        if self.increase && self.volume < 15 {
          self.volume += 1;
        } else if !self.increase && self.volume > 0 {
          self.volume -= 1;
        }
      }
    }
  }
}

struct Mixer {
  vol_left: i16,
  vol_right: i16,

  ch4_l: bool,
  ch3_l: bool,
  ch2_l: bool,
  ch1_l: bool,
  ch4_r: bool,
  ch3_r: bool,
  ch2_r: bool,
  ch1_r: bool,
}

impl Mixer {
  pub fn new() -> Mixer {
    Mixer {
      vol_left: 0,
      vol_right: 0,

      ch4_l: false,
      ch3_l: false,
      ch2_l: false,
      ch1_l: false,
      ch4_r: false,
      ch3_r: false,
      ch2_r: false,
      ch1_r: false,
    }
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF24 => { //ignoring vin
        self.vol_left = ((value & 0b0111_0000) >> 4) as i16;
        self.vol_right = (value & 0b0000_0111) as i16;
      },
      0xFF25 => {
        self.ch4_l = value & 0b1000_0000 == 0b1000_0000;
        self.ch3_l = value & 0b0100_0000 == 0b0100_0000;
        self.ch2_l = value & 0b0010_0000 == 0b0010_0000;
        self.ch1_l = value & 0b0001_0000 == 0b0001_0000;
        self.ch4_r = value & 0b0000_1000 == 0b0000_1000;
        self.ch3_r = value & 0b0000_0100 == 0b0000_0100;
        self.ch2_r = value & 0b0000_0010 == 0b0000_0010;
        self.ch1_r = value & 0b0000_0001 == 0b0000_0001;
      },
      _ => ()
    }
  }

  pub fn mix(&self, ch1: i16, ch2: i16, ch3: i16, ch4: i16) -> (i16,i16) {

    let mut left = if self.ch1_l { ch1 } else { 0 };
    let mut right = if self.ch1_r { ch1 } else { 0 };

    if self.ch2_l { left += ch2 };
    if self.ch2_r { right += ch2 };

    if self.ch3_l { left += ch3 };
    if self.ch3_r { right += ch3 };

    if self.ch4_l { left += ch4 };
    if self.ch4_r { right += ch4 };

    (self.vol_left * left, self.vol_right * right)
  }
}