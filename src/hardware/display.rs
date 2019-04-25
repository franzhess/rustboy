use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::video::Window;

use core::SCREEN_WIDTH;
use core::SCREEN_HEIGHT;

pub struct Display {
  canvas: Canvas<Window>,
}

impl Display {
  pub fn new(sdl: &Sdl, width: u32, height: u32) -> Display {
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem.window("rustboy", width, height)
      //.fullscreen_desktop()
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

  pub fn draw_screen(&mut self, screen_buffer: Vec<u8>) {
    self.canvas.set_draw_color(Color::RGB(0, 0, 0));
    self.canvas.clear();

    for (i, pixel) in screen_buffer.iter().enumerate() {
          self.canvas.set_draw_color(map_color(*pixel));
          self.canvas.draw_point(Point::new((i % SCREEN_WIDTH) as i32, (i / SCREEN_WIDTH) as i32)).unwrap();
    }

    self.canvas.present();
  }
}

fn map_color(color: u8) -> Color {
  match color {
    0 => Color::RGB( 0xE0, 0xF8, 0xD0),
    1 => Color::RGB( 0x88, 0xC0, 0x70),
    2 => Color::RGB( 0x34, 0x68, 0x56),
    _ => Color::RGB( 0x08, 0x18, 0x20)
  }
}