# rustboy-application

`rustboy-application` turns a deterministic `rustboy-core::Machine` into an emulator session. It contains application policy, not SDL or Game Boy hardware implementation.

## Responsibilities

`Session` owns a machine and applies button events to it. `advance_frame` advances approximately one 60 Hz Game Boy frame worth of CPU cycles, then delivers any completed frame and audio buffers. `run` polls inputs, advances frames, and applies host pacing.

The cycle budget is tested separately from wall-clock sleeping. That separation is important: the emulator must run the correct amount of Game Boy work per frame even when a host is slow or a test runs without a real display.

## Ports

Ports are small traits defined by the application and implemented by adapters:

- `InputSource` supplies Game Boy button events and a quit decision.
- `FrameSink` borrows a validated core `Frame`: 160 by 144 row-major
  DMG shade indices. `Frame::pixels()` exposes its immutable pixel slice.
- `AudioSink` borrows a core `AudioBuffer`: interleaved signed `i16`
  left/right samples at 48,000 stereo frames/second. `AudioBuffer::samples()` exposes
  its immutable sample slice and `frame_count()` counts complete stereo pairs.
- `RomSource` creates a core `Cartridge` from an external source, returning
  `Result<Cartridge, Self::Error>`. Each adapter supplies an associated `Error`
  type implementing `std::error::Error`, preserving structured failures and their
  causes through the port. Generic consumers can propagate `S::Error` for a source
  `S: RomSource`; trait objects specify `dyn RomSource<Error = E>`.

SDL3 implements the input, frame, and audio ports. The ROM test runner uses no-op frame and audio sinks because it only needs machine state assertions. Keeping these ports separate avoids a single frontend interface that must know about every possible input, display, audio, and testing concern.

`StepResult` owns the output values during delivery. The application passes
`&Frame` and `&AudioBuffer` to the sinks, borrowing the original buffers without
cloning them. Each borrow lasts for the sink call; a sink retaining data for later
processing must make an owned copy. Queued audio playback may be asynchronous,
but transfer from the borrowed buffer completes before `queue_audio` returns.
See the [core output contract](../rustboy-core/README.md#output-formats).

## Boundary

This crate depends on `rustboy-core` only. It has no SDL dependency and no knowledge of paths, ZIP archives, terminal output, or debugger UI commands.

## Platform errors

`InputSource`, `FrameSink`, and `AudioSink` each declare their own associated
`Error: std::error::Error + 'static` type. Adapters return concrete errors rather
than strings; ports that cannot fail use `std::convert::Infallible`. A combined
`Platform` may use a different error type for each port.

`advance_frame` and `run` return `RunError`. Its `Input`, `Frame`, and `Audio`
variants identify the failed operation, while `Error::source()` exposes the
original adapter error for inspection or downcasting. Causes are boxed only when
an operation fails, keeping SDL-specific types out of this crate. `Display` adds
operation context, and the CLI supplies the outer user-facing diagnostic.

A failure returns immediately: polling failures prevent input draining and machine
advancement; output failures stop further stepping/delivery in that call. Already
completed machine steps and delivered output are not rolled back. Tests inject
failures for each operation without opening devices or relying on host sleeps.

## CPU diagnostics

`Session::step` returns the core's `StepResult`, including its optional
`CpuDiagnostic`. `advance_frame` takes a mutable diagnostic callback;
`run` takes a `FnMut(CpuDiagnostic)` callback and forwards it through each frame
advance. Events are delivered before that step's video/audio output, so a later
output failure cannot hide an already encountered CPU diagnostic.

The frontend decides how to report or record an event. A caller intentionally
ignoring diagnostics can pass `|_| {}` to `run`, or `&mut |_| {}` to `advance_frame`.
Diagnostics are distinct from `RunError`: reporting an illegal opcode does not
itself abort the run loop. The machine enters its explicit illegal-opcode state
and continues the existing idle/device advancement behavior. Call
`session.machine().cpu_state()` to inspect the current CPU state.
