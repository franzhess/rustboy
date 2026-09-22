mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

use crate::mbc::mbc0::Mbc0;
use crate::mbc::mbc1::Mbc1;
use crate::mbc::mbc2::Mbc2;
use crate::mbc::mbc5::Mbc5;
use std::fmt;

const ADDR_TITLE_START: u16 = 0x0134;
const TITLE_SIZE: u16 = 16;
const ADDR_CARTRIDGE_TYPE: usize = 0x0147;
const MIN_ROM_SIZE: usize = ADDR_CARTRIDGE_TYPE + 1;
const MAX_ROM_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum RomLoadError {
    FileTooLarge { limit: usize },
    RomTooSmall { size: usize },
    UnsupportedCartridge(u8),
}

impl fmt::Display for RomLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomLoadError::FileTooLarge { limit } => {
                write!(formatter, "file exceeds the {limit}-byte size limit")
            }
            RomLoadError::RomTooSmall { size } => write!(
                formatter,
                "ROM is too small to contain a cartridge header ({size} bytes)"
            ),
            RomLoadError::UnsupportedCartridge(cartridge_type) => write!(
                formatter,
                "unsupported cartridge type {cartridge_type:#04X}"
            ),
        }
    }
}

impl std::error::Error for RomLoadError {}

pub(crate) trait Mbc: Send {
    fn read_rom(&self, address: u16) -> u8;
    fn read_ram(&self, address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn write_ram(&mut self, address: u16, value: u8);
    fn name(&self) -> String {
        let mut name = String::with_capacity(TITLE_SIZE as usize);

        for i in 0..TITLE_SIZE {
            match self.read_rom(ADDR_TITLE_START + i) {
                0 => break,
                ch => name.push(ch as char),
            }
        }

        name
    }
}

pub struct Cartridge {
    controller: Box<dyn Mbc>,
}

impl Cartridge {
    pub fn from_bytes(buffer: Vec<u8>) -> Result<Self, RomLoadError> {
        if buffer.len() < MIN_ROM_SIZE {
            return Err(RomLoadError::RomTooSmall { size: buffer.len() });
        }
        if buffer.len() > MAX_ROM_SIZE {
            return Err(RomLoadError::FileTooLarge {
                limit: MAX_ROM_SIZE,
            });
        }

        let mbc: Box<dyn Mbc + 'static> = match buffer[ADDR_CARTRIDGE_TYPE] {
            0x00 => Box::new(Mbc0::new(buffer)),
            0x01..=0x03 => Box::new(Mbc1::new(buffer)),
            0x05..=0x06 => Box::new(Mbc2::new(buffer)),
            0x19..=0x1E => Box::new(Mbc5::new(buffer)),
            cartridge_type => return Err(RomLoadError::UnsupportedCartridge(cartridge_type)),
        };
        Ok(Self { controller: mbc })
    }

    pub fn name(&self) -> String {
        self.controller.name()
    }

    pub(crate) fn into_controller(self) -> Box<dyn Mbc> {
        self.controller
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn cartridge_validation_errors_have_no_underlying_source() {
        for error in [
            RomLoadError::FileTooLarge {
                limit: MAX_ROM_SIZE,
            },
            RomLoadError::RomTooSmall { size: 0 },
            RomLoadError::UnsupportedCartridge(0x10),
        ] {
            let error: &dyn Error = &error;
            assert!(error.source().is_none());
        }
    }

    fn rom_with_type(cartridge_type: u8) -> Vec<u8> {
        let mut rom = vec![0; MIN_ROM_SIZE];
        rom[ADDR_CARTRIDGE_TYPE] = cartridge_type;
        rom
    }

    #[test]
    fn rejects_truncated_roms() {
        assert!(matches!(
            Cartridge::from_bytes(vec![0; MIN_ROM_SIZE - 1]),
            Err(RomLoadError::RomTooSmall { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_cartridges() {
        assert!(matches!(
            Cartridge::from_bytes(rom_with_type(0x0F)),
            Err(RomLoadError::UnsupportedCartridge(0x0F))
        ));
    }

    #[test]
    fn accepts_supported_cartridges() {
        for cartridge_type in [0x00, 0x01, 0x05, 0x19] {
            assert!(Cartridge::from_bytes(rom_with_type(cartridge_type)).is_ok());
        }
    }
}
