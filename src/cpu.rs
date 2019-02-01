pub const SCREEN_BUFFER_WIDTH: usize = 160;
pub const SCREEN_BUFFER_HEIGHT: usize = 144;

pub struct TickResult {
  pub screen_changed: bool,
}

pub struct Cpu {
  vram: [[u8; SCREEN_BUFFER_WIDTH];SCREEN_BUFFER_HEIGHT]

}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      vram: [[0;SCREEN_BUFFER_WIDTH];SCREEN_BUFFER_HEIGHT]
    }
  }

  pub fn tick(&mut self) -> TickResult {

    TickResult {
      screen_changed: false,
    }
  }

  pub fn get_screen_buffer_ref(&self) -> &[[u8; SCREEN_BUFFER_WIDTH]; SCREEN_BUFFER_HEIGHT] {
    &self.vram
  }
}