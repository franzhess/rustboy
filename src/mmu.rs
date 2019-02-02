use std::str;

const TITLE_START: usize = 0x0134;
const TITLE_END: usize = 0x0142;

pub struct MMU {
  buffer: [u8; 0xFFFF]
}

impl MMU {
  pub fn new(buffer: [u8; 0xFFFF]) -> MMU {
    println!("Title: {}", str::from_utf8(&buffer[TITLE_START..TITLE_END]).unwrap());

    MMU {
      buffer
    }
  }

  pub fn read_byte(&self, index: u16) -> u8 {
    self.buffer[index as usize]
  }

  pub fn read_word(&self, index: u16) -> u16 {
    self.buffer[index as usize] as u16 | (self.buffer[index as usize + 1] as u16) << 8
  }
}

/*
    buffer[0xFF05] = 0x00; // TIMA
    buffer[0xFF06] = 0x00; // TMA
    buffer[0xFF07] = 0x00; // TAC
    buffer[0xFF10] = 0x80; // NR10
    buffer[0xFF11] = 0xBF; // NR11
    buffer[0xFF12] = 0xF3; // NR12
    buffer[0xFF14] = 0xBF; // NR14
    buffer[0xFF16] = 0x3F; // NR21
    buffer[0xFF17] = 0x00; // NR22
    buffer[0xFF19] = 0xBF; // NR24
    buffer[0xFF1A] = 0x7F; // NR30
    buffer[0xFF1B] = 0xFF; // NR31
    buffer[0xFF1C] = 0x9F; // NR32
    buffer[0xFF1E] = 0xBF; // NR33
    buffer[0xFF20] = 0xFF; // NR41
    buffer[0xFF21] = 0x00; // NR42
    buffer[0xFF22] = 0x00; // NR43
    buffer[0xFF23] = 0xBF; // NR30
    buffer[0xFF24] = 0x77; // NR50
    buffer[0xFF25] = 0xF3; // NR51
    buffer[0xFF26] = 0xF1; //-GB, 0xF0-SGB // NR52
    buffer[0xFF40] = 0x91; // LCDC
    buffer[0xFF42] = 0x00; // SCY
    buffer[0xFF43] = 0x00; // SCx
    buffer[0xFF45] = 0x00; // LYC
    buffer[0xFF47] = 0xFC; // BGP
    buffer[0xFF48] = 0xFF; // OBP0
    buffer[0xFF49] = 0xFF; // OBP1
    buffer[0xFF4A] = 0x00; // WY
    buffer[0xFF4B] = 0x00; // Wx
    buffer[0xFFFF] = 0x00; // IE
*/