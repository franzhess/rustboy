# APU: audio channels and sample generation

This module implements the Game Boy's four-channel audio system inside
[`rustboy-core`](../../README.md). It advances from CPU T-cycles and returns
sample buffers; it does not open an audio device, sleep, or manage host playback.

## Source layout and channels

| File | Responsibility |
| --- | --- |
| [`mod.rs`](mod.rs) | Register routing, frame sequencer, sample clock, envelopes, stereo mixer |
| [`tone.rs`](tone.rs) | Pulse channels 1 and 2; channel 1's frequency sweep |
| [`wave.rs`](wave.rs) | Channel 3's programmable 32-sample waveform |
| [`noise.rs`](noise.rs) | Channel 4's shift-register noise generator |

Channels 1 and 2 share the `Tone` implementation. Both have pulse duty selection,
length control, and a volume envelope; only channel 1 receives sweep clocks.
Channel 3 reads a programmable waveform rather than a pulse table. Channel 4
generates deterministic pseudo-noise, not samples from a host random-number generator.

## Memory-mapped registers

The MMU routes `FF10–FF3F` to the APU:

| Address | Hardware name | Purpose |
| --- | --- | --- |
| `FF10` | NR10 | Channel 1 sweep |
| `FF11–FF14` | NR11–NR14 | Channel 1 duty/length, envelope, frequency, trigger |
| `FF16–FF19` | NR21–NR24 | Channel 2 duty/length, envelope, frequency, trigger |
| `FF1A–FF1E` | NR30–NR34 | Channel 3 DAC control, length, output level, frequency, trigger |
| `FF20–FF23` | NR41–NR44 | Channel 4 length, envelope, noise period/width, trigger |
| `FF24` | NR50 | Left/right output volume and hardware VIN controls |
| `FF25` | NR51 | Route each channel to left/right output |
| `FF26` | NR52 | Global audio power and channel-active status |
| `FF30–FF3F` | Wave RAM | Sixteen packed bytes, holding 32 four-bit samples |

Register names describe the hardware interface. Several reads currently return
zero rather than implementing the hardware's read masks or wave-RAM visibility.
Unmapped addresses within the APU range read zero and ignore writes.

## Three kinds of clock

The APU has three distinct timing responsibilities:

1. **Channel timers** advance waveform positions at the programmed frequency.
2. A **512 Hz frame sequencer** clocks length counters, sweep, and envelopes.
3. A **48 kHz output sample clock** decides when to emit stereo samples to the host.

`Apu::do_ticks` currently advances the frame sequencer first, then the four channel
timers, then emits samples from their resulting state. This is instruction-batched
processing, not event-by-event interleaving of every channel edge and output sample.

### Frame sequencer

`TIMER_TICKS = CPU_FREQUENCY / 512 = 8192` T-cycles. `timer_counter` accumulates
cycles, and each elapsed interval advances `timer_step` modulo eight:

| Sequencer steps | Action | Clock rate |
| --- | --- | --- |
| 0, 2, 4, 6 | Clock all channel length counters | 256 Hz |
| 2, 6 | Clock channel 1 sweep | 128 Hz |
| 7 | Clock channel 1/2/4 envelopes | 64 Hz |

Here `timer_step()` on a channel means a **length-counter tick**, not a CPU T-cycle
or waveform step. The sequencer is currently driven by its own accumulated cycle
counter; coupling to the hardware divider's edges and DIV-write effects is not modeled.

### Output sample clock

4,194,304 CPU cycles do not divide evenly into 48,000 output frames. `SampleClock`
keeps the fractional remainder using integer arithmetic:

```text
phase += elapsed_T_cycles × 48,000
frames_due = phase / 4,194,304
phase %= 4,194,304
```

Discarding the remainder or rounding to a fixed cycles-per-sample interval would
cause drift. Keeping it produces exactly 48,000 stereo frames per emulated second.

## Pulse channels (`Tone`)

The two duty bits select one of four eight-step patterns, conventionally described
as 12.5%, 25%, 50%, and 75% duty cycles. The implementation stores each pattern as
`-1`/`+1` samples and multiplies the selected phase by the envelope volume.

The frequency register holds an eleven-bit value `f`, not a frequency in Hz:

```text
T-cycles per waveform step = (2048 - f) × 4
T-cycles per complete pulse = 8 × step period
```

Larger register values therefore yield higher pitches. Low and high register
writes preserve the other part of `f`; the high write uses only bits 0–2 for
frequency. Bit 6 enables length control, and bit 7 triggers the channel.

The channel 1 sweep periodically computes an offset `f >> shift` and adds or
subtracts it according to NR10. The present sweep implementation is simplified:
shadow-frequency, trigger-time overflow checks, and exact counter behavior are
not fully modeled.

## Length counters and envelopes

Length registers store a **complement**, not a literal duration:

| Channel | Length loaded by a register write | Maximum |
| --- | --- | --- |
| Pulse 1/2 | `64 - (value & 0x3F)` | 64 length ticks |
| Wave | `256 - value` | 256 length ticks |
| Noise | `64 - (value & 0x3F)` | 64 length ticks |

With length control enabled, ticks decrement the counter and disable the channel
at zero. A trigger reloads an empty counter to its maximum; a trigger with a
nonzero counter preserves the remaining duration. Pulse/noise triggers also reset
the envelope to its initial volume. Frame-sequencer-phase-dependent extra length
clocks are not implemented.

`VolumeEnvelope` decodes initial volume from bits 4–7, direction from bit 3, and
period from bits 0–2. At each configured number of 64 Hz envelope clocks it steps
volume toward 15 or zero. The current implementation treats period zero as no
automatic volume change. Wave output uses its own level control, not this envelope.

## Wave channel (`Wave`)

Each byte of wave RAM contains two four-bit samples, high nibble first. A write
unpacks them and subtracts eight, storing signed values from `-8` through `7` in
the internal 32-entry waveform. The cursor advances modulo 32:

```text
T-cycles per waveform step = (2048 - f) × 2
```

Hardware output-level codes are mute, full, half, and quarter amplitude. The
current implementation instead uses the nonzero level code directly as a right
shift on its signed sample; exact hardware amplitude/DAC behavior is not yet
reproduced. Wave RAM reads currently return zero. Active-playback access rules,
trigger phase, and DAC-enable state are also incomplete.

## Noise channel (`Noise`)

A linear-feedback shift register (LFSR) generates a repeatable bit sequence.
The low output bit selects positive or negative envelope amplitude. NR43 supplies
a divisor code, a width-mode bit, and a clock shift:

```text
base period = 8 if divisor code is 0, otherwise 16 × divisor code
T-cycles per LFSR step = base period << clock_shift
```

The hardware uses a 15-bit LFSR and can feed back into bit 6 for a shorter sequence.
The current code XORs bits 0 and 1, shifts right, inserts feedback at bit 15, and
ORs it into bit 6 in short mode. That differs from the hardware's bit-14 feedback
and bit-6 replacement. Exact noise feedback and trigger reset behavior remain
implementation gaps; the existing tests cover length handling, not noise conformance.

## Stereo mixing and output buffers

NR51 bits 0–3 route channels 1–4 to the right output; bits 4–7 route them to the
left. NR50 provides three-bit volume fields for each side. `Mixer::mix` sums the
enabled routes separately and multiplies each sum by its stored volume value.
VIN input is ignored. Hardware gain behavior, analog filtering, and DAC conversion
are not fully reproduced.

Samples leave the APU as interleaved signed sixteen-bit values:

```text
[left0, right0, left1, right1, ...]
```

`AUDIO_BUFFER_SAMPLES = 1600` counts individual `i16` values, so each completed
buffer holds **800 stereo frames**, about 16.67 ms at 48 kHz. The buffer duration
is not the same as a Game Boy video frame. `take_audio_buffers` drains completed
buffers; any partially filled buffer stays inside the APU. `Machine::step` returns
these buffers to the application layer for delivery to an audio sink.

## Current power-control limitations

NR52 bit 7 updates `Apu::enabled`, and reads report that flag plus the four channel
activity flags. However, `do_ticks` does not currently use the global flag to stop
channels or mute output, and power-off register resets/write restrictions are not
implemented. Treat the current APU as a basic sound generator with focused fixes,
not a complete DMG audio-hardware model.

## Tests and references

```sh
cargo test -p rustboy-core apu::
```

Existing regressions cover the one-second sample-clock total, complementary length
registers for pulse/wave/noise, and triggering an empty length counter. They do not
establish full sweep, envelope, DAC, wave-RAM, mixer, or noise timing conformance.
External ROM results are tracked in the
[conformance notes](../../../../docs/rom-conformance.md).

References: [Pan Docs: Audio](https://gbdev.io/pandocs/Audio.html) and
[Audio Registers](https://gbdev.io/pandocs/Audio_Registers.html).
