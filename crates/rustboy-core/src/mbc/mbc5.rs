use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;

const RAM_BANK_SIZE: usize = 0x2000;
const NUM_RAM_BANK: usize = 16;

pub struct Mbc5 {
    rom: Vec<u8>,
    rom_mask: usize,
    selected_rom_bank: usize,
    ram: [u8; RAM_BANK_SIZE * NUM_RAM_BANK],
    ram_enabled: bool,
    selected_ram_bank: usize,
}

impl Mbc5 {
    pub fn new(buffer: Vec<u8>) -> Mbc5 {
        Mbc5 {
            rom_mask: buffer.len().next_power_of_two() - 1,
            rom: buffer,
            selected_rom_bank: 1, //0 is mapped to 0000-3FFF
            ram: [0; RAM_BANK_SIZE * NUM_RAM_BANK],
            selected_ram_bank: 0,
            ram_enabled: false,
        }
    }
}

impl Mbc for Mbc5 {
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
            self.ram
                .get(RAM_BANK_SIZE * self.selected_ram_bank + address as usize)
                .copied()
                .unwrap_or(0xFF)
        } else {
            0
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x2FFF => {
                self.selected_rom_bank = (self.selected_rom_bank & 0x100) | value as usize
            } //lower 8 bits
            0x3000..=0x3FFF => {
                self.selected_rom_bank =
                    (self.selected_rom_bank & 0xFF) | ((value as usize & 1) << 8)
            }
            0x4000..=0x5FFF => self.selected_ram_bank = (value as usize) & 0x0F,
            _ => (),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if self.ram_enabled {
            if let Some(cell) = self
                .ram
                .get_mut(RAM_BANK_SIZE * self.selected_ram_bank + address as usize)
            {
                *cell = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cartridge(banks: usize) -> Mbc5 {
        let mut rom = vec![0; banks * ROM_BANK_SIZE];
        for bank in 0..banks {
            for offset in [0, ROM_BANK_SIZE - 2] {
                rom[bank * ROM_BANK_SIZE + offset..bank * ROM_BANK_SIZE + offset + 2]
                    .copy_from_slice(&(bank as u16).to_le_bytes());
            }
        }
        Mbc5::new(rom)
    }

    fn read_bank(mbc: &Mbc5, address: u16) -> u16 {
        u16::from_le_bytes([mbc.read_rom(address), mbc.read_rom(address + 1)])
    }

    #[test]
    fn ninth_rom_bank_bit_is_masked_and_independent_of_the_low_register() {
        let mut mbc = cartridge(512);
        assert_eq!(read_bank(&mbc, 0x4000), 1);
        for value in 0..=u8::MAX {
            mbc.write_rom(0x2000, 0xA5);
            mbc.write_rom(0x3000, value);
            assert_eq!(read_bank(&mbc, 0x4000), 0xA5 | ((value as u16 & 1) << 8));
            mbc.write_rom(0x2FFF, 0x5A);
            assert_eq!(read_bank(&mbc, 0x7FFE), 0x5A | ((value as u16 & 1) << 8));
            assert_eq!(read_bank(&mbc, 0), 0);
        }
        mbc.write_rom(0x3FFF, 0);
        mbc.write_rom(0x2000, 0);
        assert_eq!(read_bank(&mbc, 0x4000), 0);
    }

    #[test]
    fn rom_capacity_masks_disconnected_address_lines() {
        for banks in [2, 4, 8, 16, 32, 64, 128, 256, 512] {
            let mut mbc = cartridge(banks);
            for bank in 0..512 {
                mbc.write_rom(0x2000, bank as u8);
                mbc.write_rom(0x3000, (bank >> 8) as u8);
                assert_eq!(read_bank(&mbc, 0x4000), (bank & (banks - 1)) as u16);
                assert_eq!(read_bank(&mbc, 0x7FFE), (bank & (banks - 1)) as u16);
            }
        }
    }
}
