const VRAM_SIZE: usize = 0x2000; //8kB vram
pub const VOAM_SIZE: usize = 0xA0;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub struct GPU {
  pub screen_buffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
  pub irq_vblank: bool,
  pub irq_stat: bool,

  clock: usize,
  vram: [u8; VRAM_SIZE],
  voam: [u8; VOAM_SIZE],
  lcd_enabled: bool, //FF40
  window_tilemap_select: bool, //FF40 - false = 9800-9BFF / true = 9C00-9FFF
  window_enable: bool, //FF40
  bg_window_tile_data_select: bool, //FF40 - false = 8800-97FF / true = 8000-8FFF
  bg_tilemap_select: bool, //FF40 false = 9800-9BFF / true = 9C00-9FFF
  sprite_size: bool, //FF40 false = 8x8 / true = 8x16
  sprite_enable: bool, //FF40
  bg_window_priority: bool, //FF40
  mode: u8, //Mode 0,1,2,3 FF41
  irq_m0_enable: bool, //sets what triggers the stat interrupt FF41
  irq_m1_enable: bool,
  irq_m2_enable: bool,
  irq_lyc_enable: bool,
  scroll_y: u8, //SCY FF42
  scroll_x: u8, //SCX FF43
  line: u8, //LY FF44 current line drawn by the display controller
  line_compare: u8, //LYC FF45 compare value for LYC
  bg_palette: u8, //BGP FF47
  obj_palette_1: u8, //OBP0 FF48
  obj_palette_2: u8, //OBP1 FF49
  window_y: u8, //WY FF4A 0
  window_x: u8, //WX FF4B 7
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      screen_buffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
      irq_vblank: false,
      irq_stat: false,
      clock: 0, // for the first line
      vram: [0; VRAM_SIZE],
      voam: [0; VOAM_SIZE],
      lcd_enabled: false,
      window_tilemap_select: false,
      window_enable: false,
      bg_window_tile_data_select: false,
      bg_tilemap_select: false,
      sprite_size: false,
      sprite_enable: false,
      bg_window_priority: false,
      mode: 0x00,
      irq_m0_enable: false,
      irq_m1_enable: false,
      irq_m2_enable: false,
      irq_lyc_enable: false,
      scroll_y: 0x00,
      scroll_x: 0x00,
      line: 0x00,
      line_compare: 0x00,
      bg_palette: 0xFC,
      obj_palette_1: 0xFF,
      obj_palette_2: 0xFF,
      window_y: 0x00,
      window_x: 0x00,
    }
  }

  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0x8000...0x9FFF => { let offset = address as usize - 0x8000; self.vram[offset] },
      0xFE00...0xFE9F => { let offset = address as usize - 0xFE00; self.voam[offset] },
      0xFF40 => { // LCD Control
        (if self.lcd_enabled { 0x80 } else { 0x00 }) |
        (if self.window_tilemap_select { 0x40 } else { 0x00 }) |
        (if self.window_enable { 0x20 } else { 0x00 }) |
        (if self.bg_window_tile_data_select { 0x10 } else { 0x00 }) |
        (if self.bg_tilemap_select { 0x08 } else { 0x00 }) |
        (if self.sprite_size { 0x04 } else { 0x00 }) |
        (if self.sprite_enable { 0x02 } else { 0x00 }) |
        (if self.bg_window_priority { 0x01 } else { 0x00 })
      },
      0xFF41 => { // LCD Status
        (if self.irq_lyc_enable { 0x40 } else { 0x00 }) |
        (if self.irq_m2_enable { 0x20 } else { 0x00 }) |
        (if self.irq_m1_enable { 0x10 } else { 0x00 }) |
        (if self.irq_m0_enable { 0x08 } else { 0x00 }) |
        (if self.line == self.line_compare { 0x04 } else { 0x00 }) |
        self.mode
      },
      0xFF42 => self.scroll_y,
      0xFF43 => self.scroll_x,
      0xFF44 => self.line,
      0xFF45 => self.line_compare,
      0xFF47 => self.bg_palette,
      0xFF48 => self.obj_palette_1,
      0xFF49 => self.obj_palette_2,
      0xFF4A => self.window_y,
      0xFF4B => self.window_x,
      _ => panic!("Invalid read at GPU memory adress: {:#06X}", address)
    }
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0x8000...0x9FFF => { let offset = address as usize - 0x8000; self.vram[offset] = value; },
      0xFE00...0xFE9F => { let offset = address as usize - 0xFE00; self.voam[offset] = value; },
      0xFF40 => {
        self.lcd_enabled = value & 0x80 == 0x80;
        self.window_tilemap_select = value & 0x40 == 0x40;
        self.window_enable = value & 0x20 == 0x20;
        self.bg_window_tile_data_select = value & 0x10 == 0x10;
        self.bg_tilemap_select = value & 0x08 == 0x08;
        self.sprite_size = value & 0x04 == 0x04;
        self.sprite_enable = value & 0x02 == 0x02;
        self.bg_window_priority = value & 0x01 == 0x01;
      },
      0xFF41 => {
        self.irq_lyc_enable = value & 0x40 == 0x40;
        self.irq_m2_enable = value & 0x20 == 0x20;
        self.irq_m1_enable = value & 0x10 == 0x10;
        self.irq_m0_enable = value & 0x08 == 0x08;
      }
      0xFF42 => self.scroll_y = value,
      0xFF43 => self.scroll_x = value,
      0xFF45 => self.line_compare = value,
      0xFF47 => self.bg_palette = value,
      0xFF48 => self.obj_palette_1 = value,
      0xFF49 => self.obj_palette_2 = value,
      0xFF4A => self.window_y = value,
      0xFF4B => self.window_x = value,
      _ => panic!("Invalid write at GPU memory adress: {:#06X} - {:#06X}", address, value)
    }
  }

  /* timing
    Line (background) 172
    Line (sprites) 80
    Horizontal blank 204
    Single line 456
    Vertical blank 4560
    Entire frame 70224 */
  pub fn do_ticks(&mut self, ticks: usize) {
    if self.lcd_enabled {
      self.clock += ticks;

      while self.clock >= 456 { //advance one line
        self.clock -= 456;
        self.line = (self.line + 1) % 154;

        if self.line > 144 && self.mode != 1 {
          self.set_mode(1);
        }
      }

      if self.clock <= 80 {
        if self.mode != 2 { self.set_mode(2); }
      } else if self.clock <= 80 + 172 {
        if self.mode != 3 { self.set_mode(3); }
      } else {
        if self.mode != 0 { self.set_mode(0); }
      }
    } else { //when the lcd is disabled line and mode are reset
      self.line = 0;
      self.mode = 0;
    }
  }

  /*
 Mode 0: The LCD controller is in the H-Blank period and
         the CPU can access both the display RAM (8000h-9FFFh)
         and OAM (FE00h-FE9Fh)

 Mode 1: The LCD controller is in the V-Blank period (or the
         display is disabled) and the CPU can access both the
         display RAM (8000h-9FFFh) and OAM (FE00h-FE9Fh)

 Mode 2: The LCD controller is reading from OAM memory.
         The CPU <cannot> access OAM memory (FE00h-FE9Fh)
         during this period.

 Mode 3: The LCD controller is reading from both OAM and VRAM,
         The CPU <cannot> access OAM and VRAM during this period.
         CGB Mode: Cannot access Palette Data (FF69,FF6B) either.
*/
  fn set_mode(&mut self, mode: u8) {
    self.mode = mode;

    //@TODO do rendering stuff
    match mode {
      0 => (), //@TODO render line here
      1 => { self.render_frame(); self.irq_vblank = true }, //@TODO when rendering lines remove render frame
      _ => ()
    }
  }

  fn render_frame(&mut self) {
    for x in 0..SCREEN_WIDTH {
      for y in 0..SCREEN_HEIGHT {
        self.screen_buffer[y][x] = ((x + y) % 4) as u8;
      }
    }

    self.render_background();
    self.render_sprites();
  }

  fn render_background(&mut self) {

  }

  fn render_sprites(&mut self) {
    if self.sprite_enable {
      for sprite_num in 0..40 {
        let sprite_address = 0xFE00u16 + (39 - sprite_num) * 4;
        println!("{:#06X}", sprite_address);
      }
    }
  }
}