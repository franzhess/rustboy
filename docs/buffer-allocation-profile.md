# Frame and audio allocation profile

This profile measures heap requests made while the core produces owned frame and
audio outputs. It does not measure SDL texture uploads or audio queue internals.

Run the reproducible probe in release mode:

```sh
cargo run --release -p rustboy-core --example buffer_allocations
```

The probe runs the same synthetic `JR` loop for ten emulated seconds in two
machines. One has the LCD disabled to establish the audio baseline; the other has
the LCD enabled. Subtracting the baseline isolates frame production. The counting
allocator reports calls to `alloc`, `realloc`, and `dealloc`. "Requested bytes" is
the sum of allocation sizes and new reallocation sizes, not peak or resident memory.

## Baseline

Measured on 2026-09-29 with `rustc 1.98.1` on arm64 macOS 26.6.2:

| Output | Count (10 s) | Allocations | Reallocations | Requested bytes |
| --- | ---: | ---: | ---: | ---: |
| Audio | 600 buffers | 1,200 | 5,400 | 4,968,000 |
| Video | 597 frames | 597 | 4,776 | 48,810,720 |
| Combined | 597 frames, 600 buffers | 1,797 | 10,176 | 53,778,720 |

The second run produced identical counters.

- Each 23,040-byte frame performs one allocation and eight growth reallocations.
  Flattening the row arrays through `collect` does not reserve the known final size,
  so it requests 81,760 bytes across growth steps per frame.
- Each 1,600-sample audio buffer performs two allocations and nine growth
  reallocations. One allocation is the completed-buffer list; the sample vector
  starts empty after every `mem::take` and repeatedly grows to its final capacity.
- Combined production averages 1,197.3 allocation/reallocation calls and 5,377,872
  requested bytes per emulated second. Output wrappers and application delivery do
  not copy either payload.

## Follow-up optimization task

Pre-size the core's output producer vectors without changing output ownership or
public APIs:

1. Build each PPU frame in a vector with `Frame::PIXEL_COUNT` capacity so frame
   production performs no growth reallocations.
2. Replace the completed APU sample vector with a new vector preallocated for
   `AUDIO_BUFFER_SAMPLES` instead of replacing it with an empty allocation.
3. Rerun this probe and require zero payload growth reallocations, unchanged output
   counts, and unchanged workspace tests.

The remaining allocation for each owned frame and audio payload is expected. The
small outer `Vec<AudioBuffer>` allocation can be assessed separately after the
payload growth is removed; changing `StepResult` solely to eliminate it is not yet
justified.
