use crate::mbc::Mbc;

const ROM_BANK_SIZE: usize = 0x4000;

const RAM_BANK_SIZE: usize = 0x2000;
const NUM_RAM_BANK: usize = 4;

enum BankingMode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    rom: Vec<u8>,
    selected_rom_bank: usize,
    ram: [u8; RAM_BANK_SIZE * NUM_RAM_BANK],
    ram_enabled: bool,
    selected_ram_bank: usize,
    banking_mode: BankingMode,
}

impl Mbc1 {
    pub fn new(buffer: Vec<u8>) -> Mbc1 {
        Mbc1 {
            rom: buffer,
            selected_rom_bank: 1, //0 is mapped to 0000-3FFF
            ram: [0; RAM_BANK_SIZE * NUM_RAM_BANK],
            selected_ram_bank: 0,
            ram_enabled: false,
            banking_mode: BankingMode::Rom,
        }
    }
}

impl Mbc for Mbc1 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => self
                .rom
                .get(ROM_BANK_SIZE * self.selected_rom_bank + (address - 0x4000) as usize)
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
            0xFF
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                self.selected_rom_bank = (self.selected_rom_bank & 0x60)
                    | match value as usize & 0x1F {
                        0 => 1,
                        n => n,
                    }
            } //lower 5 bits 0x01-0x1F higher bits 5+6 0x60
            0x4000..=0x5FFF => match self.banking_mode {
                BankingMode::Rom => {
                    self.selected_rom_bank =
                        (self.selected_rom_bank & 0x1F) | ((value as usize & 0x03) << 5)
                }
                BankingMode::Ram => self.selected_ram_bank = (value as usize) & 0x03,
            },
            0x6000..=0x7FFF => match value & 1 {
                0 => self.banking_mode = BankingMode::Rom,
                _ => self.banking_mode = BankingMode::Ram,
            },
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

    #[test]
    fn ram_enable_uses_only_the_low_nibble() {
        let mut mbc = Mbc1::new(vec![0; ROM_BANK_SIZE * 2]);
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
        let mut mbc = Mbc1::new(vec![0; ROM_BANK_SIZE * 2]);
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
}
