use crate::Frame;
use crate::SCREEN_HEIGHT;
use crate::SCREEN_WIDTH;

pub const VRAM_SIZE: usize = 0x2000; //8kB vram
pub const OAM_SIZE: usize = 0xA0; // 40 sprites * 4 attribute bytes.
const DOTS_PER_LINE: usize = 456;
const OAM_SEARCH_END: usize = 80;
const PIXEL_TRANSFER_END: usize = 252;
const LY_ADVANCE_DOT: usize = 452;
const OAM_WRITE_ACCESS_DOT: usize = 76;
const VISIBLE_LINES: u8 = 144;
const TOTAL_LINES: u8 = 154;

/// LCD modes, with discriminants matching STAT bits 0–1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

pub struct Ppu {
    irq_vblank: bool,
    irq_stat: bool,
    stat_line: bool,

    screen_buffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    color_buffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    frame: Option<Frame>,

    clock: usize,
    pixel_transfer_end: usize,
    lcd_startup: bool,
    lcd_restart_timing: bool,
    ly_advanced: bool,
    lyc_rise_pending: bool,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    lcd_enabled: bool,               //FF40
    window_tilemap_select: bool,     //FF40 - false = 9800-9BFF / true = 9C00-9FFF
    window_enable: bool,             //FF40
    bg_window_tile_addressing: bool, //FF40 - false = 8800-97FF / true = 8000-8FFF
    bg_tilemap_select: bool,         //FF40 false = 9800-9BFF / true = 9C00-9FFF
    sprite_size: usize,              //FF40 false = 8x8 / true = 8x16
    sprite_enable: bool,             //FF40
    bg_window_priority: bool,        //FF40
    mode: PpuMode,                   //FF41 bits 0–1
    irq_m0_enable: bool,             //sets what triggers the stat interrupt FF41
    irq_m1_enable: bool,
    irq_m2_enable: bool,
    irq_lyc_enable: bool,
    lyc_equal: bool,
    scroll_y: u8,      //SCY FF42
    scroll_x: u8,      //SCX FF43
    line: u8,          //LY FF44 current line drawn by the display controller
    line_compare: u8,  //LYC FF45 compare value for LYC
    bg_palette: u8,    //BGP FF47
    obj_palette_1: u8, //OBP0 FF48
    obj_palette_2: u8, //OBP1 FF49
    window_y: u8,      //WY FF4A 0
    window_x: u8,      //WX FF4B 7
}

impl Ppu {
    pub fn new() -> Ppu {
        // Execution skips the DMG boot ROM, which leaves LCDC at 0x91:
        // LCD and background enabled, with unsigned tile addressing.
        // Games may wait for VBlank before their first LCDC write.
        Ppu {
            screen_buffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            color_buffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            frame: None,
            irq_vblank: false,
            irq_stat: false,
            stat_line: false,
            clock: 0, // for the first line
            pixel_transfer_end: PIXEL_TRANSFER_END,
            lcd_startup: false,
            lcd_restart_timing: false,
            ly_advanced: false,
            lyc_rise_pending: false,
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcd_enabled: true,
            window_tilemap_select: false,
            window_enable: false,
            bg_window_tile_addressing: true,
            bg_tilemap_select: false,
            sprite_size: 8,
            sprite_enable: false,
            bg_window_priority: true,
            mode: PpuMode::HBlank,
            irq_m0_enable: false,
            irq_m1_enable: false,
            irq_m2_enable: false,
            irq_lyc_enable: false,
            lyc_equal: true,
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
            0x8000..=0x9FFF => {
                let offset = address as usize - 0x8000;
                self.vram[offset]
            }
            0xFE00..=0xFE9F => {
                let offset = address as usize - 0xFE00;
                self.oam[offset]
            }
            0xFF40 => {
                // LCD Control
                (if self.lcd_enabled { 0x80 } else { 0x00 })
                    | (if self.window_tilemap_select {
                        0x40
                    } else {
                        0x00
                    })
                    | (if self.window_enable { 0x20 } else { 0x00 })
                    | (if self.bg_window_tile_addressing {
                        0x10
                    } else {
                        0x00
                    })
                    | (if self.bg_tilemap_select { 0x08 } else { 0x00 })
                    | (if self.sprite_size == 16 { 0x04 } else { 0x00 })
                    | (if self.sprite_enable { 0x02 } else { 0x00 })
                    | (if self.bg_window_priority { 0x01 } else { 0x00 })
            }
            0xFF41 => {
                // LCD Status
                0x80 | (if self.irq_lyc_enable { 0x40 } else { 0x00 })
                    | (if self.irq_m2_enable { 0x20 } else { 0x00 })
                    | (if self.irq_m1_enable { 0x10 } else { 0x00 })
                    | (if self.irq_m0_enable { 0x08 } else { 0x00 })
                    | (if self.lyc_equal { 0x04 } else { 0x00 })
                    | self.mode as u8
            }
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.line,
            0xFF45 => self.line_compare,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette_1,
            0xFF49 => self.obj_palette_2,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Invalid read at GPU memory adress: {:#06X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                let offset = address as usize - 0x8000;
                self.vram[offset] = value;
            }
            0xFE00..=0xFE9F => {
                let offset = address as usize - 0xFE00;
                self.oam[offset] = value;
            }
            0xFF40 => {
                let was_enabled = self.lcd_enabled;
                self.lcd_enabled = value & 0x80 == 0x80;
                self.window_tilemap_select = value & 0x40 == 0x40;
                self.window_enable = value & 0x20 == 0x20;
                self.bg_window_tile_addressing = value & 0x10 == 0x10;
                self.bg_tilemap_select = value & 0x08 == 0x08;
                self.sprite_size = if value & 0x04 == 0x04 { 16 } else { 8 };
                self.sprite_enable = value & 0x02 == 0x02;
                self.bg_window_priority = value & 0x01 == 0x01;
                if self.lcd_enabled && !was_enabled {
                    self.clock = 0;
                    self.line = 0;
                    self.mode = PpuMode::HBlank;
                    self.lcd_startup = true;
                    self.lcd_restart_timing = true;
                    self.ly_advanced = false;
                    self.lyc_rise_pending = false;
                    self.lyc_equal = self.line == self.line_compare;
                } else if !self.lcd_enabled && was_enabled {
                    self.clock = 0;
                    self.line = 0;
                    self.mode = PpuMode::HBlank;
                    self.lcd_startup = false;
                    self.lcd_restart_timing = false;
                    self.ly_advanced = false;
                    self.lyc_rise_pending = false;
                }
                self.update_stat_line();
            }
            0xFF41 => {
                self.irq_lyc_enable = value & 0x40 == 0x40;
                self.irq_m2_enable = value & 0x20 == 0x20;
                self.irq_m1_enable = value & 0x10 == 0x10;
                self.irq_m0_enable = value & 0x08 == 0x08;
                self.update_stat_line();
            }
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF45 => {
                self.line_compare = value;
                if self.lcd_enabled {
                    self.lyc_equal = self.line == self.line_compare;
                    self.lyc_rise_pending = false;
                }
                self.update_stat_line();
            }
            0xFF44 => {
                self.line = 0;
                if self.lcd_enabled {
                    self.lyc_equal = self.line == self.line_compare;
                    self.lyc_rise_pending = false;
                }
                self.update_stat_line();
            }
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette_1 = value,
            0xFF49 => self.obj_palette_2 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!(
                "Invalid write at GPU memory adress: {:#06X} - {:#06X}",
                address, value
            ),
        }
    }

    pub fn get_screen_buffer(&self) -> Frame {
        let mut pixels = Vec::with_capacity(Frame::PIXEL_COUNT);
        for row in &self.screen_buffer {
            pixels.extend_from_slice(row);
        }
        Frame::try_from(pixels).expect("PPU renders one screen of valid DMG shades")
    }

    pub fn take_frame(&mut self) -> Option<Frame> {
        self.frame.take()
    }

    /// Consumes the current VBlank request without affecting STAT or frame delivery.
    pub fn take_vblank_interrupt(&mut self) -> bool {
        std::mem::take(&mut self.irq_vblank)
    }

    /// Consumes the current STAT request without affecting VBlank or frame delivery.
    pub fn take_stat_interrupt(&mut self) -> bool {
        std::mem::take(&mut self.irq_stat)
    }

    pub(crate) fn cpu_can_read_oam(&self) -> bool {
        !self.lcd_enabled
            || (matches!(self.mode, PpuMode::HBlank | PpuMode::VBlank)
                && !(self.lcd_restart_timing && self.ly_advanced && self.line < VISIBLE_LINES))
    }

    pub(crate) fn cpu_can_write_oam(&self) -> bool {
        !self.lcd_enabled
            || matches!(self.mode, PpuMode::HBlank | PpuMode::VBlank)
            || (self.lcd_restart_timing
                && self.mode == PpuMode::OamSearch
                && self.clock >= OAM_WRITE_ACCESS_DOT)
    }

    pub(crate) fn cpu_can_read_vram(&self) -> bool {
        !self.lcd_enabled
            || (self.mode != PpuMode::PixelTransfer
                && !(self.lcd_restart_timing
                    && !self.lcd_startup
                    && self.mode == PpuMode::OamSearch
                    && self.clock >= OAM_WRITE_ACCESS_DOT))
    }

    pub(crate) fn cpu_can_write_vram(&self) -> bool {
        !self.lcd_enabled || self.mode != PpuMode::PixelTransfer
    }

    /* timing
    OAM search - 80 - determine which sprites are visible
    pixel transfer - 172 - draw the stuff
    Horizontal blank 204
    Single line 456
    Vertical blank 4560
    Entire frame 70224 */
    pub fn do_ticks(&mut self, mut ticks: usize) {
        if !self.lcd_enabled {
            // Reset directly: LCD disable does not run HBlank entry effects.
            self.clock = 0;
            self.line = 0;
            self.mode = PpuMode::HBlank;
            self.update_stat_line();
            return;
        }

        if !self.lcd_startup && self.line < VISIBLE_LINES && self.clock < OAM_SEARCH_END {
            self.set_mode_if_changed(PpuMode::OamSearch);
        }

        while ticks > 0 {
            let boundary = if self.line < VISIBLE_LINES && self.clock < OAM_SEARCH_END {
                OAM_SEARCH_END
            } else if self.line < VISIBLE_LINES && self.clock < self.pixel_transfer_end {
                self.pixel_transfer_end
            } else if !self.ly_advanced && self.clock < LY_ADVANCE_DOT {
                LY_ADVANCE_DOT
            } else {
                DOTS_PER_LINE
            };
            let elapsed = ticks.min(boundary - self.clock);
            self.clock += elapsed;
            ticks -= elapsed;

            if self.clock != boundary {
                continue;
            }

            if boundary == OAM_SEARCH_END {
                self.pixel_transfer_end = PIXEL_TRANSFER_END + self.pixel_transfer_penalty();
                self.set_mode_if_changed(PpuMode::PixelTransfer);
            } else if boundary == self.pixel_transfer_end {
                self.set_mode_if_changed(PpuMode::HBlank);
            } else if boundary == LY_ADVANCE_DOT {
                self.advance_ly();
            } else if boundary == DOTS_PER_LINE {
                self.finish_line();
            } else {
                unreachable!();
            }
        }
    }

    fn pixel_transfer_penalty(&self) -> usize {
        let fine_scroll = usize::from(self.scroll_x & 7);
        if !self.sprite_enable {
            return fine_scroll;
        }

        let line_y = u16::from(self.line) + 16;
        let mut sprite_x = [0; 10];
        let mut sprite_count = 0;
        for sprite in self.oam.chunks_exact(4) {
            let y = u16::from(sprite[0]);
            if line_y >= y && line_y < y + self.sprite_size as u16 {
                sprite_x[sprite_count] = sprite[1];
                sprite_count += 1;
                if sprite_count == sprite_x.len() {
                    break;
                }
            }
        }
        sprite_x[..sprite_count].sort_unstable();

        let mut seen_tiles = [false; 22];
        let mut sprite_penalty = 0;
        for x in sprite_x[..sprite_count].iter().copied() {
            if x >= 168 {
                continue;
            }

            let shifted_x = usize::from(x) + fine_scroll;
            let tile = shifted_x / 8;
            if !seen_tiles[tile] {
                let pixel_offset = if x == 0 { 0 } else { shifted_x % 8 };
                sprite_penalty += 5usize.saturating_sub(pixel_offset);
                seen_tiles[tile] = true;
            }
            sprite_penalty += 6;
        }
        fine_scroll + sprite_penalty / 4 * 4
    }

    fn advance_ly(&mut self) {
        self.line = (self.line + 1) % TOTAL_LINES;
        self.ly_advanced = true;
        let equal = self.line == self.line_compare;
        self.lyc_rise_pending = equal && !self.lyc_equal;
        if !equal {
            self.lyc_equal = false;
        }
        self.update_stat_line();
    }

    fn finish_line(&mut self) {
        self.clock = 0;
        if self.ly_advanced {
            self.ly_advanced = false;
            if self.lyc_rise_pending {
                self.lyc_equal = true;
                self.lyc_rise_pending = false;
                self.update_stat_line();
            }
        } else {
            self.line = (self.line + 1) % TOTAL_LINES;
            self.lyc_equal = self.line == self.line_compare;
            self.lyc_rise_pending = false;
            self.update_stat_line();
        }
        self.lcd_startup = false;

        if self.line == VISIBLE_LINES {
            self.set_mode_if_changed(PpuMode::VBlank);
        } else if self.line < VISIBLE_LINES {
            self.set_mode_if_changed(PpuMode::OamSearch);
        }
    }

    fn set_mode_if_changed(&mut self, mode: PpuMode) {
        if self.mode != mode {
            self.set_mode(mode);
        }
    }

    fn update_stat_line(&mut self) {
        let mode_source = self.lcd_enabled
            && match self.mode {
                PpuMode::HBlank => self.irq_m0_enable,
                PpuMode::VBlank => {
                    self.irq_m1_enable || (self.irq_m2_enable && self.line == VISIBLE_LINES)
                }
                PpuMode::OamSearch => self.irq_m2_enable,
                PpuMode::PixelTransfer => false,
            };
        let next_line = mode_source || (self.irq_lyc_enable && self.lyc_equal);
        if next_line && !self.stat_line {
            self.irq_stat = true;
        }
        self.stat_line = next_line;
    }

    /// Applies mode-entry rendering and interrupt effects.
    fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        self.update_stat_line();

        match mode {
            PpuMode::VBlank => {
                self.irq_vblank = true;
                self.frame = Some(self.get_screen_buffer());
            }
            PpuMode::OamSearch => {}
            PpuMode::PixelTransfer => self.render_line(),
            PpuMode::HBlank => {}
        }
    }

    fn render_line(&mut self) {
        self.render_background();

        if self.window_enable {
            self.render_window();
        }

        if self.sprite_enable {
            self.render_sprites();
        }
    }

    fn render_background(&mut self) {
        let tile_map_address = if self.bg_tilemap_select {
            0x1C00
        } else {
            0x1800
        };

        let mut current_tile = 0; //address of the currently selected tile
        let mut color: [u8; 8] = [0; 8];

        let y = self.line as usize; //the line index
        let bg_y = (y + self.scroll_y as usize) % 256;
        let bg_y_tile = bg_y / 8; // there are 32x32 tiles that are 8x8 - find the tile index by divide through 8
        let bg_y_offset = bg_y % 8; //the offset of the line inside the sprite;

        for x in 0..SCREEN_WIDTH {
            //draw one pixel after the other
            let bg_x = (x + self.scroll_x as usize) % 256;
            let bg_x_tile = bg_x / 8;
            let bg_x_offset = bg_x % 8;

            let tile_selected_tile = tile_map_address + (bg_y_tile * 32) + bg_x_tile;
            if tile_selected_tile != current_tile {
                let tile_offset = self.vram[tile_selected_tile];
                let tile_address = if self.bg_window_tile_addressing {
                    tile_offset as usize * 16
                } else {
                    0x1000u16.wrapping_add((tile_offset as i8 as i16 * 16) as u16) as usize
                }; //false = 8800-97FF / true = 8000-8FFF
                let byte_address = tile_address + (bg_y_offset * 2);

                color = Ppu::sprite_row(self.vram[byte_address], self.vram[byte_address + 1]);
                current_tile = tile_selected_tile;
            }

            self.screen_buffer[y][x] = (self.bg_palette >> (color[bg_x_offset] * 2)) & 0x03;
            self.color_buffer[y][x] = color[bg_x_offset];
        }
    }

    fn render_window(&mut self) {
        // TODO: Implement window rendering.
    }

    fn render_sprites(&mut self) {
        for sprite_num in 0..40 {
            //draw from the back to the front for sprite priority
            let sprite_address = (39 - sprite_num) * 4;

            let y = self.oam[sprite_address] as usize;
            let x = self.oam[sprite_address + 1] as usize;

            let line = self.line as usize + 16;
            if y > 0 && y < 160 && x > 0 && x < 168 {
                //otherwise the sprite is hidden
                if line >= y && line < (y + self.sprite_size) {
                    let sprite_id = self.oam[sprite_address + 2];
                    let sprite_attributes = self.oam[sprite_address + 3];

                    let palette = if sprite_attributes & 0x10 == 0x10 {
                        self.obj_palette_2
                    } else {
                        self.obj_palette_1
                    };
                    let flip_x = sprite_attributes & 0x20 == 0x20;
                    let flip_y = sprite_attributes & 0x40 == 0x40;
                    let behind_bg = sprite_attributes & 0x80 == 0x80;

                    /*  Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
                    (Used for both BG and Window. BG color 0 is always behind OBJ)
                    Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
                    Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
                    Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
                    Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
                    Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7) */

                    let sprite_start_address = sprite_id as usize * 16;
                    let sprite_line = if flip_y {
                        y + self.sprite_size - 1 - line
                    } else {
                        line - y
                    };

                    let first = self.vram[sprite_start_address + sprite_line * 2];
                    let second = self.vram[sprite_start_address + sprite_line * 2 + 1];

                    let color = Ppu::sprite_row(first, second);
                    for x_offset in 0..8 {
                        let pixel = if flip_x { 7 - x_offset } else { x_offset };
                        if color[pixel] > 0 {
                            // OAM stores sprite X plus eight, so X values below eight leave
                            // only the sprite's rightmost columns visible on the screen.
                            if let Some(screen_x) = (x + x_offset).checked_sub(8) {
                                if screen_x < 160
                                    && !(self.color_buffer[self.line as usize][screen_x] > 0
                                        && behind_bg)
                                {
                                    self.screen_buffer[self.line as usize][screen_x] =
                                        (palette >> (color[pixel] * 2)) & 0x03;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn sprite_row(first: u8, second: u8) -> [u8; 8] {
        let mut result = [0u8; 8];
        for (i, color) in result.iter_mut().enumerate() {
            let bit_index = 7 - i; // bit 7 left most bit 0 right most
            *color = ((first >> bit_index) & 0x01) | (((second >> bit_index) & 0x01) << 1);
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stat_reports_modes_at_exact_scanline_boundaries() {
        let mut ppu = Ppu::new();
        assert_eq!(ppu.read_byte(0xFF41) & 3, 0);

        for (ticks, expected_mode, expected_line) in [
            (1, 2, 0),
            (78, 2, 0),
            (1, 3, 0), // Dot 80.
            (171, 3, 0),
            (1, 0, 0), // Dot 252.
            (199, 0, 0),
            (1, 0, 1), // LY advances at dot 452.
            (3, 0, 1),
            (1, 2, 1), // The next line enters mode 2 at dot 456.
        ] {
            ppu.do_ticks(ticks);
            assert_eq!(ppu.read_byte(0xFF41) & 3, expected_mode);
            assert_eq!(ppu.read_byte(0xFF44), expected_line);

            // STAT writes can enable interrupts but cannot overwrite mode bits.
            ppu.write_byte(0xFF41, 0xFF);
            assert_eq!(ppu.read_byte(0xFF41) & 3, expected_mode);
            assert_eq!(ppu.read_byte(0xFF41) & 0xF8, 0xF8);
        }
    }

    #[test]
    fn visible_mode_entries_request_only_the_enabled_stat_interrupt() {
        for enables in [0, 0x08, 0x10, 0x20] {
            let mut ppu = Ppu::new();
            ppu.do_ticks(OAM_SEARCH_END); // Start from mode 3, which has no STAT source.
            ppu.write_byte(0xFF41, enables);
            assert!(!ppu.take_stat_interrupt());
            ppu.do_ticks(PIXEL_TRANSFER_END - OAM_SEARCH_END);
            assert_eq!(ppu.take_stat_interrupt(), enables == 0x08);
            ppu.do_ticks(DOTS_PER_LINE - PIXEL_TRANSFER_END);
            assert_eq!(ppu.take_stat_interrupt(), enables == 0x20);
            ppu.do_ticks(4); // Same mode: no repeated request.
            assert!(!ppu.take_stat_interrupt());
            assert!(!ppu.take_vblank_interrupt());
            assert!(ppu.take_frame().is_none());
        }
    }

    #[test]
    fn combined_stat_line_blocks_edges_while_any_source_stays_high() {
        let mut ppu = Ppu::new();
        ppu.do_ticks(1); // Mode 2 with LY=LYC.
        ppu.write_byte(0xFF41, 0x78);
        assert!(ppu.take_stat_interrupt());

        ppu.do_ticks(DOTS_PER_LINE - 1); // LYC, mode 0, and mode 2 bridge the line.
        assert_eq!((ppu.line, ppu.mode), (1, PpuMode::OamSearch));
        assert!(!ppu.take_stat_interrupt());

        ppu.write_byte(0xFF45, 1);
        ppu.do_ticks(OAM_SEARCH_END); // Coincidence bridges mode 2 through mode 3.
        assert_eq!(ppu.mode, PpuMode::PixelTransfer);
        assert!(!ppu.take_stat_interrupt());

        ppu.write_byte(0xFF41, 0);
        ppu.write_byte(0xFF41, 0x40);
        assert!(ppu.take_stat_interrupt());
    }

    #[test]
    fn lcd_off_retains_the_coincidence_latch_until_restart() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF41, 0x40);
        assert!(ppu.take_stat_interrupt());
        ppu.write_byte(0xFF45, 1);
        ppu.do_ticks(DOTS_PER_LINE);
        assert!(ppu.take_stat_interrupt());
        assert_ne!(ppu.read_byte(0xFF41) & 0x04, 0);

        ppu.write_byte(0xFF40, 0);
        ppu.do_ticks(4);
        ppu.write_byte(0xFF45, 2);
        assert_ne!(ppu.read_byte(0xFF41) & 0x04, 0);

        ppu.write_byte(0xFF40, 0x80);
        assert_eq!(ppu.read_byte(0xFF41) & 0x04, 0);
        assert!(!ppu.take_stat_interrupt());
    }

    #[test]
    fn mode_two_stat_source_also_rises_at_vblank_entry() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF41, 0x20);
        ppu.do_ticks(143 * DOTS_PER_LINE + PIXEL_TRANSFER_END);
        assert!(ppu.take_stat_interrupt());

        ppu.do_ticks(DOTS_PER_LINE - PIXEL_TRANSFER_END);
        assert_eq!((ppu.line, ppu.mode), (VISIBLE_LINES, PpuMode::VBlank));
        assert!(ppu.take_vblank_interrupt());
        assert!(ppu.take_stat_interrupt());
    }

    #[test]
    fn cpu_oam_access_follows_mode_boundaries() {
        let mut ppu = Ppu::new();
        assert!(ppu.cpu_can_read_oam());

        ppu.do_ticks(1);
        assert!(!ppu.cpu_can_read_oam());
        ppu.do_ticks(79);
        assert_eq!(ppu.mode, PpuMode::PixelTransfer);
        assert!(!ppu.cpu_can_read_oam());
        ppu.do_ticks(172);
        assert_eq!(ppu.mode, PpuMode::HBlank);
        assert!(ppu.cpu_can_read_oam());
    }

    #[test]
    fn one_tick_batch_processes_every_crossed_mode_boundary() {
        let mut ppu = Ppu::new();

        ppu.do_ticks(DOTS_PER_LINE + OAM_SEARCH_END);

        assert_eq!(ppu.line, 1);
        assert_eq!(ppu.clock, OAM_SEARCH_END);
        assert_eq!(ppu.mode, PpuMode::PixelTransfer);
    }

    #[test]
    fn scx_and_sprite_fetches_extend_mode_three() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF43, 7);
        ppu.sprite_enable = true;
        ppu.oam[0] = 16;
        ppu.oam[1] = 0;

        ppu.do_ticks(OAM_SEARCH_END);
        assert_eq!(ppu.pixel_transfer_end, PIXEL_TRANSFER_END + 7 + 8);
        ppu.do_ticks(PIXEL_TRANSFER_END - OAM_SEARCH_END);
        assert_eq!(ppu.mode, PpuMode::PixelTransfer);
        ppu.do_ticks(15);
        assert_eq!(ppu.mode, PpuMode::HBlank);
    }

    #[test]
    fn sprite_penalties_follow_fetch_tiles_not_oam_order() {
        fn penalty(xs: &[u8]) -> usize {
            let mut ppu = Ppu::new();
            ppu.line = 66;
            ppu.sprite_enable = true;
            for (sprite, x) in ppu.oam.chunks_exact_mut(4).zip(xs.iter().copied()) {
                sprite[0] = 82;
                sprite[1] = x;
            }
            ppu.pixel_transfer_penalty()
        }

        assert_eq!(penalty(&[0]), 8);
        assert_eq!(penalty(&[1]), 8);
        assert_eq!(penalty(&[4]), 4);
        assert_eq!(penalty(&[5]), 4);
        assert_eq!(penalty(&[168]), 0);
        assert_eq!(penalty(&[0; 10]), 64);
        assert_eq!(penalty(&[1; 10]), 64);
        assert_eq!(penalty(&[0, 8]), 20);
        assert_eq!(penalty(&[8, 0]), 20);
        assert_eq!(penalty(&[0, 8, 16, 24, 32, 40, 48, 56, 64, 72]), 108);
    }

    #[test]
    fn lcd_restart_uses_the_dmg_first_line_timeline() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF40, 0);
        ppu.write_byte(0xFF40, 0x80);

        assert_eq!((ppu.line, ppu.clock, ppu.mode), (0, 0, PpuMode::HBlank));
        assert_eq!(ppu.read_byte(0xFF41) & 0x07, 0x04);
        ppu.do_ticks(OAM_SEARCH_END);
        assert_eq!(
            (ppu.line, ppu.clock, ppu.mode),
            (0, 80, PpuMode::PixelTransfer)
        );
        assert_eq!(ppu.read_byte(0xFF41) & 0x07, 0x07);
        ppu.do_ticks(PIXEL_TRANSFER_END - OAM_SEARCH_END);
        assert_eq!((ppu.line, ppu.clock, ppu.mode), (0, 252, PpuMode::HBlank));

        ppu.do_ticks(LY_ADVANCE_DOT - PIXEL_TRANSFER_END);
        assert_eq!((ppu.line, ppu.clock, ppu.mode), (1, 452, PpuMode::HBlank));
        assert_eq!(ppu.read_byte(0xFF41) & 0x07, 0);
        ppu.do_ticks(DOTS_PER_LINE - LY_ADVANCE_DOT);
        assert_eq!((ppu.line, ppu.clock, ppu.mode), (1, 0, PpuMode::OamSearch));
        assert_eq!(ppu.read_byte(0xFF41) & 0x07, 0x02);

        ppu.do_ticks(OAM_SEARCH_END);
        assert_eq!(
            (ppu.line, ppu.clock, ppu.mode),
            (1, 80, PpuMode::PixelTransfer)
        );
        ppu.do_ticks(PIXEL_TRANSFER_END - OAM_SEARCH_END);
        assert_eq!((ppu.line, ppu.clock, ppu.mode), (1, 252, PpuMode::HBlank));
    }

    #[test]
    fn lcd_restart_exposes_the_dmg_oam_write_window() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF40, 0);
        ppu.write_byte(0xFF40, 0x80);

        ppu.do_ticks(LY_ADVANCE_DOT);
        assert!(!ppu.cpu_can_read_oam());
        assert!(ppu.cpu_can_write_oam());
        assert!(ppu.cpu_can_read_vram());
        assert!(ppu.cpu_can_write_vram());

        ppu.do_ticks(DOTS_PER_LINE - LY_ADVANCE_DOT + OAM_WRITE_ACCESS_DOT);
        assert_eq!(ppu.mode, PpuMode::OamSearch);
        assert!(!ppu.cpu_can_read_oam());
        assert!(ppu.cpu_can_write_oam());
        assert!(!ppu.cpu_can_read_vram());
        assert!(ppu.cpu_can_write_vram());

        ppu.do_ticks(OAM_SEARCH_END - OAM_WRITE_ACCESS_DOT);
        assert_eq!(ppu.mode, PpuMode::PixelTransfer);
        assert!(!ppu.cpu_can_read_oam());
        assert!(!ppu.cpu_can_write_oam());
        assert!(!ppu.cpu_can_read_vram());
        assert!(!ppu.cpu_can_write_vram());
    }

    #[test]
    fn lcd_restart_delays_a_rising_coincidence_until_mode_two() {
        let mut ppu = Ppu::new();
        ppu.write_byte(0xFF40, 0);
        ppu.write_byte(0xFF45, 1);
        ppu.write_byte(0xFF40, 0x80);

        ppu.do_ticks(LY_ADVANCE_DOT);
        assert_eq!((ppu.line, ppu.mode), (1, PpuMode::HBlank));
        assert_eq!(ppu.read_byte(0xFF41) & 0x04, 0);

        ppu.do_ticks(DOTS_PER_LINE - LY_ADVANCE_DOT);
        assert_eq!(ppu.mode, PpuMode::OamSearch);
        assert_eq!(ppu.read_byte(0xFF41) & 0x04, 0x04);
    }

    #[test]
    fn vblank_entry_publishes_one_frame_and_requests_interrupts() {
        for stat_enabled in [false, true] {
            let mut ppu = Ppu::new();
            ppu.write_byte(0xFF41, if stat_enabled { 0x10 } else { 0 });
            for _ in 0..(144 * 456 / 4) {
                ppu.do_ticks(4);
            }
            assert_eq!(ppu.read_byte(0xFF44), 144);
            assert_eq!(ppu.read_byte(0xFF41) & 3, 1);
            assert!(ppu.take_vblank_interrupt());
            assert!(!ppu.take_vblank_interrupt());
            assert_eq!(ppu.take_stat_interrupt(), stat_enabled);
            assert!(!ppu.take_stat_interrupt());
            assert_eq!(
                ppu.take_frame().expect("completed frame").pixels().len(),
                SCREEN_WIDTH * SCREEN_HEIGHT
            );

            // Stay within VBlank: neither frames nor requests are repeated.
            for _ in 0..(10 * 456 / 4 - 1) {
                ppu.do_ticks(4);
            }
            assert_eq!(ppu.read_byte(0xFF41) & 3, 1);
            assert!(ppu.take_frame().is_none());
            assert!(!ppu.take_vblank_interrupt());
            assert!(!ppu.take_stat_interrupt());
            ppu.do_ticks(4);
            assert_eq!((ppu.read_byte(0xFF44), ppu.read_byte(0xFF41) & 3), (0, 2));
        }
    }

    #[test]
    fn disabling_lcd_resets_line_and_mode_without_a_hblank_entry_interrupt() {
        let mut ppu = Ppu::new();
        ppu.do_ticks(456 + 84); // Line 1, pixel transfer.
        assert_eq!((ppu.read_byte(0xFF44), ppu.read_byte(0xFF41) & 3), (1, 3));
        ppu.write_byte(0xFF41, 0x08);
        ppu.write_byte(0xFF40, 0);
        ppu.do_ticks(4);
        assert_eq!((ppu.read_byte(0xFF44), ppu.read_byte(0xFF41) & 3), (0, 0));
        assert!(!ppu.take_stat_interrupt());
        assert!(!ppu.take_vblank_interrupt());
        assert!(ppu.take_frame().is_none());
    }

    #[test]
    fn sprite_rows_to_color_values() {
        let first = 0b01010101;
        let second = 0b11000011;

        let result = Ppu::sprite_row(first, second);

        assert_eq!(result[0], 2);
        assert_eq!(result[1], 3);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 1);
        assert_eq!(result[4], 0);
        assert_eq!(result[5], 1);
        assert_eq!(result[6], 2);
        assert_eq!(result[7], 3);
    }

    #[test]
    fn renders_the_visible_edge_of_a_left_clipped_sprite() {
        let mut ppu = Ppu::new();
        ppu.oam[156] = 16;
        ppu.oam[157] = 1;
        ppu.vram[0] = 0b0000_0001;

        ppu.render_sprites();

        assert_eq!(ppu.screen_buffer[0][0], 3);
    }

    #[test]
    fn vertically_flipped_sprite_uses_its_bottom_source_row_first() {
        let mut ppu = Ppu::new();
        ppu.oam[156] = 16;
        ppu.oam[157] = 8;
        ppu.oam[159] = 0x40;
        ppu.obj_palette_1 = 0b1110_0100;
        ppu.vram[0] = 0b1000_0000;
        ppu.vram[15] = 0b1000_0000;

        ppu.render_sprites();

        assert_eq!(ppu.screen_buffer[0][0], 2);
    }
}
