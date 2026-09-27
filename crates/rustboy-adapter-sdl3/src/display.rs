use crate::InitError;
use rustboy_core::{Frame, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{
    BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureCreator, WindowCanvas,
};
use sdl3::sys::render::SDL_LOGICAL_PRESENTATION_INTEGER_SCALE;
use sdl3::video::{Window, WindowContext};
use sdl3::Sdl;

const BYTES_PER_PIXEL: usize = 4;
const PALETTE: [[u8; BYTES_PER_PIXEL]; 4] = [
    [0xE0, 0xF8, 0xD0, 0xFF],
    [0x88, 0xC0, 0x70, 0xFF],
    [0x34, 0x68, 0x56, 0xFF],
    [0x08, 0x18, 0x20, 0xFF],
];

/// Owns the renderer and creator so a connected display can borrow both safely.
pub struct DisplayContext {
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
}

impl DisplayContext {
    pub fn new(sdl: &Sdl, width: u32, height: u32) -> Result<Self, InitError> {
        let video = sdl.video()?;
        let window = video
            .window("rustboy", width, height)
            .position_centered()
            .build()
            .map_err(InitError::Window)?;
        let mut canvas = window.into_canvas();
        configure_canvas(&mut canvas)?;
        let texture_creator = canvas.texture_creator();
        Ok(Self {
            canvas,
            texture_creator,
        })
    }

    pub fn connect(&mut self) -> Result<Display<'_>, InitError> {
        Display::new(&mut self.canvas, &self.texture_creator)
    }
}

// Generic over the target so software-surface tests exercise the same path as windows.
pub struct Display<'a, T: RenderTarget = Window> {
    canvas: &'a mut Canvas<T>,
    texture: Texture<'a>,
}

impl<'a, T: RenderTarget> Display<'a, T> {
    fn new(
        canvas: &'a mut Canvas<T>,
        creator: &'a TextureCreator<T::Context>,
    ) -> Result<Self, InitError> {
        let mut texture = creator
            .create_texture_streaming(
                PixelFormat::RGBA32,
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )
            .map_err(InitError::Texture)?;
        texture.set_scale_mode(ScaleMode::Nearest);
        texture.set_blend_mode(BlendMode::None);
        Ok(Self { canvas, texture })
    }

    pub fn draw_screen(&mut self, frame: &Frame) -> Result<(), sdl3::Error> {
        self.texture
            .with_lock(None, |pixels, pitch| write_pixels(frame, pixels, pitch))?;
        self.canvas.set_draw_color(Color::RGB(0x08, 0x18, 0x20));
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None)?;
        self.canvas.present();
        Ok(())
    }
}

fn configure_canvas<T: RenderTarget>(canvas: &mut Canvas<T>) -> Result<(), InitError> {
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
    Ok(())
}

fn write_pixels(frame: &Frame, pixels: &mut [u8], pitch: usize) {
    for (y, row) in frame.pixels().chunks_exact(SCREEN_WIDTH).enumerate() {
        let start = y * pitch;
        let output = &mut pixels[start..start + SCREEN_WIDTH * BYTES_PER_PIXEL];
        for (shade, pixel) in row.iter().zip(output.chunks_exact_mut(BYTES_PER_PIXEL)) {
            pixel.copy_from_slice(&PALETTE[usize::from(*shade)]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl3::rect::Point;
    use sdl3::surface::Surface;
    use std::time::{Duration, Instant};

    // Preserve the original per-pixel renderer as an independent visual/benchmark reference.
    fn original_color(shade: u8) -> Color {
        match shade {
            0 => Color::RGB(0xE0, 0xF8, 0xD0),
            1 => Color::RGB(0x88, 0xC0, 0x70),
            2 => Color::RGB(0x34, 0x68, 0x56),
            _ => Color::RGB(0x08, 0x18, 0x20),
        }
    }

    fn draw_original<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        frame: &Frame,
    ) -> Result<(), sdl3::Error> {
        canvas.set_draw_color(Color::RGB(0x08, 0x18, 0x20));
        canvas.clear();
        for (i, &shade) in frame.pixels().iter().enumerate() {
            canvas.set_draw_color(original_color(shade));
            canvas.draw_point(Point::new(
                (i % SCREEN_WIDTH) as i32,
                (i / SCREEN_WIDTH) as i32,
            ))?;
        }
        canvas.present();
        Ok(())
    }

    fn patterned_frame(invert: bool) -> Frame {
        Frame::try_from(
            (0..Frame::PIXEL_COUNT)
                .map(|i| {
                    let shade = ((i % SCREEN_WIDTH + 2 * (i / SCREEN_WIDTH)) % 4) as u8;
                    if invert {
                        3 - shade
                    } else {
                        shade
                    }
                })
                .collect::<Vec<_>>(),
        )
        .expect("valid patterned frame")
    }

    fn capture<T: RenderTarget>(canvas: &Canvas<T>) -> Vec<u8> {
        let surface = canvas
            .read_pixels(None)
            .expect("read renderer pixels")
            .convert_format(PixelFormat::RGBA32)
            .expect("RGBA readback");
        let pitch = surface.pitch() as usize;
        let row_bytes = surface.width() as usize * BYTES_PER_PIXEL;
        surface.with_lock(|pixels| {
            pixels
                .chunks_exact(pitch)
                .flat_map(|row| row[..row_bytes].iter().copied())
                .collect()
        })
    }

    #[test]
    fn pixel_conversion_preserves_palette_rows_and_texture_padding() {
        let frame = patterned_frame(false);
        let pitch = SCREEN_WIDTH * BYTES_PER_PIXEL + 12;
        let mut pixels = vec![0xCC; pitch * SCREEN_HEIGHT];
        write_pixels(&frame, &mut pixels, pitch);
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let color = original_color(frame.pixels()[y * SCREEN_WIDTH + x]);
                let start = y * pitch + x * BYTES_PER_PIXEL;
                assert_eq!(&pixels[start..start + 4], &[color.r, color.g, color.b, 255]);
            }
            assert!(
                pixels[y * pitch + SCREEN_WIDTH * BYTES_PER_PIXEL..(y + 1) * pitch]
                    .iter()
                    .all(|byte| *byte == 0xCC)
            );
        }
    }

    #[test]
    fn streaming_texture_matches_original_rendering_at_integer_scales_and_with_letterboxing() {
        // Software surfaces require no window, display server or audio device.
        for (width, height) in [(160, 144), (320, 288), (337, 309)] {
            let surface = Surface::new(width, height, PixelFormat::RGBA32).expect("surface");
            let mut canvas = surface.into_canvas().expect("software renderer");
            configure_canvas(&mut canvas).expect("integer logical size");
            let creator = canvas.texture_creator();
            let mut display = Display::new(&mut canvas, &creator).expect("streaming texture");
            // Change every pixel on the second draw, reusing the same texture.
            for invert in [false, true] {
                let frame = patterned_frame(invert);
                draw_original(display.canvas, &frame).expect("reference rendering");
                let expected = capture(display.canvas);
                display.draw_screen(&frame).expect("texture rendering");
                assert_eq!(
                    capture(display.canvas),
                    expected,
                    "{width}x{height}, invert={invert}"
                );
            }
        }
    }

    #[test]
    #[ignore = "manual rendering benchmark: run in release mode with --ignored --nocapture"]
    fn streaming_texture_rendering_benchmark() {
        const FRAMES: u32 = 100;
        let surface = Surface::new(320, 288, PixelFormat::RGBA32).expect("surface");
        let mut canvas = surface.into_canvas().expect("software renderer");
        configure_canvas(&mut canvas).expect("integer logical size");
        let creator = canvas.texture_creator();
        let mut display = Display::new(&mut canvas, &creator).expect("streaming texture");
        let frame = patterned_frame(false);
        for _ in 0..10 {
            draw_original(display.canvas, &frame).expect("reference rendering");
            display.draw_screen(&frame).expect("texture rendering");
        }
        let measure = |draw: &mut dyn FnMut()| -> Duration {
            let start = Instant::now();
            for _ in 0..FRAMES {
                draw();
            }
            start.elapsed()
        };
        let mut original = Vec::new();
        let mut streaming = Vec::new();
        for round in 0..5 {
            if round % 2 == 0 {
                original.push(measure(&mut || {
                    draw_original(display.canvas, &frame).expect("reference")
                }));
                streaming.push(measure(&mut || {
                    display.draw_screen(&frame).expect("texture")
                }));
            } else {
                streaming.push(measure(&mut || {
                    display.draw_screen(&frame).expect("texture")
                }));
                original.push(measure(&mut || {
                    draw_original(display.canvas, &frame).expect("reference")
                }));
            }
        }
        original.sort();
        streaming.sort();
        let old_us = original[2].as_secs_f64() * 1_000_000.0 / f64::from(FRAMES);
        let new_us = streaming[2].as_secs_f64() * 1_000_000.0 / f64::from(FRAMES);
        println!(
            "SDL software renderer, 320x288, {}-{}, median of 5x{FRAMES} frames: \
            per-pixel {old_us:.1} us/frame; streaming {new_us:.1} us/frame; {:.2}x ratio",
            std::env::consts::OS,
            std::env::consts::ARCH,
            old_us / new_us
        );
    }
}
