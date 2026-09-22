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

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Zip(error) => Some(error),
            Self::Cartridge(error) => Some(error),
            Self::FileTooLarge { .. } | Self::NoRomInArchive | Self::MultipleRomsInArchive => None,
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
    type Error = LoadError;

    fn load_cartridge(&self) -> Result<Cartridge, Self::Error> {
        load_rom(&self.path)
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

#[cfg(test)]
mod tests {
    use super::{io, FileRomSource, LoadError, RomLoadError};
    use rustboy_application::RomSource;
    use std::error::Error;
    use std::path::Path;
    use zip::result::ZipError;

    #[test]
    fn rom_source_port_preserves_typed_io_errors_and_their_causes() {
        // A checked-in regular file cannot contain a child ROM. This triggers a
        // real I/O failure without temporary files or assumptions about permissions.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Cargo.toml")
            .join("rom.gb");
        let source = FileRomSource::new(path);
        let port: &dyn RomSource<Error = LoadError> = &source;
        let error = port.load_cartridge().err().expect("invalid ROM path");

        let LoadError::Io(io_error) = &error else {
            panic!("expected typed I/O failure, got {error:?}");
        };
        let cause = error
            .source()
            .expect("I/O cause")
            .downcast_ref::<io::Error>()
            .expect("original I/O error type");
        assert_eq!(cause.kind(), io_error.kind());
        assert_eq!(error.to_string(), io_error.to_string());
    }

    #[test]
    fn io_source_preserves_error_kind_and_message() {
        let error = LoadError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "ROM access denied",
        ));
        let cause = error
            .source()
            .expect("I/O cause")
            .downcast_ref::<io::Error>()
            .expect("original I/O error type");

        assert_eq!(cause.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(cause.to_string(), "ROM access denied");
        assert_eq!(error.to_string(), "ROM access denied");
    }

    #[test]
    fn zip_source_preserves_nested_io_error() {
        let error = LoadError::Zip(ZipError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated ZIP data",
        )));
        let cause = error
            .source()
            .expect("ZIP cause")
            .downcast_ref::<ZipError>()
            .expect("original ZIP error type");

        let ZipError::Io(io_error) = cause else {
            panic!("expected nested I/O error, got {cause:?}");
        };
        assert_eq!(io_error.kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(io_error.to_string(), "truncated ZIP data");
        assert_eq!(error.to_string(), format!("invalid ZIP archive: {cause}"));
    }

    #[test]
    fn cartridge_source_preserves_validation_details_and_ends_the_chain() {
        let error = LoadError::Cartridge(RomLoadError::RomTooSmall { size: 42 });
        let cause = error.source().expect("cartridge cause");

        assert!(matches!(
            cause.downcast_ref::<RomLoadError>(),
            Some(RomLoadError::RomTooSmall { size: 42 })
        ));
        assert!(cause.source().is_none());
        assert_eq!(error.to_string(), cause.to_string());
    }

    #[test]
    fn adapter_validation_errors_have_no_underlying_source() {
        for error in [
            LoadError::FileTooLarge { limit: 8 },
            LoadError::NoRomInArchive,
            LoadError::MultipleRomsInArchive,
        ] {
            assert!(error.source().is_none());
        }
    }
}
