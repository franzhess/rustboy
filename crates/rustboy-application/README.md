# rustboy-application

`rustboy-application` turns a deterministic `rustboy-core::Machine` into an emulator session. It contains application policy, not SDL or Game Boy hardware implementation.

## Responsibilities

`Session` owns a machine, applies button events to it, and retains time-slice accounting. `advance_time_slice` advances a nominal 1/60 second of emulated CPU time and delivers completed frames and audio buffers as they become available. `run` polls input, advances time slices, and applies host pacing.

A time slice is not a Game Boy video frame. The PPU determines frame publication on its own approximately 59.73 Hz schedule. Cycle accounting is tested independently of wall-clock sleeping and requires no display or audio device.

## Time-slice accounting

`TIME_SLICES_PER_SECOND` is 60. The cumulative whole-T-cycle target after `n`
completed slices is:

```text
floor(n × CPU_FREQUENCY / TIME_SLICES_PER_SECOND)
```

Each nominal slice contributes 69,905 or 69,906 cycles. `Session` carries the
division remainder between slices so the four cycles left over from dividing
4,194,304 by 60 are not lost each second. It also credits the final instruction's
overshoot against the next slice. Cumulative execution therefore stays at or just
above the target, less than one final instruction ahead, rather than accumulating
an overrun on every slice. Individual calls can execute less than the nominal
quota when a prior instruction has already covered part of it.

`advance_time_slice` returns the cycles actually executed during that call and
never sleeps. It accounts for each step before diagnostic or output delivery. If
an output error interrupts an unfinished slice, retrying resumes the remaining
budget. Already delivered or failed outputs are not replayed. A new slice is only
budgeted after the previous cycle target has been reached.

`Session::step` remains an unpaced single-step API for headless runners and
debuggers; its calls do not debit the time-slice budget. Host pacing is separate:
`run` sleeps for the unused part of its nominal slice duration, or skips sleeping
when processing has already taken longer. Pacing uses per-slice elapsed time;
the core executes instruction-sized steps.

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

`advance_time_slice` and `run` return `RunError`. Its `Input`, `Frame`, and `Audio`
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
`CpuDiagnostic`. `advance_time_slice` takes a mutable diagnostic callback;
`run` takes a `FnMut(CpuDiagnostic)` callback and forwards it through each time-slice
advance. Events are delivered before that step's video/audio output, so a later
output failure cannot hide an already encountered CPU diagnostic.

The frontend decides how to report or record an event. A caller intentionally
ignoring diagnostics can pass `|_| {}` to `run`, or `&mut |_| {}` to `advance_time_slice`.
Diagnostics are distinct from `RunError`: reporting an illegal opcode does not
itself abort the run loop. The machine enters its explicit illegal-opcode state
and continues the existing idle/device advancement behavior. Call
`session.machine().cpu_state()` to inspect the current CPU state.
