# rustboy-rom-tests

`rustboy-rom-tests` runs external Game Boy conformance ROMs as native-style test cases. It is an acceptance-test adapter: each ROM drives the same deterministic core used by the SDL frontend, then asserts memory or register values described by suite metadata.

## Setup

The upstream suites are intentionally not committed. Clone them from the workspace root:

```sh
git clone https://github.com/adtennant/GameboyTestSuites.git roms/GameboyTestSuites
cargo rom-tests
```

Set `RUSTBOY_TEST_ROMS` to use a different checkout location.

## Emulated-time allowance

Opcode-based Mooneye `emulator-only/mbc1`, `mbc2`, and `mbc5` tests get at least
30 seconds of emulated CPU time so exhaustive register sweeps can finish. The runner
stops immediately when the exit opcode executes and preserves longer metadata limits.
Other tests use their metadata limit (or the five-second default). Time-only tests
always retain their specified assertion time. These budgets count CPU T-cycles, not
host wall-clock time.

## How it runs without SDL

The runner provides a small test implementation of the application frame and audio ports. It accepts generated frames and audio buffers but discards them, because conformance tests normally assert machine memory or CPU registers rather than host-visible output. This is different from a second, special emulator: it uses the same `Machine` and `Session` path as the SDL frontend.

## Scope

The no-op frame and audio sinks use `Infallible` as their associated error type.
The stepping helpers propagate that type; output delivery cannot produce a runner
failure. ROM loading errors, assertion failures, timeouts and captured core panics
remain separate outcomes.

The runner reports `StepResult::diagnostic` events to stderr. An illegal opcode
diagnostic does not change trial classification or exit-opcode detection: the
runner still uses the step's opcode and the existing cycle deadline/assertions.

Only eligible DMG tests with machine-readable memory or register expectations are registered. CGB/SGB, screenshot-only, and unsupported cartridge tests remain outside current emulator support. A failing ROM is useful evidence of a missing or inaccurate hardware behavior and is a natural starting point for the future debugger.
