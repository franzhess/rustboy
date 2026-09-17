# rustboy-application

`rustboy-application` turns a deterministic `rustboy-core::Machine` into an emulator session. It contains application policy, not SDL or Game Boy hardware implementation.

## Responsibilities

`Session` owns a machine and applies button events to it. `advance_frame` advances approximately one 60 Hz Game Boy frame worth of CPU cycles, then delivers any completed frame and audio buffers. `run` polls inputs, advances frames, and applies host pacing.

The cycle budget is tested separately from wall-clock sleeping. That separation is important: the emulator must run the correct amount of Game Boy work per frame even when a host is slow or a test runs without a real display.

## Ports

Ports are small traits defined by the application and implemented by adapters:

- `InputSource` supplies Game Boy button events and a quit decision.
- `FrameSink` receives completed 160 by 144 pixel frames.
- `AudioSink` receives generated stereo sample buffers.
- `RomSource` creates a core `Cartridge` from an external source.

SDL3 implements the input, frame, and audio ports. The ROM test runner uses no-op frame and audio sinks because it only needs machine state assertions. Keeping these ports separate avoids a single frontend interface that must know about every possible input, display, audio, and testing concern.

## Boundary

This crate depends on `rustboy-core` only. It has no SDL dependency and no knowledge of paths, ZIP archives, terminal output, or debugger UI commands.
