use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::video::Window;

use crate::cpu::mmu::gpu::SCREEN_HEIGHT;
use crate::cpu::mmu::gpu::SCREEN_WIDTH;

pub struct Display {
  canvas: Canvas<Window>,
}

impl Display {
  pub fn new(sdl: &Sdl, width: u32, height: u32) -> Display {
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem.window("rustboy", width, height)
      .position_centered()
      .build()
      .expect("Failed to create the main window!");

    let mut canvas = window.into_canvas().build().expect("Failed to place a canvas in the window!");
    canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).expect("Failed to set the logical size of the canvas!");

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    Display {
      canvas,
    }
  }

  pub fn draw_screen(&mut self, screen_buffer: &[[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
    self.canvas.set_draw_color(Color::RGB(0, 0, 0));
    self.canvas.clear();

    for (y, line) in screen_buffer.iter().enumerate() {
      for (x, pixel) in line.iter().enumerate() {
          self.canvas.set_draw_color(map_color(*pixel));
          self.canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
      }
    }

    self.canvas.present();
  }
}

fn map_color(color: u8) -> Color {
  match color {
    0 => Color::RGB( 0x00, 0x00, 0x00),
    1 => Color::RGB( 0x67, 0x67, 0x67),
    2 => Color::RGB( 0xB6, 0xB6, 0xB6),
    _ => Color::RGB( 0xFF, 0xFF, 0xFF)
  }
}