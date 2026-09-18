# Rustboy Agent Guide

## Project Structure

Rustboy is an in-progress Rust Nintendo Game Boy (DMG) emulator.

- `crates/rustboy-core`: deterministic emulator core. It must not depend on SDL, filesystems,
  ZIP handling, threads, channels, wall-clock time, or host sleeps.
- `crates/rustboy-application`: session, frame, input, and audio port layer.
- `crates/rustboy-adapter-rom`: ROM and ZIP loading.
- `crates/rustboy-adapter-sdl3`: SDL3 display, input, and audio integration.
- `crates/rustboy-cli`: application composition root.
- `crates/rustboy-rom-tests`: headless external ROM conformance runner.
- `crates/rustboy-debugger` and `crates/rustboy-adapter-debugger-cli`: future debugger layers.

Keep emulation behavior in `rustboy-core`; adapters translate host I/O at the boundary.

## Timing Model

- The master clock is 4,194,304 Hz. The core uses T-cycles throughout; four T-cycles are one
  CPU M-cycle.
- `Machine::step` executes one instruction, services an interrupt (20 T-cycles), or idles in
  HALT (4 T-cycles), advancing devices by that step's total T-cycles. Its opcode is `None`
  for interrupt entry and HALT idle. Devices share this cycle count, never host time.
- The core is T-cycle-accounted but not M-cycle-executed. It does not yet schedule CPU bus reads,
  writes, and register updates within their exact M-cycle phases.
- Model hardware edges inside instruction-sized batches when required. The timer, for example,
  advances its divider one T-cycle at a time to preserve selected-bit falling edges.
- Do not claim or implement cycle-perfect behavior without adding the required CPU scheduling.
  TIMA reload-cycle write priority and some HALT edge cases need M-cycle-level ordering.

## Timer Notes

- DIV and TIMA share a 16-bit divider. `FF04` exposes its upper byte.
- TAC selects divider bits 9, 3, 5, and 7 for frequency settings 0 through 3; TIMA increments on
  a falling edge of the enabled selected signal.
- Writing DIV, changing TAC, and disabling TAC can all create a falling edge and increment TIMA.
- TIMA overflow exposes zero, then reloads from TMA and requests an interrupt four T-cycles later.
  Writing TIMA during the pending delay cancels the reload and interrupt; writing TMA changes
  the value used by the reload. Disabling TAC does not cancel a pending reload.
- Mooneye `acceptance/timer` passes 11 of 13 tests, including `tima_reload`.
  `tima_write_reloading` and `tma_write_reloading` still fail: reload-cycle write priority
  requires CPU M-cycle scheduling and corresponding timer reload-cycle handling.

## Interrupt Notes

- Interrupt entry consumes 20 T-cycles without executing a handler instruction in that step.
  It clears IME, acknowledges the highest-priority enabled request, and saves the return PC.
- Execution tracing and ROM exit-opcode detection use the opcode returned by `Machine::step`,
  not the speculative `next_opcode` peek.
- Mooneye `acceptance/intr_timing` and `acceptance/reti_intr_timing` pass.
  `acceptance/interrupts/ie_push` still fails; interrupt dispatch does not yet model changes
  to IE during the stack writes of the interrupt-entry sequence.

## EI and HALT Notes

- EI takes effect after the following instruction completes. Repeated EI does not postpone
  an existing enable; DI and interrupt entry cancel any pending enable. RETI enables IME
  immediately for interrupt dispatch at the next instruction boundary.
- HALT with IME clear and an enabled pending request suppresses one opcode-fetch PC increment.
  Otherwise HALT waits for an enabled request; with IME clear it resumes without servicing it.
- EI followed by HALT with a request already pending saves the HALT address on interrupt entry
  and clears the fetch-suppression flag before the handler runs.
- Mooneye `ei_sequence`, `ei_timing`, `rapid_di_ei`, and `halt_ime1_timing` pass. The other three
  `acceptance/halt_*` tests, `di_timing-GS`, and `reti_timing` still time out in the ROM runner.
- The EI/HALT integration makes `gbmicrotest/halt_op_dupe` and `int_hblank_halt_bug_b` pass,
  but changes `gbmicrotest/halt_bug` from passing to failing. That test sums timer reads:
  it now reports `0x16` instead of `0x14`. The old path incorrectly executed the post-HALT
  INC only once; the corrected path executes it twice. Investigate the remaining timing
  mismatch rather than removing fetch suppression. Existing opcode timing errors observed
  in its trace include JP a16 taking 12 instead of 16 T-cycles and ADD A,(HL) taking 4 instead
  of 8 T-cycles.

## Testing

The post-integration ROM baseline and follow-up notes are in `docs/rom-conformance.md`.

Run these checks after relevant changes:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

External ROM suites are intentionally ignored by Git. Install them with:

```sh
git clone https://github.com/adtennant/GameboyTestSuites.git roms/GameboyTestSuites
```

Run all eligible conformance tests with `cargo rom-tests`. Pass a path fragment to focus the
runner, for example:

```sh
cargo rom-tests -- acceptance/timer
```

Set `RUSTBOY_TEST_ROMS` to use an alternate ROM-suite checkout. The runner selects DMG ROMs
with register or memory assertions; tests without DMG support and screenshot-only tests are
excluded. Unsupported cartridges currently return an internal skipped outcome that the trial
wrapper reports as a failure; this runner-classification issue is tracked in the baseline notes.

## Git Workflow

- Keep `main` linear. Integrate reviewed branches with squash merges, not merge commits.
- Rebase dependent review branches whenever their parent commit is amended or recreated.
- Do not revert unrelated worktree changes. Inspect the current branch and status before editing.
