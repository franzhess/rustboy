use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x200;

pub struct Mbc2 {
    rom: Vec<u8>,
    // Masks off disconnected ROM address bits so out-of-range bank selections mirror
    // existing banks. Set in new() to the ROM byte length rounded up to a power of two,
    // minus one; bytes missing from a truncated image still read as 0xFF.
    rom_mask: usize,
    selected_rom_bank: usize,
    ram: [u8; RAM_BANK_SIZE],
    ram_enabled: bool,
}

impl Mbc2 {
    pub fn new(buffer: Vec<u8>) -> Mbc2 {
        Mbc2 {
            rom_mask: buffer.len().next_power_of_two() - 1,
            rom: buffer,
            selected_rom_bank: 1, //0 is mapped to 0000-3FFF
            ram: [0; RAM_BANK_SIZE],
            ram_enabled: false,
        }
    }
}

impl Mbc for Mbc2 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => self
                .rom
                .get(
                    (ROM_BANK_SIZE * self.selected_rom_bank + (address - 0x4000) as usize)
                        & self.rom_mask,
                )
                .copied()
                .unwrap_or(0xFF),
            _ => 0,
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enabled {
            self.ram[(address as usize) & 0x01FF] | 0xF0
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        if address <= 0x3FFF {
            if address & 0x100 == 0 {
                self.ram_enabled = value & 0x0F == 0x0A;
            } else {
                self.selected_rom_bank = ((value as usize) & 0x0F).max(1);
            }
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if self.ram_enabled {
            self.ram[(address as usize) & 0x01FF] = value & 0x0F;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cartridge(banks: usize) -> Mbc2 {
        let rom = (0..banks)
            .flat_map(|bank| vec![bank as u8; ROM_BANK_SIZE])
            .collect();
        Mbc2::new(rom)
    }

    #[test]
    fn address_bit_eight_selects_the_register_throughout_the_control_region() {
        let mut mbc = cartridge(16);
        for address in 0..=0x3FFF {
            mbc.write_rom(0, 0);
            mbc.write_rom(0x100, 1);
            mbc.write_rom(address, 0xFA);
            if address & 0x100 == 0 {
                assert_eq!(mbc.read_ram(0), 0xF0);
                assert_eq!(mbc.read_rom(0x4000), 1);
            } else {
                assert_eq!(mbc.read_ram(0), 0xFF);
                assert_eq!(mbc.read_rom(0x4000), 10);
            }
        }
    }

    #[test]
    fn ram_is_four_bits_mirrored_and_write_protected_when_disabled() {
        let mut mbc = cartridge(2);
        for value in 0..=u8::MAX {
            mbc.write_rom(0, value);
            mbc.write_ram(0x1FFF, 0xAB);
            assert_eq!(
                mbc.read_ram(0x1FF),
                if value & 0x0F == 0x0A { 0xFB } else { 0xFF }
            );
            mbc.write_rom(0, 0x0A);
            assert_eq!(
                mbc.read_ram(0x3FF),
                if value & 0x0F == 0x0A { 0xFB } else { 0xF0 }
            );
            mbc.write_ram(0x1FF, 0);
        }
    }

    #[test]
    fn rom_bank_zero_is_remapped_before_disconnected_address_bits_are_masked() {
        for banks in [2, 4, 8, 16] {
            let mut mbc = cartridge(banks);
            assert_eq!(mbc.read_rom(0x4000), 1);
            for value in 0..=u8::MAX {
                mbc.write_rom(0x2100, value);
                assert_eq!(
                    mbc.read_rom(0x4000),
                    ((value & 15).max(1) as usize & (banks - 1)) as u8
                );
                assert_eq!(mbc.read_rom(0), 0);
            }
        }
    }

    #[test]
    fn upper_rom_writes_leave_banking_and_ram_enable_unchanged() {
        let mut mbc = cartridge(16);
        mbc.write_rom(0, 0x0A);
        mbc.write_ram(0, 5);
        mbc.write_rom(0x100, 3);
        for address in 0x4000..=0x7FFF {
            for value in [0, 0xFF] {
                mbc.write_rom(address, value);
                assert_eq!(mbc.read_rom(0x4000), 3);
                assert_eq!(mbc.read_ram(0), 0xF5);
            }
        }
    }
}
