# Rustboy

Rustboy is an in-progress Nintendo Game Boy emulator written in Rust. It separates the emulation core from SDL frontends and currently implements CPU execution, graphics, audio, input, timers, and several memory-bank controllers.

## Architecture

- `rustboy-core`: deterministic emulation
- `rustboy-application`: session and input/frame/audio ports.
- `rustboy-adapter-rom`: filesystem and ZIP ROM loader.
- `rustboy-adapter-sdl3`: SDL3 input, display, and audio adapter.
- `rustboy-cli`: command-line composition root.
- `rustboy-rom-tests`: external ROM conformance runner.
- `rustboy-debugger`: future debugger
- `rustboy-adapter-debugger-cli`: future terminal debugger adapter.

The core has no SDL, filesystem, ZIP, threads, channels, or wall-clock dependencies. Future debugger support will build on its synchronous `Machine::step` API.

## Status

This is a learning project and is not yet a fully compatible emulator. The currently supported cartridge families are ROM-only, MBC1, MBC2, and MBC5. MBC3, window rendering, serial interrupts, and joypad interrupts are not implemented. Save persistence, Game Boy Color support, and cycle-perfect emulation are out of scope for the current version.

## Legal ROM use

Rustboy does not include game ROMs. Use only ROMs you are legally entitled to use. The repository ignores ROM and archive files to prevent accidental redistribution of copyrighted games.

## Requirements

- Rust 1.73 or newer
- SDL3 development libraries installed on the system

Install SDL3 with your platform package manager before building:

```sh
# macOS
brew install sdl3

# Debian or Ubuntu
sudo apt install libsdl3-dev

# Fedora
sudo dnf install SDL3-devel
```

On Windows, install SDL3 through your preferred package manager or provide the SDL3 development libraries for your compiler toolchain. Rustboy does not distribute SDL binaries.

## Build and run

```sh
cargo emulate /path/to/your-game.gb
```

ZIP archives containing a single Game Boy ROM are also accepted. ROM images are limited to the maximum size supported by this emulator.

## Controls

| Game Boy | Keyboard |
| --- | --- |
| D-pad | Arrow keys |
| A | A |
| B | S |
| Start | Enter |
| Select | Space |
| Quit | Escape |

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The most valuable automated tests sit at the behavior boundaries:

- `rustboy-core`: deterministic CPU, mapper, timer, DMA, PPU, and APU edge cases.
- `rustboy-application`: frame-cycle budgets, input delivery, output effects, and pacing policy with fake ports.
- adapters: ROM file/ZIP limits and SDL input mapping, without duplicating core conformance tests.
- `rustboy-rom-tests`: black-box acceptance coverage from upstream ROM suites.

## Emulator test suites

Test ROM suites are intentionally not committed. They are large, combine projects under separate upstream licenses, and should remain independently updateable. Clone the MIT-licensed [GameboyTestSuites](https://github.com/adtennant/GameboyTestSuites) aggregate into the ignored `roms/` directory when validating emulator behavior:

```sh
git clone https://github.com/adtennant/GameboyTestSuites.git roms/GameboyTestSuites
```

The aggregate includes metadata, expected screenshots, and test ROMs from projects such as Blargg, Mooneye, SameSuite, and mealybug-tearoom-tests. Review each included suite's license before redistributing it.

Run the supported headless DMG conformance tests with the Cargo shortcut:

```sh
cargo rom-tests
```

The runner uses `roms/GameboyTestSuites` by default, or `RUSTBOY_TEST_ROMS` when set. Each eligible ROM is registered as an individual native-style test. The final Cargo test summary reports every passing and failing ROM, then exits nonzero if any conformance test fails. It currently evaluates time- and opcode-based exits with memory or register assertions for supported cartridge types. CGB/SGB tests, unsupported cartridge types, and screenshot-only assertions are skipped until the corresponding emulator support is implemented.

## License

Rustboy is available under the [MIT License](LICENSE). SDL3 and Rust crate dependencies remain subject to their own licenses.
