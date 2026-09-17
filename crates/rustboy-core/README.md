# rustboy-core

`rustboy-core` is the deterministic Game Boy machine. It models the hardware inside the handheld without knowing how a ROM was loaded, how pixels are displayed, how audio reaches speakers, or how quickly the host computer runs.

The public entry point is `Machine`. Create a `Cartridge` with `Cartridge::from_bytes`, put it into a `Machine`, then call `Machine::step`. Each step executes one CPU instruction and returns its cycle count plus any completed video frame or audio buffers. A frontend, test runner, or debugger decides what to do with those effects.

## Timing and T-cycles

The original Game Boy is synchronized by a 4,194,304 Hz master clock. A **T-cycle** is one tick of that clock and is the smallest timing unit used by the core. CPU instructions take a multiple of four T-cycles; four T-cycles make one CPU **M-cycle** (machine cycle).

The CPU, timer, PPU, and APU advance from the same T-cycle count. After executing an instruction, `Machine::step` passes its consumed T-cycles to each hardware component. This keeps timer edges, display modes, audio generation, and interrupts synchronized with CPU execution rather than with the host computer's wall clock.

Frontends map emulated T-cycles to real time. The desktop application targets 4,194,304 T-cycles per second and sleeps only for the unused portion of each host frame. The APU uses a fractional sample clock so that 4,194,304 emulated T-cycles produce exactly 48,000 audio output frames per second, despite that ratio not being an integer.

## Timing granularity

Rustboy is **T-cycle-accounted**, not yet **M-cycle-executed**. Each `Machine::step` call completes one CPU instruction, then advances devices by that instruction's total T-cycle count. This keeps device clocks and instruction durations synchronized, and components such as the timer can process individual T-cycle edges inside that total.

The core does not yet schedule every CPU memory read, write, and register update at its exact M-cycle bus phase. Hardware effects that depend on the ordering of a CPU write within a single M-cycle, such as some TIMA reload and HALT edge cases, therefore require future M-cycle or finer-grained CPU scheduling.

## The Game Boy machine

A Game Boy program is a ROM cartridge containing instructions and game data. The machine runs those instructions while its hardware components share one 16-bit address space.

- **CPU**: fetches and executes the Game Boy's 8-bit instruction set. Instructions take a known number of clock cycles, which drives all other devices.
- **MMU**: the memory management unit routes each address to the appropriate device. For example, `0x8000..=0x9FFF` is video memory, while `0xFF00` is the joypad register.
- **Cartridge and MBC**: ROMs can be larger than the CPU's directly addressable cartridge window. Memory bank controllers switch ROM and save-RAM banks as games write controller registers.
- **PPU**: the picture processing unit reads video RAM, tile maps, palettes, and sprite data to produce the Game Boy's 160 by 144 pixel frame. It also raises display-related interrupts at precise points in the frame.
- **APU**: the audio processing unit combines two square-wave channels, a programmable wave channel, and a noise channel. It produces stereo sample buffers from the same CPU-cycle clock.
- **Timer**: maintains the divider and programmable timer registers used by games for timing, music, and interrupts.
- **Joypad**: presents button presses through the `FF00` hardware register and can request an interrupt when input changes.
- **Serial**: models the serial transfer registers used by link-cable software and many test ROMs.

## Why it is deterministic

The core contains no SDL, filesystem access, ZIP handling, host threads, channels, sleeps, wall-clock time, or random initialization. Given the same cartridge bytes, input events, and sequence of `step` calls, it produces the same machine state and effects. This makes it suitable for ROM conformance tests and for the future debugger.

## Boundary

This crate intentionally does not load files or display anything. Those concerns belong to adapters:

- `rustboy-adapter-rom` turns a filesystem path or ZIP archive into a `Cartridge`.
- `rustboy-application` delivers `Machine::step` effects to ports.
- `rustboy-adapter-sdl3` implements those ports with SDL3.
