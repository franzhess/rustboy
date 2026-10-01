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
  HALT (4 T-cycles). Its opcode is `None` for interrupt entry and HALT idle. Devices use CPU
  time, never host time.
- Normal instructions use an M-cycle execution bus. CPU accesses are sampled at the start of
  their four-T-cycle operation, then devices advance through that operation. Remaining duration
  is filled with internal M-cycles. Multi-byte reads and writes are separate bus operations.
- The model is not cycle-perfect: several instructions still place internal cycles at the end,
  and register updates are not independently phased. Normal PUSH, RST, CALL, and interrupt-entry
  stack writes are phased high byte then low byte.
- The timer advances its divider one T-cycle at a time inside each M-cycle to preserve selected-bit
  falling edges. TIMA reload-cycle write priority and some HALT edges still need finer ordering.

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
  It uses two internal M-cycles, writes PC high then low, and finishes with one internal M-cycle.
- The enabled request is selected after the high-byte write. An IE change there can cancel or
  reprioritize dispatch; the selection is latched before the low-byte write. Cancellation sets
  PC to zero without acknowledging IF. Successful entry acknowledges the selected request.
- Execution tracing and ROM exit-opcode detection use the opcode returned by `Machine::step`,
  not the speculative `next_opcode` peek.
- Mooneye `acceptance/intr_timing`, `acceptance/reti_intr_timing`, and
  `acceptance/interrupts/ie_push` pass.

## DMA Notes

- Fresh OAM DMA has a two-M-cycle startup: the `FF46` write M-cycle and one following
  accessible M-cycle. It then copies one byte per M-cycle for 160 M-cycles.
- Active DMA makes CPU OAM reads and opcode fetches return `FF` and ignores CPU OAM writes.
  DMA-internal accesses and untimed inspection bypass this CPU gate.
- Active-transfer `FF46` writes update readback immediately. The old transfer remains active
  through the replacement's two-M-cycle startup, then the replacement begins at offset zero.
- DMA-internal source pages `E0..=FF` alias `C0..=DF`; ordinary CPU accesses do not use
  this alias.
- Mooneye `oam_dma_start`, `oam_dma_timing`, `oam_dma_restart`, `oam_dma/basic`,
  `oam_dma/reg_read`, and `oam_dma/sources-GS` pass.

## EI and HALT Notes

- EI takes effect after the following instruction completes. Repeated EI does not postpone
  an existing enable; DI and interrupt entry cancel any pending enable. RETI enables IME
  immediately for interrupt dispatch at the next instruction boundary.
- HALT with IME clear and an enabled pending request suppresses one opcode-fetch PC increment.
  Otherwise HALT waits for an enabled request; with IME clear it resumes without servicing it.
- EI followed by HALT with a request already pending saves the HALT address on interrupt entry
  and clears the fetch-suppression flag before the handler runs.
- Mooneye `ei_sequence`, `ei_timing`, `rapid_di_ei`, `di_timing-GS`, `reti_timing`, and all
  four direct `acceptance/halt_*` tests pass.
- The EI/HALT integration makes `gbmicrotest/halt_op_dupe` and `int_hblank_halt_bug_b` pass,
  but changes `gbmicrotest/halt_bug` from passing to failing. That test sums timer reads:
  it now reports `0x16` instead of `0x14`. The old path incorrectly executed the post-HALT
  INC only once; the corrected path executes it twice. Investigate the remaining timing
  mismatch rather than removing fetch suppression. The opcode-duration audit corrected
  JP a16 to 16 T-cycles and ADD A,(HL) to 8, plus LD (HL+),A, SUB/CP (HL), and all CB (HL)
  durations. `gbmicrotest/halt_bug` still fails; `acceptance/jp_timing` now passes.
  All legal base and CB opcode durations are covered by unit tests across all flag combinations;
  this verifies instruction totals, not the ordering of bus operations within an instruction.

## PPU Notes

- Visible lines use fixed boundaries: mode 2 at dot 0, mode 3 at dot 80, HBlank at dot 252,
  and the next line at dot 456. Batched ticks process every crossed boundary in order.
- CPU OAM reads return `FF` and writes are ignored in modes 2 and 3. DMA and untimed internal
  accesses bypass this gate.
- STAT sources use one combined level signal and request an interrupt only on its rising edge.
  LY=LYC is latched while LCD is off, and the DMG mode-2 source also rises at VBlank entry.
- Mooneye `acceptance/ppu` passes 7 of 12 tests. `stat_lyc_onoff` still needs the LCD
  startup state; other remaining work covers startup/write timing and variable mode-3
  duration from SCX and sprite fetches.

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
