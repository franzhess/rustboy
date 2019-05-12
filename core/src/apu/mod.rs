mod square;
mod noise;
mod wave;

use crate::CPU_FREQUENCY;
use blip_buf::BlipBuf;
use crate::AUDIO_OUTPUT_FREQUENCY;
use std::f64;
use sample::signal::Signal;
use sample::signal::ConstHz;
use sample::signal::Sine;
use sample::signal::Square;

const FRAME_SIZE: usize = AUDIO_OUTPUT_FREQUENCY / 60;
const FRAME_TICKS: usize = ((FRAME_SIZE as f64 / AUDIO_OUTPUT_FREQUENCY as f64) * CPU_FREQUENCY as f64) as usize;
const SAMPLE_RATE: usize = CPU_FREQUENCY / AUDIO_OUTPUT_FREQUENCY;

pub struct Apu {
  signal: Square<ConstHz>,
  audio_updated: bool,
  last_amp: i32,
  counter: usize,
  buffer: [i16; FRAME_SIZE]
}

impl Apu {
  pub fn new() -> Apu {
    let mut signal = sample::signal::rate(AUDIO_OUTPUT_FREQUENCY as f64).const_hz(800 as f64).square();

    Apu {
      signal,
      audio_updated: false,
      last_amp: 0,
      counter: 0,
      buffer: [0; FRAME_SIZE]
    }
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.counter += ticks;

    while self.counter >= FRAME_TICKS {
      self.generate_frame();
      self.counter -= FRAME_TICKS;
      self.audio_updated = true;
    }
  }

  fn generate_frame(&mut self) {
    for x in 0 .. FRAME_SIZE {
      self.buffer[x] = (1000f64 * self.signal.next()[0]) as i16;
    }
  }

  pub fn is_audio_updated(&mut self) -> bool {
    if self.audio_updated {
      self.audio_updated = false;
      true
    } else {
      false
    }
  }

  pub fn get_sound_buffer(&mut self) -> Vec<i16> {
    self.buffer.to_vec()
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