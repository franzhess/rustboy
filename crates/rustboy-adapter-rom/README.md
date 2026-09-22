# rustboy-adapter-rom

`rustboy-adapter-rom` is the filesystem adapter. It reads a ROM from a host path, optionally extracts the single `.gb` file contained in a ZIP archive, and creates a `rustboy-core::mbc::Cartridge`.

## Why ROM loading is an adapter

A Game Boy only sees bytes from its cartridge. It does not know about macOS paths, Windows paths, ZIP files, or filesystem errors. The core therefore accepts ROM bytes, while this crate handles host-specific input concerns.

## Safety limits

ROM files are limited to 8 MiB and ZIP archives to 16 MiB before extraction. ZIP archives must contain exactly one `.gb` file. These limits prevent accidental or hostile input from allocating arbitrarily large buffers.

`FileRomSource` implements the application `RomSource` port. The CLI uses it to load a game; other frontends can use either this port or the lower-level `load_rom` function. Both return the same structured loading errors.

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

`FileRomSource` sets `RomSource::Error` to `LoadError` and returns the result of
`load_rom` directly. Callers of either API can match error variants and inspect
their sources. Formatting user-facing diagnostics belongs to the frontend; the
adapter does not convert failures to strings.

Run the error-source regression tests with `cargo test -p rustboy-adapter-rom`.
