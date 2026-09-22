# rustboy-adapter-rom

`rustboy-adapter-rom` is the filesystem adapter. It reads a ROM from a host path, optionally extracts the single `.gb` file contained in a ZIP archive, and creates a `rustboy-core::mbc::Cartridge`.

## Why ROM loading is an adapter

A Game Boy only sees bytes from its cartridge. It does not know about macOS paths, Windows paths, ZIP files, or filesystem errors. The core therefore accepts ROM bytes, while this crate handles host-specific input concerns.

## Safety limits

ROM files are limited to 8 MiB and ZIP archives to 16 MiB before extraction. ZIP archives must contain exactly one `.gb` file. These limits prevent accidental or hostile input from allocating arbitrarily large buffers.

`FileRomSource` implements the application `RomSource` port. The CLI uses it to load a game; other frontends can use the lower-level `load_rom` function when they need direct error handling.

## Errors

`load_rom` returns `Result<Cartridge, LoadError>`. `LoadError` implements
`std::error::Error` as well as `Display`, so callers can inspect its cause with
`Error::source()` and downcast it to the original concrete error type:

| Variant | Source |
| --- | --- |
| `Io` | `std::io::Error`, retaining its error kind and details |
| `Zip` | `zip::result::ZipError`, retaining any nested error |
| `Cartridge` | `rustboy_core::mbc::RomLoadError` |
| `FileTooLarge`, `NoRomInArchive`, `MultipleRomsInArchive` | None; these are adapter validation failures |

The application `RomSource` port currently returns string errors, so
`FileRomSource::load_cartridge` converts these errors to text. Use `load_rom`
directly when you need structured errors and their sources.

Run the error-source regression tests with `cargo test -p rustboy-adapter-rom`.
