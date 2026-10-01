# ROM conformance baseline

## OAM DMA restart timing — 2026-09-30

Writing `FF46` during active OAM DMA now starts a two-M-cycle replacement delay.
The old transfer continues during both M-cycles, OAM remains continuously blocked,
and the replacement source then begins a fresh 160-M-cycle transfer at offset zero.
Repeated writes replace the pending source and reset its activation deadline.

`acceptance/oam_dma_restart` now passes without regressions. The eligible Mooneye
acceptance result is **42/62**, with **24/29** direct tests passing. All direct DMA
timing tests now pass; `acceptance/oam_dma/sources-GS` remains deferred to source-page
alias handling.

## Fresh OAM DMA timing — 2026-09-30

OAM DMA now has a two-M-cycle startup followed by 160 one-byte transfer M-cycles.
CPU OAM accesses remain available for the first M-cycle after `FF46`, then active
DMA returns `FF` for reads and ignores writes. Internal DMA accesses bypass the CPU
gate, and `FF46` readback remains independent from transfer state.

The eligible Mooneye acceptance result is now **41/62**, with **23/29** direct tests
passing. `acceptance/oam_dma_start` and `acceptance/oam_dma_timing` newly pass, while
`acceptance/oam_dma/basic` and `acceptance/oam_dma/reg_read` remain passing. Restart
source replacement and source-page aliases remain separately scoped failures.

## CPU M-cycle bus seam — 2026-09-30

Normal CPU instructions now advance devices through individual bus and internal
M-cycles instead of one post-instruction batch. Multi-byte reads are split into byte
operations; interrupt entry remains instruction-batched pending its dedicated fix.

The eligible Mooneye acceptance baseline improved from **30/62** to **39/62** with
no regressions. `pop_timing`, `add_sp_e_timing`, `ld_hl_sp_e_timing`, `jp_timing`,
`jp_cc_timing`, `call_timing`, `call_cc_timing`, `ret_timing`, and `reti_timing`
now pass. POP's low and high stack reads occur on separate M-cycles and observe the
intervening DIV edge. Workspace tests cover every legal opcode's total duration;
focused tests also cover device advancement and split POP reads.

## MBC ROM time allowance — 2026-09-20

The runner now gives opcode-based Mooneye `emulator-only/mbc1`, `mbc2`, and `mbc5` tests
a minimum budget of 30 emulated seconds. Longer metadata limits are preserved. Other
groups and time-only assertions keep their original metadata timing or five-second default.
Three runner unit tests cover these boundaries; all 52 workspace unit tests pass on `main`.

A temporary worktree combined the opcode-duration corrections, this allowance, and all five
MBC fix commits (`2a0f865`, `52a91a5`, `0b94e44`, `89413ca`, `7d50a55`). `cargo rom-tests -- mbc`
then reported **30 passed, 1 failed**. All seven former MBC register-sweep timeouts passed.
The remaining failure is the unsupported MBC3 RTC cartridge type `0x10`. The MBC fixes still
live on their indexed review branches; this result describes the combined verification,
not the mapper implementation currently on `main`.

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
