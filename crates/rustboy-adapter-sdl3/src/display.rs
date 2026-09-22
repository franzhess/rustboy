use crate::InitError;
use ::sdl3::pixels::Color;
use ::sdl3::rect::Point;
use ::sdl3::render::WindowCanvas;
use ::sdl3::sys::render::SDL_LOGICAL_PRESENTATION_INTEGER_SCALE;
use ::sdl3::Sdl;
use rustboy_core::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Display {
    canvas: WindowCanvas,
}

impl Display {
    pub fn new(sdl: &Sdl, width: u32, height: u32) -> Result<Self, InitError> {
        let video = sdl.video()?;
        let window = video
            .window("rustboy", width, height)
            .position_centered()
            .build()
            .map_err(InitError::Window)?;
        let mut canvas = window.into_canvas();
        canvas
            .set_logical_size(
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
                SDL_LOGICAL_PRESENTATION_INTEGER_SCALE,
            )
            .map_err(InitError::LogicalSize)?;
        canvas.set_draw_color(Color::RGB(0x08, 0x18, 0x20));
        canvas.clear();
        canvas.present();
        Ok(Self { canvas })
    }

    pub fn draw_screen(&mut self, screen_buffer: Vec<u8>) -> Result<(), sdl3::Error> {
        self.canvas.set_draw_color(Color::RGB(0x08, 0x18, 0x20));
        self.canvas.clear();
        for (i, pixel) in screen_buffer.iter().enumerate() {
            self.canvas.set_draw_color(map_color(*pixel));
            self.canvas.draw_point(Point::new(
                (i % SCREEN_WIDTH) as i32,
                (i / SCREEN_WIDTH) as i32,
            ))?;
        }
        self.canvas.present();
        Ok(())
    }
}

fn map_color(color: u8) -> Color {
    match color {
        0 => Color::RGB(0xE0, 0xF8, 0xD0),
        1 => Color::RGB(0x88, 0xC0, 0x70),
        2 => Color::RGB(0x34, 0x68, 0x56),
        _ => Color::RGB(0x08, 0x18, 0x20),
    }
}
