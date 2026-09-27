# rustboy-adapter-sdl3

`rustboy-adapter-sdl3` connects the emulator application ports to SDL3. It is the playable desktop frontend adapter and contains no Game Boy emulation rules.

## What SDL3 provides

- **Input**: translates keyboard and gamepad events into Game Boy button presses and releases. Arrow keys map to the directional pad, `A` and `S` map to the A and B buttons, Space maps to Select, Return maps to Start, and Escape quits.
- **Display**: converts the core's validated 160 by 144 `Frame` into a reusable
  RGBA streaming texture, then draws it with one texture copy using nearest-neighbor
  filtering and integer scaling.
- **Audio**: reads interleaved left/right samples from `AudioBuffer` and sends
  them to an SDL stream configured for `AUDIO_CHANNELS` (2) and
  `AUDIO_OUTPUT_FREQUENCY` (48,000 stereo frames/second), using native signed `i16` PCM.

`Sdl3Adapter` implements the application `InputSource`, `FrameSink`, and `AudioSink` ports. It does not decide how many Game Boy cycles to run; that is application policy.

The output ports and display/audio helpers borrow `&Frame` and `&AudioBuffer` for
each call. Rendering reads the frame into the locked texture; audio submission passes the sample
slice to SDL, which copies it into its stream for later playback. The Rust adapter
does not clone the producer's buffers or retain references after returning.

## Renderer and texture ownership

Create the context before the adapter:

```rust
use rustboy_adapter_sdl3::{Sdl3Adapter, Sdl3Context};

let mut context = Sdl3Context::new(320, 288)?;
let mut adapter = Sdl3Adapter::new(&mut context)?;
```

`Sdl3Context` owns SDL, the window renderer and its texture creator.
`Sdl3Adapter<'_>` borrows that context, keeping the creator alive until its texture
is dropped. The texture is allocated once when the adapter connects and reused
for every frame. Rust lifetimes enforce this ordering without leaked resources
or self-referential storage.

The palette remains `E0F8D0`, `88C070`, `346856`, `081820` for shades 0 through 3.
Conversion writes opaque RGBA bytes directly into the locked texture, honoring
SDL's row pitch, including padding. `RGBA32` specifies byte order portably across
host endianness. No temporary RGB vector is allocated per frame. The canvas keeps
its 160×144 integer logical presentation, and the texture uses nearest-neighbor
filtering so enlarged pixels stay sharp.

## Requirements

SDL3 development libraries must be available to build this crate. See the workspace README for platform installation guidance.

## Errors

`Sdl3Context::new` and `Sdl3Adapter::new` return `InitError`, distinguishing SDL
setup, window creation, logical-size configuration and texture creation failures.
It implements `std::error::Error` and exposes the original SDL error through `source()`.

Frame presentation, audio queuing, and the `play`/`stop` methods return concrete
`sdl3::Error` values. Input polling uses `Infallible`: the event pump is created
during initialization, and polling itself has no fallible result. Optional gamepad
failures retain the keyboard fallback described below.

The application wraps runtime port failures in `RunError::Frame`, `RunError::Audio`,
or `RunError::Input`, retaining their causes. No propagated SDL failure is converted
to a string in this adapter.

## Rendering verification and benchmark

`cargo test -p rustboy-adapter-sdl3` checks padded-pitch RGBA conversion and compares
SDL software-renderer pixel readback with the original per-pixel path at 1×, 2×,
and a letterboxed size. Consecutive frames invert every shade to verify texture
updates. These tests need no window or audio device.

An explicit lifecycle smoke test uses SDL's dummy drivers to create a window,
render a frame, queue audio, and drop/recreate an adapter against the same context:

```sh
SDL_VIDEO_DRIVER=dummy SDL_AUDIO_DRIVER=dummy cargo test -p rustboy-adapter-sdl3 context_can_create_and_drop_adapters_with_streaming_output -- --ignored --test-threads=1
```

Run the manual release-mode rendering benchmark with:

```sh
cargo test -p rustboy-adapter-sdl3 --release streaming_texture_rendering_benchmark -- --ignored --nocapture --test-threads=1
```

Measured on macOS/aarch64 using SDL's software renderer at 320×288, after ten
warm-up frames per path, taking the median of five 100-frame batches with alternating
measurement order:

| Path | Microseconds per frame |
| --- | ---: |
| Original per-pixel drawing | 944.2 |
| Streaming texture | 34.4 |

The measured rendering-only ratio is approximately **27.4×**. Timing includes
conversion, drawing and presentation, but excludes texture initialization, pixel
readback, emulation and application pacing. It measures the software backend, not
desktop GPU performance. Both manual checks are ignored by the default test run;
the benchmark has no timing assertion.

## Controllers

The adapter automatically opens the first available SDL-recognized gamepad at startup.
Controllers can also be connected while the emulator is running. One gamepad is active
at a time; connecting another does not replace it. If the active controller disconnects,
its held buttons are released and another available controller is selected.

| Controller input | Game Boy button |
| --- | --- |
| D-pad | Up / Down / Left / Right |
| South face button (Xbox A / PlayStation Cross) | A |
| East face button (Xbox B / PlayStation Circle) | B |
| Start / Menu | Start |
| Back / View / Share | Select |

Face buttons are mapped by physical position, not the printed Nintendo/Xbox labels.
This initial mapping uses the D-pad, not analog sticks, and has no remapping UI.
Keyboard input remains available alongside the controller: a button is held while
either source holds it. Gamepad initialization/open failures are reported to stderr
without disabling keyboard input.

### Verification

`cargo test -p rustboy-adapter-sdl3` checks button translation, duplicate presses,
overlapping keyboard/controller holds, disconnect releases, and inactive controller
events without requiring a physical gamepad.

For a hardware smoke test, launch a game with a controller connected, check the
mapping, then unplug it while holding a direction. Verify the direction releases,
reconnect and check input again. Also try connecting after startup and holding the
same button on keyboard and controller while releasing each source in turn.
