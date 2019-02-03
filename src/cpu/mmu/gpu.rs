const VRAM_SIZE: usize = 0x2000; //8kB vram
const VOAM_SIZE: usize = 0xA0;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub struct GPU {
  vram: [u8; VRAM_SIZE],
  voam: [u8; VOAM_SIZE],
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      vram: [0; VRAM_SIZE],
      voam: [0; VOAM_SIZE]
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    println!("reading from vram!");
    0
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    println!("writing to vram!");
  }
}

/* timing
Line (background) 172
Line (sprites) 80
Horizontal blank 204
Single line 456
Vertical blank 4560
Entire frame 70224
*/