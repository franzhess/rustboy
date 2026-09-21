# rustboy-adapter-sdl3

`rustboy-adapter-sdl3` connects the emulator application ports to SDL3. It is the playable desktop frontend adapter and contains no Game Boy emulation rules.

## What SDL3 provides

- **Input**: translates keyboard and gamepad events into Game Boy button presses and releases. Arrow keys map to the directional pad, `A` and `S` map to the A and B buttons, Space maps to Select, Return maps to Start, and Escape quits.
- **Display**: uploads the core's 160 by 144 pixel frame to an SDL renderer and scales it using integer scaling so Game Boy pixels remain crisp.
- **Audio**: sends the core's interleaved stereo `i16` sample buffers to an SDL audio stream.

`Sdl3Adapter` implements the application `InputSource`, `FrameSink`, and `AudioSink` ports. It does not decide how many Game Boy cycles to run; that is application policy.

## Requirements

SDL3 development libraries must be available to build this crate. See the workspace README for platform installation guidance.

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
