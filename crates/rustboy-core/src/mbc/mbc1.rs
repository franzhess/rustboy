use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;

const RAM_BANK_SIZE: usize = 0x2000;

// Fixed bytes at 0x0104–0x0133 in a Game Boy header. Used only as a signature
// for detecting an additional game header, not as part of the banking logic.
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

enum BankingMode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    rom: Vec<u8>,
    // Connected ROM address bits: new() rounds the ROM byte length up to a power
    // of two and subtracts one. Masking mirrors banks beyond the chip's capacity;
    // bytes missing from a truncated image still read as 0xFF.
    rom_mask: usize,
    // Position of bank2 in the ROM bank number: 5 for standard MBC1 (groups of
    // 32 banks), 4 for MBC1M (16-bank games). Set by the header heuristic in new().
    bank_shift: u32,
    // Five-bit register for the low ROM bank bits, with zero remapped to one.
    bank1: usize,
    // Shared two-bit register: high ROM bank bits (5–6, or 4–5 on MBC1M)
    // and, in RAM mode, the RAM bank.
    // Mode switches change its routing, not its stored value.
    bank2: usize,
    ram: Vec<u8>,
    ram_enabled: bool,
    banking_mode: BankingMode,
}

impl Mbc1 {
    pub fn new(buffer: Vec<u8>) -> Mbc1 {
        // MBC1M shares MBC1's cartridge-type code, so the main header cannot
        // identify its different wiring. In a 1 MiB ROM, a second Nintendo logo
        // at bank 16's header position suggests another game starting 256 KiB in.
        // Require the main logo too. This is a heuristic: modified headers can
        // evade it, and a normal ROM containing both signatures can match it.
        let multicart = buffer.len() == 64 * ROM_BANK_SIZE
            && buffer.get(0x104..0x134) == Some(NINTENDO_LOGO.as_slice())
            && buffer.get(16 * ROM_BANK_SIZE + 0x104..16 * ROM_BANK_SIZE + 0x134)
                == Some(NINTENDO_LOGO.as_slice());
        // Header 0x147 is the cartridge type: 0x02/0x03 are MBC1 with RAM
        // (without/with a battery). Type 0x01 has no RAM, regardless of 0x149.
        // Header 0x149 encodes RAM capacity: 1 = 2 KiB, 2 = 8 KiB, 3 = 32 KiB.
        let ram_size = if matches!(buffer.get(0x147), Some(0x02 | 0x03)) {
            match buffer.get(0x149) {
                Some(1) => 0x800,
                Some(2) => RAM_BANK_SIZE,
                Some(3) => RAM_BANK_SIZE * 4,
                _ => 0,
            }
        } else {
            0
        };
        Mbc1 {
            rom_mask: buffer.len().next_power_of_two() - 1,
            bank_shift: if multicart { 4 } else { 5 },
            rom: buffer,
            bank1: 1,
            bank2: 0,
            ram: vec![0; ram_size],
            ram_enabled: false,
            banking_mode: BankingMode::Rom,
        }
    }

    fn ram_index(&self, address: u16) -> Option<usize> {
        // The MMU supplies an offset within 0xA000–0xBFFF, not a CPU address.
        // None makes disabled/absent RAM read as 0xFF and ignore writes.
        if !self.ram_enabled || self.ram.is_empty() {
            return None;
        }
        // ROM mode fixes RAM at bank zero; RAM mode also uses bank2 to select RAM.
        let bank = match self.banking_mode {
            BankingMode::Rom => 0,
            BankingMode::Ram => self.bank2,
        };
        // Supported RAM sizes are powers of two. Masking ignores disconnected
        // address lines: small chips mirror both bank selections and offsets.
        Some((bank * RAM_BANK_SIZE + address as usize) & (self.ram.len() - 1))
    }
}

impl Mbc for Mbc1 {
    fn read_rom(&self, address: u16) -> u8 {
        // The lower 16 KiB window is bank zero in ROM mode. In RAM mode it maps
        // bank 0/32/64/96 on MBC1, or the game base 0/16/32/48 on MBC1M.
        // The upper window combines bank2's high bits with bank1
        // in BOTH modes; selecting RAM mode does not clear the high ROM bits.
        let bank = match address {
            0x0000..=0x3FFF => match self.banking_mode {
                BankingMode::Rom => 0,
                BankingMode::Ram => self.bank2 << self.bank_shift,
            },
            // Zero remapping happens in the five-bit register before MBC1M
            // disconnects bit four: writing 0x10 can therefore select bank zero.
            0x4000..=0x7FFF => {
                // (1 << bank_shift) - 1 is 0x1F for MBC1 or 0x0F for MBC1M.
                // Place bank2 above those low bits, then combine the two registers.
                // E.g. bank2=2, bank1=3 selects bank 67 on MBC1, 35 on MBC1M.
                (self.bank2 << self.bank_shift) | (self.bank1 & ((1 << self.bank_shift) - 1))
            }
            _ => return 0xFF,
        };
        // 0x3FFF keeps the offset within a 16 KiB window. Apply the chip-size
        // mask after bank selection so physically absent address bits are ignored.
        self.rom
            .get((bank * ROM_BANK_SIZE + (address as usize & 0x3FFF)) & self.rom_mask)
            .copied()
            .unwrap_or(0xFF)
    }

    fn read_ram(&self, address: u16) -> u8 {
        self.ram_index(address)
            .map_or(0xFF, |index| self.ram[index])
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            // Only the low nibble controls RAM enable; upper bits are ignored.
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            // Keep five bits, then remap zero BEFORE applying the ROM-size mask.
            // E.g. on a 16-bank ROM, writing 0 selects bank 1, but 0x10 selects 0.
            0x2000..=0x3FFF => self.bank1 = (value as usize & 0x1F).max(1),
            // Latch both bits regardless of mode; reads decide how to route them.
            0x4000..=0x5FFF => self.bank2 = value as usize & 3,
            // Only bit zero selects the mode; bank register contents are retained.
            0x6000..=0x7FFF => match value & 1 {
                0 => self.banking_mode = BankingMode::Rom,
                _ => self.banking_mode = BankingMode::Ram,
            },
            _ => (),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if let Some(index) = self.ram_index(address) {
            self.ram[index] = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cartridge(banks: usize, ram_size: u8) -> Mbc1 {
        let mut rom: Vec<u8> = (0..banks)
            .flat_map(|bank| vec![bank as u8; ROM_BANK_SIZE])
            .collect();
        rom[0x147] = 3;
        rom[0x149] = ram_size;
        Mbc1::new(rom)
    }

    #[test]
    fn ram_enable_uses_only_the_low_nibble() {
        let mut mbc = cartridge(2, 3);
        for value in 0..=u8::MAX {
            mbc.write_rom(0x0000, value);
            assert_eq!(mbc.read_ram(0), if value & 0x0F == 0x0A { 0 } else { 0xFF });
            mbc.write_ram(0, value);
            mbc.write_rom(0x0000, 0x0A);
            assert_eq!(
                mbc.read_ram(0),
                if value & 0x0F == 0x0A { value } else { 0 }
            );
            mbc.write_ram(0, 0);
        }
    }

    #[test]
    fn banking_mode_uses_only_bit_zero() {
        let mut mbc = cartridge(2, 3);
        mbc.write_rom(0x0000, 0x0A);
        mbc.write_rom(0x6000, 1);
        mbc.write_rom(0x4000, 1);
        mbc.write_ram(0, 0x55);
        mbc.write_rom(0x4000, 0);
        for value in 0..=u8::MAX {
            mbc.write_rom(0x6000, value);
            mbc.write_rom(0x4000, 1);
            assert_eq!(mbc.read_ram(0), if value & 1 == 1 { 0x55 } else { 0 });
            mbc.write_rom(0x6000, 1);
            mbc.write_rom(0x4000, 0);
        }
    }

    #[test]
    fn mode_changes_remap_both_windows_without_losing_bank_registers() {
        let mut mbc = cartridge(128, 3);
        mbc.write_rom(0, 0x0A);
        mbc.write_ram(0, 0x10);
        mbc.write_rom(0x2000, 2);
        mbc.write_rom(0x4000, 3);
        assert_eq!(mbc.read_rom(0), 0);
        assert_eq!(mbc.read_rom(0x4000), 98);
        assert_eq!(mbc.read_ram(0), 0x10);
        mbc.write_rom(0x6000, 1);
        assert_eq!(mbc.read_rom(0), 96);
        assert_eq!(mbc.read_rom(0x4000), 98);
        mbc.write_ram(0, 0x30);
        mbc.write_rom(0x2000, 0);
        assert_eq!(mbc.read_rom(0x4000), 97);
        mbc.write_rom(0x6000, 0);
        assert_eq!(mbc.read_rom(0), 0);
        assert_eq!(mbc.read_rom(0x4000), 97);
        assert_eq!(mbc.read_ram(0), 0x10);
        mbc.write_rom(0x6000, 1);
        assert_eq!(mbc.read_ram(0), 0x30);
    }

    #[test]
    fn small_roms_ignore_disconnected_bank_bits_after_zero_remapping() {
        for banks in [2, 4, 8, 16, 32, 64, 128] {
            let mut mbc = cartridge(banks, 0);
            for value in 0..=u8::MAX {
                mbc.write_rom(0x2000, value);
                mbc.write_rom(0x4000, 0xFF);
                assert_eq!(
                    mbc.read_rom(0x7FFF),
                    (((value as usize & 31).max(1) | 96) & (banks - 1)) as u8
                );
            }
        }
    }

    #[test]
    fn ram_capacity_controls_bank_aliasing_and_absent_ram_reads() {
        for (size, banks) in [(0, 0), (1, 1), (2, 1), (3, 4)] {
            let mut mbc = cartridge(2, size);
            mbc.write_rom(0, 0x0A);
            mbc.write_rom(0x6000, 1);
            for bank in 0..4 {
                mbc.write_rom(0x4000, bank);
                mbc.write_ram(0, bank + 1);
            }
            for bank in 0..4 {
                mbc.write_rom(0x4000, bank);
                assert_eq!(
                    mbc.read_ram(0),
                    match banks {
                        0 => 0xFF,
                        1 => 4,
                        _ => bank + 1,
                    }
                );
            }
            if size == 1 {
                assert_eq!(mbc.read_ram(0x800), 4);
            }
        }
        let mut rom = vec![0; ROM_BANK_SIZE * 2];
        rom[0x147] = 1;
        rom[0x149] = 3;
        let mut mbc = Mbc1::new(rom);
        mbc.write_rom(0, 0x0A);
        mbc.write_ram(0, 0x55);
        assert_eq!(mbc.read_ram(0), 0xFF);
    }

    #[test]
    fn multicart_wiring_selects_sixteen_bank_games_and_masks_after_zero_remapping() {
        let mut rom = cartridge(64, 0).rom;
        for bank in [0, 16] {
            rom[bank * ROM_BANK_SIZE + 0x104..bank * ROM_BANK_SIZE + 0x134]
                .copy_from_slice(&NINTENDO_LOGO);
        }
        let mut mbc = Mbc1::new(rom);
        for game in 0..4 {
            mbc.write_rom(0x4000, game);
            for (value, offset) in [(0, 1), (1, 1), (15, 15), (16, 0), (31, 15)] {
                mbc.write_rom(0x2000, value);
                mbc.write_rom(0x6000, 0);
                assert_eq!(mbc.read_rom(0), 0);
                assert_eq!(mbc.read_rom(0x4000), game * 16 + offset);
                mbc.write_rom(0x6000, 1);
                assert_eq!(mbc.read_rom(0), game * 16);
                assert_eq!(mbc.read_rom(0x4000), game * 16 + offset);
            }
        }
    }

    #[test]
    fn normal_one_megabyte_rom_with_only_one_header_keeps_standard_wiring() {
        let mut rom = cartridge(64, 0).rom;
        rom[0x104..0x134].copy_from_slice(&NINTENDO_LOGO);
        let mut mbc = Mbc1::new(rom);
        mbc.write_rom(0x4000, 1);
        assert_eq!(mbc.read_rom(0x4000), 33);
        mbc.write_rom(0x6000, 1);
        assert_eq!(mbc.read_rom(0), 32);
    }
}
