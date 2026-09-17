use rustboy_core::mbc::{Cartridge, RomLoadError};
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

const MAX_ROM_SIZE: usize = 8 * 1024 * 1024;
const MAX_ARCHIVE_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Zip(zip::result::ZipError),
    FileTooLarge { limit: usize },
    NoRomInArchive,
    MultipleRomsInArchive,
    Cartridge(RomLoadError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Zip(error) => write!(f, "invalid ZIP archive: {error}"),
            Self::FileTooLarge { limit } => write!(f, "file exceeds the {limit}-byte size limit"),
            Self::NoRomInArchive => write!(f, "ZIP archive contains no .gb ROM"),
            Self::MultipleRomsInArchive => write!(f, "ZIP archive contains multiple .gb ROMs"),
            Self::Cartridge(error) => write!(f, "{error}"),
        }
    }
}

pub fn load_rom(path: impl AsRef<Path>) -> Result<Cartridge, LoadError> {
    let path = path.as_ref();
    let limit = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        MAX_ARCHIVE_SIZE
    } else {
        MAX_ROM_SIZE
    };
    let mut file = File::open(path).map_err(LoadError::Io)?;
    let bytes = read_limited(&mut file, limit)?;
    let bytes = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        extract_rom(bytes)?
    } else {
        bytes
    };
    Cartridge::from_bytes(bytes).map_err(LoadError::Cartridge)
}

pub struct FileRomSource {
    path: std::path::PathBuf,
}

impl FileRomSource {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl rustboy_application::RomSource for FileRomSource {
    fn load_cartridge(&self) -> Result<Cartridge, String> {
        load_rom(&self.path).map_err(|error| error.to_string())
    }
}

fn read_limited(reader: &mut impl Read, limit: usize) -> Result<Vec<u8>, LoadError> {
    let mut bytes = Vec::new();
    reader
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(LoadError::Io)?;
    if bytes.len() > limit {
        return Err(LoadError::FileTooLarge { limit });
    }
    Ok(bytes)
}

fn extract_rom(bytes: Vec<u8>) -> Result<Vec<u8>, LoadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(LoadError::Zip)?;
    let mut rom = None;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(LoadError::Zip)?;
        if !file.is_dir() && file.name().to_ascii_lowercase().ends_with(".gb") {
            if rom.is_some() {
                return Err(LoadError::MultipleRomsInArchive);
            }
            rom = Some(read_limited(&mut file, MAX_ROM_SIZE)?);
        }
    }
    rom.ok_or(LoadError::NoRomInArchive)
}
