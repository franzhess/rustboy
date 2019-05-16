mod square;
mod noise;
mod wave;

use crate::CPU_FREQUENCY;
use crate::AUDIO_OUTPUT_FREQUENCY;
use std::f64;
use std::sync::mpsc::Sender;
use std::thread::park;
use crate::AUDIO_BUFFER_SIZE;
use crate::apu::wave::Wave;
use crate::apu::square::Tone;
use crate::apu::noise::Noise;

const FRAME_TICKS: usize = ((AUDIO_BUFFER_SIZE as f64 / AUDIO_OUTPUT_FREQUENCY as f64) * CPU_FREQUENCY as f64) as usize;

pub struct Apu {
  audio_sender: Sender<Vec<i16>>,
  counter: usize,

  channel_3: Wave,

  /*channel_1: Tone,
  channel_2: Tone,
  channel_4: Noise*/
}

impl Apu {
  pub fn new(audio_sender: Sender<Vec<i16>>) -> Apu {
    Apu {
      audio_sender,
      counter: 0,
      channel_3: Wave::new()
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    0
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF30 ... 0xFF3F => self.channel_3.write_byte(address, value),
      _ => ()
    }
  }

  pub fn do_ticks(&mut self, ticks: usize) {
    self.counter += ticks;

    while self.counter >= FRAME_TICKS {
      self.generate_frame();
      self.counter -= FRAME_TICKS;
    }
  }

  fn generate_frame(&mut self) {
    self.channel_3.run();

    let mut buffer = vec![];
    for x in 0 .. AUDIO_BUFFER_SIZE {
      buffer.push(self.channel_3.sound_buffer[x]); //mono
    }
    self.audio_sender.send(buffer).expect("Failed to send audio data!");
    park();
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