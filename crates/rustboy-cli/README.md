# rustboy-cli

`rustboy-cli` is the composition root for the desktop emulator. It is intentionally small: it parses the ROM path, creates the ROM and SDL3 adapters, creates a core machine/session, and starts the application run loop.

Run a ROM from the workspace root:

```sh
cargo emulate path/to/game.gb
```

This is a Cargo alias for `cargo run -p rustboy-cli --release --`. ZIP archives containing one `.gb` ROM are also supported through `rustboy-adapter-rom`.

This crate is where user-facing errors are printed. It should not contain CPU instructions, PPU rendering rules, mapper behavior, or SDL event translation.
