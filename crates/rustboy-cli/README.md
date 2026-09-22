# rustboy-cli

`rustboy-cli` is the composition root for the desktop emulator. It is intentionally small: it parses the ROM path, creates the ROM and SDL3 adapters, creates a core machine/session, and starts the application run loop.

Run a ROM from the workspace root:

```sh
cargo emulate path/to/game.gb
```

This is a Cargo alias for `cargo run -p rustboy-cli --release --`. ZIP archives containing one `.gb` ROM are also supported through `rustboy-adapter-rom`.

This crate is where user-facing errors are printed. It should not contain CPU instructions, PPU rendering rules, mapper behavior, or SDL event translation.

ROM-loading failures reach the CLI as `rustboy_adapter_rom::LoadError` through
the `RomSource` port. The CLI formats them with the `Could not load ROM:` context
and exits with status 1 before initializing SDL. The typed error and its source
chain remain available until that presentation boundary.

Platform initialization failures retain their typed SDL causes. Audio startup and
shutdown failures are reported with `Could not start audio:` and `Could not stop
audio:` respectively. Runtime failures are reported as `Emulator stopped:` followed
by the application's input/frame/audio operation context and underlying error.
