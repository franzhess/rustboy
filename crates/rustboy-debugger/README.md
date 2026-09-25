# rustboy-debugger

`rustboy-debugger` is the planned debugging application layer. It will depend on the deterministic core, not on SDL or terminal libraries.

## Why it is separate

Debugging is not part of Game Boy hardware. The core should execute instructions faithfully; the debugger should decide when to pause, what to inspect, and which events to record. This keeps ordinary emulation fast and keeps debugger policy reusable by a terminal UI, a graphical UI, and ROM-test failure reports.

## Planned responsibilities

- Single-instruction and cycle-aware stepping.
- Register, memory, and cartridge-state inspection.
- Breakpoints based on program counter or execution location.
- Watchpoints for memory reads and writes.
- Trace capture around a ROM-test failure.
- Disassembly coordination using the instruction information returned by core stepping.

The synchronous `Machine::step` API is the foundation for these features. The terminal interface will live in `rustboy-adapter-debugger-cli`.

The current facade re-exports `CpuState` and `CpuDiagnostic` alongside `Machine`,
`RegisterValues` and `StepResult`. A debugger can inspect `Machine::cpu_state()`
and record the optional diagnostic from each step without scraping terminal output.

It also re-exports `Frame` and `AudioBuffer` for inspecting the typed output values
in `StepResult`; their pixel/sample slices are immutable.
