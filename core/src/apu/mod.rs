mod tone;
mod noise;
mod wave;

use crate::CPU_FREQUENCY;
use crate::AUDIO_OUTPUT_FREQUENCY;
use std::f64;
use std::sync::mpsc::Sender;
use std::thread::park;
use crate::AUDIO_BUFFER_SIZE;
use crate::apu::wave::Wave;
use crate::apu::tone::Tone;
use crate::apu::noise::Noise;

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
  channel_4: Noise
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
      channel_4: Noise::new()
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0xFF26 => {
        let mut ret = 0;
        if self.enabled {
          ret |= 0b1000_0000;
        }
        if self.channel_3.is_enabled() {
          ret |= 0b0000_0100;
        }
        if self.channel_2.is_enabled() {
          ret |= 0b0000_0010;
        }
        if self.channel_1.is_enabled() {
          ret |= 0b0000_0001;
        }
        ret
      },
      _ => 0
    }
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF10 ... 0xFF14 => self.channel_1.write_byte(address, value),
      0xFF16 ... 0xFF19 => self.channel_2.write_byte(address, value),
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

    while self.counter >= SAMPLE_TICKS {
      self.counter -= SAMPLE_TICKS;

      let ch1 = self.channel_1.get_sample();
      let ch2 = self.channel_2.get_sample();
      let ch3 = self.channel_3.get_sample();

      let (left, right) = self.mix(ch1, ch2, ch3);

      self.buffer.push(left);
      //self.buffer.push(right);

      if self.buffer.len() >= AUDIO_BUFFER_SIZE {
        self.audio_sender.send(self.buffer.clone());
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
      }

      if self.timer_step == 2 || self.timer_step == 6 {
        self.channel_1.sweep_step();
      }

      if self.timer_step == 7 {
        self.channel_1.envelope_step();
        self.channel_2.envelope_step();
      }
    }

  }

  fn mix(&self, ch1: i16, ch2: i16, ch3: i16) -> (i16,i16) {
    let amp = (ch1 + ch2 + ch3) * 1000;
    (amp, amp)
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

/*
       Square 1
NR10 FF10 -PPP NSSS Sweep period, negate, shift
NR11 FF11 DDLL LLLL Duty, Length load (64-L)
NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
NR13 FF13 FFFF FFFF Frequency LSB
NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB

       Square 2
     FF15 ---- ---- Not used
NR21 FF16 DDLL LLLL Duty, Length load (64-L)
NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
NR23 FF18 FFFF FFFF Frequency LSB
NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB

       Wave
NR30 FF1A E--- ---- DAC power
NR31 FF1B LLLL LLLL Length load (256-L)
NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
NR33 FF1D FFFF FFFF Frequency LSB
NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB

       Noise
     FF1F ---- ---- Not used
NR41 FF20 --LL LLLL Length load (64-L)
NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
NR44 FF23 TL-- ---- Trigger, Length enable

       Control/Status
NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
NR51 FF25 NW21 NW21 Left enables, Right enables
NR52 FF26 P--- NW21 Power control/status, Channel length statuses

       Not used
     FF27 ---- ----
     .... ---- ----
     FF2F ---- ----

       Wave Table
     FF30 0000 1111 Samples 0 and 1
     ....
     FF3F 0000 1111 Samples 30 and 31
*/