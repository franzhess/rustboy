# rustboy-core

`rustboy-core` is the deterministic Game Boy machine. It models the hardware inside the handheld without knowing how a ROM was loaded, how pixels are displayed, how audio reaches speakers, or how quickly the host computer runs.

The public entry point is `Machine`. Create a `Cartridge` with `Cartridge::from_bytes`, put it into a `Machine`, then call `Machine::step`. Each step handles one CPU instruction, services an interrupt, or idles. It returns its cycle count, any CPU diagnostic, and completed video/audio output. A frontend, test runner, or debugger decides what to do with those effects.

## Module guides

The directories inside this crate are Rust modules, not separate Cargo crates:

- [CPU](src/cpu/README.md): registers, instruction execution, interrupts, and timing tests.
- [APU](src/apu/README.md): sound channels, register layout, clocks, and sample generation.
- [Cartridge/MBC](src/mbc/README.md): cartridge types, bank registers, and memory mapping.

The single-file modules are covered below: [MMU](#memory-bus-mmu),
[PPU](#display-ppu), [timer](#timer), [joypad](#joypad), and [serial](#serial).

## Timing and T-cycles

The original Game Boy is synchronized by a 4,194,304 Hz master clock. A **T-cycle** is one tick of that clock and is the smallest timing unit used by the core. CPU instructions take a multiple of four T-cycles; four T-cycles make one CPU **M-cycle** (machine cycle).

The CPU, timer, PPU, and APU advance from the same T-cycle count. `Machine::step` lends the MMU to `Cpu::tick`, then passes the returned T-cycle total to the MMU's timer, PPU, and APU exactly once. Interrupt entry costs 20 T-cycles and idle costs 4; both advance these devices just as instructions do. This ties device progress to CPU execution rather than the host computer's wall clock. STOP and illegal-opcode idle currently use this same timing approximation.

Frontends map emulated T-cycles to real time. The desktop application targets 4,194,304 T-cycles per second and sleeps only for the unused portion of each host frame. The APU uses a fractional sample clock so that 4,194,304 emulated T-cycles produce exactly 48,000 audio output frames per second, despite that ratio not being an integer.

## Timing granularity

Rustboy is **T-cycle-accounted**, not yet **M-cycle-executed**. Each `Machine::step` completes an instruction, interrupt entry, or idle step, then advances devices by that step's total T-cycle count. Components such as the timer process individual T-cycle edges inside that total.

The core does not yet schedule every CPU memory read, write, and register update at its exact M-cycle bus phase. Hardware effects that depend on the ordering of a CPU write within a single M-cycle, such as some TIMA reload and HALT edge cases, therefore require future M-cycle or finer-grained CPU scheduling.

## The Game Boy machine

A Game Boy program is a ROM cartridge containing instructions and game data. The machine runs those instructions while its hardware components share one 16-bit address space.

- **CPU**: fetches and executes the Game Boy's 8-bit instruction set. Instructions take a known number of clock cycles, which drives all other devices.
- **MMU**: the memory management unit routes each address to the appropriate device. For example, `0x8000..=0x9FFF` is video memory, while `0xFF00` is the joypad register.
- **Cartridge and MBC**: ROMs can be larger than the CPU's directly addressable cartridge window. Memory bank controllers switch ROM and save-RAM banks as games write controller registers.
- **PPU**: the picture processing unit reads video RAM, tile maps, palettes, and sprite data to produce the Game Boy's 160 by 144 pixel frame. The current implementation uses a simplified scanline/mode schedule for display-related interrupts.
- **APU**: the audio processing unit combines two square-wave channels, a programmable wave channel, and a noise channel. It produces stereo sample buffers from the same CPU-cycle clock.
- **Timer**: maintains the divider and programmable timer registers used by games for timing, music, and interrupts.
- **Joypad**: presents button presses through `FF00` and records a local interrupt request; forwarding that request to the CPU is not implemented yet.
- **Serial**: currently stores a subset of the serial registers; timed link-cable transfers are not implemented.

## Public machine boundary

`Machine` owns `Cpu` and `Mmu` as siblings. The MMU owns the cartridge controller
and devices; the CPU owns registers and execution/interrupt-control state.

```text
Machine
├── Cpu: registers, flags, execution state, IME/EI, HALT fetch suppression
└── Mmu: address routing, RAM, cartridge, PPU, APU, timer, joypad, serial, IF/IE
```

`Machine::step` coordinates each instruction-sized batch in this order:

1. Consume device requests into IF.
2. Call `Cpu::tick(&mut mmu)` to execute an instruction, service an interrupt, or idle.
3. Advance devices by the returned T-cycle total, after CPU bus accesses complete.
4. Drain completed video/audio output and return the CPU result in `StepResult`.

Requests generated by step 3 are collected at the start of the next machine step.
The CPU borrows the bus for execution; it neither owns devices nor advances them.
Input goes directly from `Machine` to the MMU's joypad, and memory reads and output
collection also bypass the CPU. This ownership split preserves the existing
instruction-batched timing model.

`StepResult` contains:

| Field | Meaning |
| --- | --- |
| `cycles` | T-cycles consumed by this step |
| `opcode` | Base opcode fetched for execution (including illegal encodings), or `None` during interrupt entry/idle |
| `diagnostic` | Optional `CpuDiagnostic`, currently an illegal opcode's byte and fetch address |
| `frame` | `Option<Frame>`: a complete row-major 160 × 144 screen of shade indices `0–3` |
| `audio_buffers` | `Vec<AudioBuffer>`: completed interleaved left/right `i16` buffers at 48,000 stereo frames/second |

For a CB-prefixed instruction, `opcode` is `Some(0xCB)`. `next_opcode()` is only a
peek at the current PC: interrupt dispatch may happen before that byte is executed.
Tracing and ROM exit detection must use the step result, not the peek.

`cpu_state()` exposes `CpuState::{Running, Halted, Stopped, IllegalOpcode}`.
`next_opcode()` returns `None` in every non-running state. An illegal opcode emits
one diagnostic on its encounter step; subsequent idle steps emit none. The CPU
returns structured data rather than printing the diagnostic itself.

The distinct states currently preserve the old transition behavior: an enabled
pending interrupt request wakes any idle state, with IME deciding whether to service
it or resume execution. Accurate STOP power/input behavior and permanent hardware
illegal-opcode lockup remain future behavior changes.

`process_input` accepts button events without stepping the CPU; `read_byte` and `registers` expose observations
for a runner or debugger. Construction starts the CPU at `0x0100` with hard-coded
register defaults; there is no boot-ROM execution path or complete boot-ROM-state model.

## Output formats

[`src/output.rs`](src/output.rs) defines owned `Frame` and `AudioBuffer` values.
Their private `Vec` storage is validated on construction, and consumers receive
immutable slices rather than mutable access to the underlying data.

- **Video:** `Frame::try_from(Vec<u8>)` requires exactly `Frame::PIXEL_COUNT`
  (23,040) pixels. Each byte is a DMG shade: 0 is lightest and 3 darkest.
  `pixels()` is row-major, top-left first; pixel `(x, y)` is at
  `y * SCREEN_WIDTH + x`. These are shade indices, not RGB or RGBA bytes.
- **Audio:** `AudioBuffer::try_from(Vec<i16>)` requires complete stereo pairs:
  `[left0, right0, left1, right1, ...]`. The signed PCM values use the full `i16`
  representation, with no byte-order serialization implied. `AUDIO_CHANNELS` is 2;
  `AUDIO_OUTPUT_FREQUENCY` is 48,000 stereo frames per emulated second, or 96,000
  individual samples. `samples()` exposes the interleaved values; `frame_count()`
  counts pairs. Empty buffers are valid. Buffer length is independent of video timing.

Invalid frame dimensions/shades produce `FrameError`; incomplete stereo pairs
produce `AudioBufferError`. Both implement `std::error::Error`. Successful wrapping
and `into_pixels()`/`into_samples()` preserve the original vector allocation and
capacity. The PPU still copies its screen into one vector when publishing a frame;
the APU still moves completed sample vectors out with `mem::take`.

The core transfers ownership of completed outputs to `StepResult`. Synchronous
application sinks borrow these values as `&Frame` and `&AudioBuffer`; the step
result owns their storage throughout delivery. A backend retaining data beyond
the call must own its copy, as SDL does for queued audio playback.

Run format and storage-preservation tests with `cargo test -p rustboy-core output::tests`.

## Memory bus (MMU)

[`src/mmu.rs`](src/mmu.rs) routes the shared 16-bit CPU address space:

| Address | Destination |
| --- | --- |
| `0000–7FFF` | Cartridge ROM reads / MBC control writes |
| `8000–9FFF` | PPU video RAM |
| `A000–BFFF` | Cartridge RAM (mapper receives address minus `0xA000`) |
| `C000–DFFF` | Work RAM |
| `E000–FDFF` | Echo of work RAM at `C000–DDFF` |
| `FE00–FE9F` | PPU sprite attributes (OAM) |
| `FEA0–FEFF` | Unusable region; currently reads zero and ignores writes |
| `FF00` | Joypad |
| `FF01–FF02` | Serial registers |
| `FF04–FF07` | Timer registers |
| `FF0F` | Interrupt requests (IF) |
| `FF10–FF3F` | APU registers and wave RAM |
| `FF40–FF4B` | PPU registers, except `FF46` |
| `FF46` | OAM DMA control, handled by the MMU |
| `FF80–FFFE` | High RAM (127 bytes) |
| `FFFF` | Interrupt enable (IE) |

Other addresses currently read zero and ignore writes; this is not a complete
hardware open-bus/unimplemented-register model. Work RAM has a larger backing array
than the 8 KiB exposed by the DMG map; that does not implement CGB RAM banking.

Word accesses are little-endian and wrap the second byte from `FFFF` to `0000`.
`process_irq_requests` collects device requests into IF at the start of a CPU
step. The request flags are private to their devices; the MMU consumes them through:

| Device method | IF bit |
| --- | --- |
| `Ppu::take_vblank_interrupt()` | 0: VBlank |
| `Ppu::take_stat_interrupt()` | 1: LCD STAT |
| `Timer::take_interrupt()` | 2: Timer |

Each method returns the current request and clears only that device-local flag.
Repeated collection without a new request does not reassert an acknowledged IF
bit. The MMU ORs requests into IF, preserving other pending bits even when IE is
zero. Consuming a device request is separate from acknowledging IF: IF remains set
until interrupt service or a software write clears it. Frame delivery and timer
registers are independent of request consumption.

Request generation and collection timing are unchanged, including the PPU's
simplified mode/LYC logic. Serial and joypad request forwarding is still TODO.
MMU tests cover individual bit mapping, simultaneous requests and acknowledgement;
device tests exercise consuming requests and raising subsequent ones.

### OAM DMA

Writing a byte to `FF46` selects source address `value << 8`. The MMU immediately
copies `0xA0` bytes from there into `FE00–FE9F`. This reproduces the bulk copy,
not hardware DMA timing: transfers are not spread over 160 M-cycles, and CPU bus
restrictions during DMA are not modeled.

## Display (PPU)

[`src/ppu.rs`](src/ppu.rs) owns 8 KiB of VRAM and 160 bytes of OAM. Each of the
40 sprite entries has four bytes: Y, X, tile number, and attributes. Hardware sprite
coordinates include offsets of +16 vertically and +8 horizontally; clipping must
account for these rather than allowing subtraction to wrap.

| Register | Function |
| --- | --- |
| `FF40` / LCDC | LCD, background/window, sprites, tile maps, and tile-addressing controls |
| `FF41` / STAT | LCD mode, LY=LYC status, and STAT interrupt enables |
| `FF42–FF43` / SCY, SCX | Background scrolling |
| `FF44–FF45` / LY, LYC | Current scanline and comparison value |
| `FF47–FF49` / BGP, OBP0, OBP1 | Background and sprite palettes |
| `FF4A–FF4B` / WY, WX | Window position |

Tile maps occupy `9800–9BFF` or `9C00–9FFF`: each is 32 × 32 tile indices.
An 8 × 8 tile uses sixteen bytes, two bitplanes per row. For each pixel, one bit
from each plane forms a color index `0–3`; a palette register maps that index to
a two-bit shade. Background tile addressing is either unsigned from `8000`, or
signed relative to `9000`. Sprite color zero is transparent. A separate color
buffer retains background indices for sprite/background priority checks.

The nominal frame schedule is 456 T-cycles per line, with 144 visible lines and
10 VBlank lines: `456 × 154 = 70,224` T-cycles, approximately 59.73 frames/second.

| STAT mode bits | `PpuMode` | Nominal role in the current simplified schedule |
| --- | --- | --- |
| 2 | `OamSearch` | OAM search, first 80 T-cycles of a visible line |
| 3 | `PixelTransfer` | Pixel transfer, next 172 T-cycles; the implementation renders the whole line on entry |
| 0 | `HBlank` | HBlank, remaining 204 T-cycles |
| 1 | `VBlank` | VBlank, lines 144–153; entering it publishes a frame and requests VBlank |

The mode is stored as a `PpuMode` enum, whose explicit discriminants encode STAT
bits 0–1. STAT writes only change interrupt enables, not the current mode. Mode-entry
effects use an exhaustive match: OAM search and HBlank request STAT when enabled;
pixel transfer renders a line; VBlank publishes a frame, requests VBlank, and
requests STAT if its mode-1 enable is set.

The current initial mode is HBlank even though LCDC starts enabled. Disabling the
LCD resets LY and the mode to HBlank on the next device tick, without running
HBlank entry effects. The existing clock comparisons remain inclusive: visible
line clocks `<= 80` select OAM search, `81–252` select pixel transfer, and `253–455`
select HBlank. These are implementation thresholds rather than cycle-exact hardware
edges. Regression tests preserve these thresholds, STAT encoding, interrupt requests,
single-frame publication at VBlank entry, and the LCD-disable reset behavior.

Run the PPU tests with `cargo test -p rustboy-core ppu::`.

Mode selection is evaluated at step boundaries using fixed thresholds, not a pixel
FIFO. Sprite/scroll-dependent transfer durations and exact STAT edge behavior are
not fully modeled. VRAM/OAM access restrictions are not enforced. The renderer
supports background tiles and basic sprites with flipping, palettes, and clipping,
but `render_window` is still a stub and the ten-sprites-per-line limit is not implemented.

## Timer

[`src/timer.rs`](src/timer.rs) uses one shared 16-bit divider, incremented every
T-cycle. DIV is not an independent counter:

| Register | Function |
| --- | --- |
| `FF04` / DIV | Upper eight divider bits; any write resets the entire divider |
| `FF05` / TIMA | Programmable eight-bit counter |
| `FF06` / TMA | Value copied into TIMA after overflow |
| `FF07` / TAC | Enable in bit 2; divider-bit selection in bits 0–1 |

| TAC selection | Divider bit | T-cycles per TIMA increment | Frequency |
| --- | --- | --- | --- |
| `00` | 9 | 1024 | 4,096 Hz |
| `01` | 3 | 16 | 262,144 Hz |
| `10` | 5 | 64 | 65,536 Hz |
| `11` | 7 | 256 | 16,384 Hz |

TIMA increments on a **falling edge** of `enabled AND selected divider bit`.
Resetting DIV, changing the selected bit, or disabling the timer can produce that
edge immediately. The timer iterates over each T-cycle inside a CPU step to retain
edges even when the CPU advances by a whole instruction.

Overflow exposes zero for four T-cycles, then reloads TIMA from TMA and requests
the timer interrupt. A TIMA write during the pending delay cancels the reload and
request; a TMA write changes the reload value. Disabling TAC does not cancel a
pending reload. Exact write priority on the reload cycle remains limited by the
CPU's instruction-sized bus scheduling.

## Joypad

[`src/joypad.rs`](src/joypad.rs) stores eight button states supplied through
`Machine::process_input`. The JOYP/P1 register at `FF00` uses active-low selection
and button bits: zero means selected or pressed.

| Low bit | Direction group (select bit 4) | Button group (select bit 5) |
| --- | --- | --- |
| 0 | Right | A |
| 1 | Left | B |
| 2 | Up | Select |
| 3 | Down | Start |

The implementation keeps only one selected group. It prioritizes the button group
when both select bits are clear, and retains the previous group when neither is
selected; simultaneous/no-selection hardware behavior is not fully implemented.
A released-to-pressed event sets `irq_joypad`, but the MMU does not yet forward
that flag into IF.

## Serial

[`src/serial.rs`](src/serial.rs) is a register placeholder. It stores the last byte
written to SB (`FF01`); SC (`FF02`) only records writes equal to `0x81`.
On DMG hardware, that value requests a transfer using the internal clock. Rustboy
does not yet shift bits, clock a transfer, clear the start bit on completion, or
request a serial interrupt. There is no link-cable peer or serial-output port.

## Why it is deterministic

The core contains no SDL, filesystem access, ZIP handling, host threads, channels, sleeps, wall-clock time, or random initialization. Given the same cartridge bytes, input events, and sequence of `step` calls, it produces the same machine state and effects. This makes it suitable for ROM conformance tests and for the future debugger.

## Boundary

This crate intentionally does not load files or display anything. Those concerns belong to adapters:

- `rustboy-adapter-rom` turns a filesystem path or ZIP archive into a `Cartridge`.
- `rustboy-application` delivers `Machine::step` effects to ports.
- `rustboy-adapter-sdl3` implements those ports with SDL3.

## Testing and references

Run the built-in regressions without external ROM files:

```sh
cargo test -p rustboy-core
```

Useful filters include `cpu::`, `apu::`, `mbc::`, `mmu::`, `ppu::`, and `timer::`.
Joypad and serial currently have no dedicated unit tests. Module guides describe
the existing coverage; a documented hardware rule is not automatically a claim
that every edge case is implemented or tested.

Workspace checks are `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
External suite setup and results are in [ROM conformance notes](../../docs/rom-conformance.md).
Hardware descriptions are available in [Pan Docs](https://gbdev.io/pandocs/).
