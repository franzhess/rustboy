# ROM conformance baseline

## Opcode-duration audit — 2026-09-20

The CPU timing regression tests cover all 244 legal unprefixed instructions and all 256
CB-prefixed instructions through `Cpu::tick`, with all 16 flag combinations. Conditional
JR/JP/CALL/RET instructions exercise both outcomes. The 11 illegal unprefixed opcodes are
checked as unsupported rather than assigned an instruction duration. Two additional tests
check that JP a16 and ADD A,(HL) advance the timer by their corrected durations.

The audit corrected five base opcodes (`22`, `86`, `96`, `BE`, `C3`) and all 32 CB-prefixed
`(HL)` operations: BIT takes 12 T-cycles, while read/modify/write operations take 16.
These are instruction-total corrections; CPU M-cycle bus scheduling remains future work.

Formatting, Clippy, and all 49 workspace unit tests pass. Focused ROM checks retain 11/13
timer passes and passes for `intr_timing` and `reti_intr_timing`. `acceptance/jp_timing`
still times out and `gbmicrotest/halt_bug` still fails its memory assertion.

## After review-branch integration — 2026-09-18

- Rustboy: `9a73680` (`fix: emulate EI and HALT timing`)
- GameboyTestSuites: `706ba6644e1a88553cd78181b17810814765c5c3` (clean checkout)
- Command: `cargo rom-tests`
- Result: **148 passed, 489 failed, 637 total** (86.13 seconds on the development host).
- Workspace checks: formatting and Clippy pass; **45 unit tests pass**.

These are the runner's raw results, including timeouts and unsupported-cartridge load outcomes.
They are a starting point for incremental fixes, not 489 independently diagnosed hardware bugs.
The runner selects tests with DMG metadata and memory or register assertions. Tests requiring
only screenshot comparison and tests without DMG support are excluded from these totals.

## Results relevant to the integrated branches

| Group | Result |
| --- | --- |
| Mooneye `acceptance/timer` | 11/13 pass; reload-cycle TIMA/TMA writes still fail |
| Mooneye `intr_timing`, `reti_intr_timing` | Both pass |
| Mooneye `interrupts/ie_push` | Fails; IE changes during interrupt-entry stack writes are not modeled |
| Mooneye `ei_sequence`, `ei_timing`, `rapid_di_ei` | All pass |
| Mooneye `halt_ime1_timing` | Passes |
| Mooneye `halt_ime0_ei`, `halt_ime0_nointr_timing`, `halt_ime1_timing2-GS` | Exit-opcode timeouts |
| Mooneye `di_timing-GS`, `reti_timing` | Exit-opcode timeouts |

Comparison against the integrated interrupt-service branch (`3b57845`) found three newly
passing cases in the EI/HALT-related subsets:

- `mooneye-test-suite/acceptance/ei_sequence`
- `gbmicrotest/halt_op_dupe`
- `gbmicrotest/int_hblank_halt_bug_b`

One case, `gbmicrotest/halt_bug`, changed from passing to failing. Its assertion sums timer
reads rather than checking the duplicated opcode directly. A trace shows that the old path
incorrectly executes the post-HALT `INC A` once; the corrected path executes it twice. The
timer-read sum consequently changes from the expected `0x14` to `0x16`. Keep the corrected
fetch suppression and investigate the remaining timing mismatch. The trace also exposes
existing instruction-duration errors: `JP a16` takes 12 instead of 16 T-cycles, and
`ADD A,(HL)` takes 4 instead of 8. Correcting these alone has not yet been verified to fix
the ROM.

## Follow-up order

1. Check runner classification before interpreting every failure as emulation behavior.
   In particular, `mealybug-tearoom-tests/mbc/mbc3_rtc` cannot load cartridge type `0x10`.
   `run_test` returns `Outcome::Skipped`, but the trial wrapper turns that into a failure.
2. Fix instruction-duration errors with focused CPU tests, beginning with the observed
   `JP a16` and `ADD A,(HL)` errors, then rerun the affected ROMs and the HALT regression.
3. Diagnose individual remaining CPU/interrupt and timer cases. Reload-cycle timer writes
   and interrupt-entry stack ordering require finer CPU scheduling; do not treat all
   timeouts as evidence of the same cause.
4. Continue with separately scoped memory-controller, DMA, and PPU failures.

For each fix, retain a focused regression test for the hardware rule and run the original
ROM. Compare full-suite results at useful milestones, including newly failing cases rather
than only comparing the total number of passes.
