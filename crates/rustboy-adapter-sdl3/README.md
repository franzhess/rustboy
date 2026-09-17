# rustboy-adapter-sdl3

`rustboy-adapter-sdl3` connects the emulator application ports to SDL3. It is the playable desktop frontend adapter and contains no Game Boy emulation rules.

## What SDL3 provides

- **Input**: translates keyboard events into Game Boy button presses and releases. Arrow keys map to the directional pad, `A` and `S` map to the A and B buttons, Space maps to Select, Return maps to Start, and Escape quits.
- **Display**: uploads the core's 160 by 144 pixel frame to an SDL renderer and scales it using integer scaling so Game Boy pixels remain crisp.
- **Audio**: sends the core's interleaved stereo `i16` sample buffers to an SDL audio stream.

`Sdl3Adapter` implements the application `InputSource`, `FrameSink`, and `AudioSink` ports. It does not decide how many Game Boy cycles to run; that is application policy.

## Requirements

SDL3 development libraries must be available to build this crate. See the workspace README for platform installation guidance.
