pub struct Timer {
  pub irq_timer: bool,

  divide: u8,
  divide_ticks: usize,

  timer_counter: u8,
  timer_ticks: usize,
  timer_modulo: u8,
  timer_enabled: bool,
  timer_steps: usize
}

impl Timer {
  pub fn new() -> Timer {
    Timer {
      irq_timer: false,

      divide: 0,
      divide_ticks: 0,

      timer_counter: 0,
      timer_ticks: 0,
      timer_modulo: 0,
      timer_enabled: false,
      timer_steps: 1024
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0xFF04 => self.divide,
      0xFF05 => self.timer_counter,
      0xFF06 => self.timer_modulo,
      0xFF07 => {
        (if self.timer_enabled { 0x04 } else { 0x0 }) |
        (match self.timer_steps {
          1024 => 0x00,
          16 => 0x01,
          64 => 0x02,
          _ => 0x03
        })
      },
      _ => { println!("Read at unmapped timer address: {:#06X}", address); 0x00 }
    }
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0xFF04 => self.divide = 0x00, //reset on any write
      0xFF05 => self.timer_counter = value,
      0xFF06 => self.timer_modulo = value,
      0xFF07 => {
        self.timer_enabled = (value & 0x04) == 0x04; //bit 2 is enabled yes/no
        self.timer_steps = match value & 0x03 {
          0 => 1024,
          1 => 16,
          2 => 64,
          _ => 256
        }; // the two lowest bits are the mode
      },
      _ => { println!("Write to unmapped timer address: {:#06X}", address); }
    }
  }

  /*00: CPU Clock / 1024 (DMG, CGB:   4096 Hz, SGB:   ~4194 Hz)
            01: CPU Clock / 16   (DMG, CGB: 262144 Hz, SGB: ~268400 Hz)
            10: CPU Clock / 64   (DMG, CGB:  65536 Hz, SGB:  ~67110 Hz)
            11: CPU Clock / 256  (DMG, CGB:  16384 Hz, SGB:  ~16780 Hz)
            */
  pub fn do_ticks(&mut self, ticks: usize) {
    self.divide_ticks += ticks;
    while self.divide_ticks >= 256 {
      self.divide_ticks -= 256;
      self.divide = if self.divide == 0xFF { 0x00 } else { self.divide + 1 };
    }

    if self.timer_enabled {
      self.timer_ticks += ticks;
      while self.timer_ticks >= self.timer_steps {
        self.timer_ticks -= self.timer_steps;
        self.timer_counter = if self.timer_counter == 0xFF {
          self.irq_timer = true;
          self.timer_modulo
        } else {
          self.timer_counter + 1
        };
      }
    }
  }
}