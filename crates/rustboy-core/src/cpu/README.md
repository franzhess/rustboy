# CPU: SM83 instruction execution

This module implements the Game Boy's SM83 CPU inside
[`rustboy-core`](../../README.md). It is an 8-bit processor with a 16-bit address
space, related to the 8080/Z80 family but with its own instruction set and timing.
It owns registers and execution state, and borrows the machine-owned MMU while
executing. `Machine` coordinates device requests, T-cycle advancement and output.

## Source layout

| File | Responsibility |
| --- | --- |
| [`mod.rs`](mod.rs) | Step loop, fetch helpers, stack, interrupt dispatch, EI/HALT state |
| [`registers.rs`](registers.rs) | Registers, register-pair access, HL auto-increment/decrement |
| [`flags.rs`](flags.rs) | Concrete F-register storage and typed Z/N/H/C access |
| [`state.rs`](state.rs) | Execution-state enum and structured CPU diagnostics |
| [`alu.rs`](alu.rs) | Arithmetic/logic operations and their flag updates |
| [`opcodes.rs`](opcodes.rs) | Grouped base and CB-prefixed dispatch, shared decoding helpers, and instruction durations |
| [`operand.rs`](operand.rs) | Shared 8-bit register/indirect-HL operand decoding and access |
| [`timing_tests.rs`](timing_tests.rs) | Reference timings for every legal base/CB instruction |
| [`cb_tests.rs`](cb_tests.rs) | CB instruction effects, operand routing and memory-write behavior |
| [`base_tests.rs`](base_tests.rs) | Base instruction routing, loads, branches, stack effects, and special cases |

## Registers and operands

| Register | Role |
| --- | --- |
| A | Eight-bit accumulator |
| F | Flags: Z, N, H, C in bits 7, 6, 5, 4; low nibble is zero |
| B, C, D, E, H, L | General eight-bit registers |
| AF, BC, DE, HL | Sixteen-bit pairs; the first register supplies the high byte |
| PC | Address of the next instruction byte |
| SP | Stack pointer |

Z means zero, N records subtraction, H records half-carry/borrow, and C records
carry/borrow. Half-carry is relevant to nibble arithmetic and decimal adjustment;
its boundary depends on the operation.

`Registers` owns a concrete `Flags` value. `Flags::from_bits` masks input with
`0xF0`, and its private storage can only be changed through typed `CpuFlag`
setters or a reset. The low nibble therefore remains zero, including when
`Registers::set_af` loads F for POP AF. `Flags::bits()` exposes the masked byte
for AF packing and register snapshots.

ALU helpers and the CPU's operation function pointers take `&mut Flags`, giving
operations access to F without borrowing the other registers. Flag reads and
writes use concrete methods; there is no flag-register trait-object dispatch.
Conditional branches read the same `registers.flags` value.

`Registers::get16(&self, name)` reads AF, BC, DE, HL or SP without requiring mutable
access. `set16` routes writes to the corresponding pair setter, including AF's
flag masking. `get_hl()` likewise reads the pair without changing it.

`(HL)` denotes the byte in memory at the address held in HL, not the low byte of
HL. `post_increment_hl` and `post_decrement_hl` return the original address and
update HL with 16-bit wrapping arithmetic. Instructions such as `LD (HL+),A`
therefore use the old address for their memory access. These helpers update the
pair before returning; they do not model the hardware's bus-phase ordering.

Construction starts at PC `0x0100`, with AF=`01B0`, BC=`0013`, DE=`00D8`, HL=`014D`,
and SP=`FFFE`. These are current implementation defaults, not the result of an
emulated boot ROM or a guarantee of exact DMG post-boot state. IME starts clear.

## One machine step

`Machine` owns the CPU and MMU separately and performs this sequence:

1. Collect device interrupt requests into IF through the MMU.
2. Lend the MMU to `Cpu::tick(&mut mmu)`. The CPU checks for interrupts, then either
   performs interrupt entry, executes one instruction when running, or idles.
3. The CPU completes pending EI-delay bookkeeping after an executed instruction
   and returns cycles, opcode and optional diagnostic. It does not advance devices.
4. `Machine` advances the MMU's timer, PPU and APU by the returned T-cycle total.
5. `Machine` collects completed frames and audio buffers into the public step result.

Opcode handlers, fetch/stack helpers and `Operand8` receive an explicit bus borrow
where needed. Reads use `&Mmu`; writes use `&mut Mmu`. Pure register/ALU helpers
need no bus. The CPU no longer forwards input, memory inspection or device output.

The opcode handlers return `OpcodeResult::Executed(t_cycles)` or
`OpcodeResult::UnknownOpcode`. The public machine returns the executed **base**
opcode; CB instructions report `0xCB`.
Interrupt entry and all idle states return no opcode. An awake `next_opcode()` peek can
be superseded by an interrupt and must not be used as an execution trace.

All legal opcode encodings are dispatched. The eleven illegal base encodings
return `UnknownOpcode`; the step loop enters `CpuState::IllegalOpcode`, consumes
four cycles and returns `CpuDiagnostic::IllegalOpcode { address, opcode }`. The
address is captured before the fetch, including when PC wraps. The encounter step
still reports `Some(opcode)`; later idle steps report neither opcode nor diagnostic.
The CLI and ROM runner own diagnostic output rather than the CPU.

### Execution states

| `CpuState` | Entry | Current stepping behavior |
| --- | --- | --- |
| `Running` | Construction or wake-up | Execute instructions or service interrupts |
| `Halted` | HALT without the HALT-bug condition | Idle for 4 T-cycles until an enabled request |
| `Stopped` | STOP, after consuming its padding byte | Currently the same idle/wake behavior as HALT |
| `IllegalOpcode` | An illegal base opcode | Currently the same idle/wake behavior as HALT, with a one-shot diagnostic |

`Machine::cpu_state()` exposes these states. IME, pending EI enable and HALT-bug
fetch suppression remain separate controls. In particular, HALT with the bug
condition leaves the state `Running` and sets fetch suppression.

All current idle states keep advancing devices. On an enabled pending request,
IME set leads to interrupt entry; IME clear resumes normal execution. Retaining
that behavior makes this a structural refactor. STOP's hardware power/wake rules
and permanent illegal-opcode lockup require separately tested behavior changes.

## Opcode fields and groups

Both opcode spaces are decoded in `opcodes.rs`. `execute` dispatches base opcodes
to `execute_misc`, `execute_load`, `execute_alu`, or `execute_control`. The `0xCB`
prefix fetches a second byte and passes it to `execute_cb` in the same module.
The two mixed subgroups in `execute_misc` have named helpers for relative branches
and accumulator operations. `execute_control` uses a flat `(z, y)` match so each
special encoding is visible alongside its mnemonic, instead of nested selectors.

```text
bit positions:  7 6 | 5 4 3 | 2 1 0
field:           x |   y   |   z

x = opcode >> 6
y = (opcode >> 3) & 7
z = opcode & 7
p = y >> 1       (bits 5–4, for register-pair families)
q = y & 1        (bit 3, selects one of two related operations)
```

Shared field mappings:

| Field | Order |
| --- | --- |
| `r[0..7]` | B, C, D, E, H, L, `(HL)`, A (`Operand8`) |
| `rp[0..3]` | BC, DE, HL, SP (`register_pair`) |
| `rp2[0..3]` | BC, DE, HL, AF (`stack_pair`) |
| `cc[0..3]` | NZ, Z, NC, C (`condition_holds`) |
| `ALU[0..7]` | ADD, ADC, SUB, SBC, AND, XOR, OR, CP (`apply_alu`) |

`d8`/`d16` are immediate data, `a16` an immediate address, `a8` an offset into
`FF00–FFFF`, and `e8` a signed displacement. Words are fetched little-endian.

### Base groups

| `x` | Opcodes | Group |
| --- | --- | --- |
| 0 | `00–3F` | Miscellaneous, immediate loads, INC/DEC, word arithmetic, relative branches |
| 1 | `40–7F` | `LD r[y],r[z]`, except `76` is HALT |
| 2 | `80–BF` | `ALU[y] A,r[z]` |
| 3 | `C0–FF` | Control flow, stack, immediate ALU, high-memory loads and special instructions |

For `x=1`, source data is read before the destination is changed. This preserves
the original HL address in instructions such as `LD H,(HL)`. HALT is handled
before either operand is accessed; it is not a memory-to-memory load.

#### `x=0`: miscellaneous and load/arithmetic families

| `z` | Subgroups |
| --- | --- |
| 0 | `y=0`: NOP; `1`: `LD (a16),SP`; `2`: STOP; `3`: `JR e8`; `4–7`: `JR cc[y-4],e8` |
| 1 | `q=0`: `LD rp[p],d16`; `q=1`: `ADD HL,rp[p]` |
| 2 | `p` selects BC, DE, HL+, HL−; `q=0` stores A through that address; `q=1` loads A |
| 3 | `q=0`: `INC rp[p]`; `q=1`: `DEC rp[p]` |
| 4 | `INC r[y]` |
| 5 | `DEC r[y]` |
| 6 | `LD r[y],d8` |
| 7 | `y=0..7`: RLCA, RRCA, RLA, RRA, DAA, CPL, SCF, CCF |

HL+ and HL− use the original address and update HL with wraparound. Word INC/DEC
preserve flags. Accumulator rotations clear Z, unlike their CB counterparts.

#### `x=3`: control flow and special instructions

| `z` | Subgroups |
| --- | --- |
| 0 | `y=0–3`: `RET cc[y]`; `4`: `LDH (a8),A`; `5`: `ADD SP,e8`; `6`: `LDH A,(a8)`; `7`: `LD HL,SP+e8` |
| 1 | `q=0`: `POP rp2[p]`; `q=1,p=0..3`: RET, RETI, `JP HL`, `LD SP,HL` |
| 2 | `y=0–3`: `JP cc[y],a16`; `4`: `LD (FF00+C),A`; `5`: `LD (a16),A`; `6`: `LD A,(FF00+C)`; `7`: `LD A,(a16)` |
| 3 | `y=0`: `JP a16`; `1`: CB prefix; `6`: DI; `7`: EI; `2–5`: illegal |
| 4 | `y=0–3`: `CALL cc[y],a16`; `4–7`: illegal |
| 5 | `q=0`: `PUSH rp2[p]`; `q=1,p=0`: `CALL a16`; other `q=1` slots: illegal |
| 6 | `ALU[y] A,d8` |
| 7 | `RST y*8` |

The eleven illegal slots are `D3 DB DD E3 E4 EB EC ED F4 FC FD`; they return
`UnknownOpcode`. Special cases retain their existing EI/DI/RETI/HALT/STOP behavior.
Conditional branches preserve the current implementation's fetch behavior:
untaken JR/JP/CALL advance PC over the immediate operand without reading it.
This is an instruction-level model, not a claim about hardware bus ordering.

### CB-prefixed groups

All 256 CB encodings are valid. The same `x/y/z` fields describe:

| `x` | Opcodes | Meaning of `y` | Operation on `r[z]` |
| --- | --- | --- | --- |
| 0 | `00–3F` | Operation selector | RLC, RRC, RL, RR, SLA, SRA, SWAP, SRL |
| 1 | `40–7F` | Bit index | BIT |
| 2 | `80–BF` | Bit index | RES |
| 3 | `C0–FF` | Bit index | SET |

`Operand8` reads and writes either the selected register or memory through the
MMU. BIT reads without writeback; the other groups write their result even when
it equals the original byte. This distinction matters for memory-mapped registers
with write side effects. RES/SET leave all flags untouched. Rotations use the CB
ALU variants, which set Z from the result. Durations remain 8 T-cycles for register
operands, 12 for BIT `(HL)`, and 16 for other `(HL)` operations.

### Worked examples

- `41` = `01 000 001`: `x=1`, destination B, source C → `LD B,C` (4 T-cycles).
- `86` = `10 000 110`: `x=2`, ADD, source `(HL)` → `ADD A,(HL)` (8 T-cycles).
- `C4` = `11 000 100`: `x=3,z=4`, condition NZ → `CALL NZ,a16`
  (24 T-cycles taken, 12 not taken).
- `CB 7E`: second byte `01 111 110` → `BIT 7,(HL)` (12 T-cycles including prefix).

## Fetching, branches, and the stack

- `fetch_byte` reads at PC and normally increments PC with 16-bit wraparound.
- `fetch_word` reads little-endian data and advances PC by two, also wrapping.
- `JR` sign-extends its eight-bit displacement and adds it to PC **after** fetching
  the operand. The cast through `i8` and `i16` preserves negative offsets before
  `wrapping_add` applies them in the 16-bit address space.
- `CALL` saves the PC after its immediate address has been fetched; that is the
  return address. `return_from_call` restores PC by popping the stack; it is used
  by `RET`, taken conditional returns, and `RETI`. The `RETI` handler also enables
  IME immediately.
- The stack grows downward: push subtracts two from SP and writes a little-endian
  word; pop reads the word and adds two. These are aggregate operations, not a
  model of the hardware's individual stack-write bus phases.

For conditional JR/JP/CALL/RET, Z and C determine whether the branch is taken.
Both paths consume their instruction operands, but their durations differ.

## Instruction timing

The master clock is 4,194,304 Hz. One M-cycle is four T-cycles; all durations in
the implementation are **T-cycles**, including the opcode fetch and CB prefix.

| Example | T-cycles |
| --- | --- |
| NOP, register-to-register LD, ADD A,B | 4 |
| ADD A,(HL), LD (HL+),A | 8 |
| JP a16 | 16 |
| JR cc | 8 not taken / 12 taken |
| JP cc | 12 not taken / 16 taken |
| CALL cc | 12 not taken / 24 taken |
| RET cc | 8 not taken / 20 taken |
| CB operation on a register | 8 |
| CB BIT on `(HL)` | 12 |
| Other CB operations on `(HL)` | 16 |

The `(HL)` CB operations need memory access beyond the prefix/opcode fetches.
BIT reads but does not write; rotates/shifts/RES/SET read and write the operand.
This explains the 12-versus-16-cycle distinction.

Devices advance **after** the instruction's state changes and memory accesses.
The CPU is T-cycle-accounted, not M-cycle-executed: correct instruction totals
do not establish correct ordering of bus reads/writes within an instruction.

## Interrupts

The CPU checks `IE & IF & 0x1F`. IE is at `FFFF`, IF at `FF0F`; IME is a separate
internal master-enable flag, not a bit in either register.

Before this check, the MMU consumes the PPU's VBlank/STAT requests and the timer's
request through their `take_*` methods and ORs them into IF. Collection is independent
of IE and IME. Device-local flags are private; consuming them does not acknowledge
IF, so other pending requests survive until serviced or cleared by software.

| Bit | Interrupt | Vector |
| --- | --- | --- |
| 0 | VBlank | `0040` |
| 1 | LCD STAT | `0048` |
| 2 | Timer | `0050` |
| 3 | Serial | `0058` |
| 4 | Joypad | `0060` |

The lowest numbered pending bit wins (`trailing_zeros` identifies it). Entry
clears IME and a pending EI enable, saves PC, acknowledges that IF bit, and jumps
to `0x40 + 8 * interrupt_number`. It consumes **20 T-cycles**. The handler's first
instruction executes on the next step. Serial and joypad vectors are recognized,
but the MMU does not yet forward their device requests into IF.

### EI, DI, and RETI

EI enables IME after the following instruction completes. Internally,
`ei_requested` counts instruction completions: EI starts at two, EI's own completion
reduces it to one, and the next instruction reduces it to zero. Repeated EI does
not restart an existing delay. DI and interrupt entry cancel it. RETI enables IME
immediately, allowing dispatch at the next instruction boundary.

### HALT and fetch suppression

Normally HALT waits for an enabled pending interrupt. A HALT idle step consumes
four cycles so devices can continue raising requests. With IME set, waking can
enter the interrupt; with IME clear, execution resumes without servicing it.

Executing HALT with IME clear and an enabled request already pending produces
the HALT bug: `halt_bug` suppresses **one opcode-fetch PC increment**. It does not
simply repeat an entire instruction. A one-byte instruction can execute twice;
a multi-byte instruction can consume its opcode again as its first operand.

EI followed by HALT with a request already pending has special handling: interrupt
entry saves the HALT address and clears fetch suppression before running the
handler. STOP consumes its padding byte and enters `Stopped`; its distinct
low-power/input-resume behavior is not implemented yet.

## Tests and current boundaries

```sh
cargo test -p rustboy-core cpu::
cargo test -p rustboy-core cpu::alu::tests
cargo test -p rustboy-core cpu::timing_tests
cargo test -p rustboy-core cpu::cb_tests
cargo test -p rustboy-core cpu::base_tests
```

CPU integration fixtures contain a real `Machine` with a synthetic cartridge;
instruction tests use `Machine::step`, exercising the production coordination path.
Separate boundary tests verify request collection before operand reads, device
advancement after CPU bus accesses, input routing without stepping, and CPU-only
execution against a borrowed bus without automatic device advancement.

The timing tests execute all **244 legal unprefixed instructions** and all **256
CB-prefixed instructions** across all sixteen flag combinations. That includes
both outcomes of every conditional branch. Illegal base opcodes are checked as
unsupported; they are not assigned hardware timing by the reference table.
Separate tests cover device advancement, address wrapping, interrupt priority,
EI/DI/RETI delays, HALT wake-up, and fetch suppression.

ALU semantic tests check results and flags independently of instruction durations:
all byte operand/flag combinations for arithmetic and logical operations, all
byte/flag combinations for rotations, shifts and flag control, and half-carry
boundaries for `ADD HL,rr`. DAA is checked against decimal arithmetic for all valid
two-digit BCD operand pairs plus selected non-BCD cases. Signed SP-addition tests
cover every signed offset at selected SP boundaries; instruction-level E8/F8 tests
also verify sign extension, destinations, preserved A, PC and cycle totals.
Dedicated flag tests cover all input bytes, per-flag updates and resets; register
tests verify that loading AF masks the unused bits without altering A.

CB semantic tests execute every opcode with all 256 operand values and all sixteen
flag combinations. They check destination and unaffected registers, WRAM, flags,
PC/SP, reported opcode, HALT state and cycle totals. Separate tests address DIV
through `(HL)` to detect unwanted BIT writes and missing read/modify/write stores,
including stores whose value is unchanged. These verify instruction-level effects,
not M-cycle bus ordering.

Base semantic tests cover the register-load matrix, all byte inputs/flags for
INC/DEC/immediate-load routing, ALU register/memory/immediate selection, word
operations, HL auto-update, conditional branch outcomes, stack pairs, RST vectors,
high-memory/absolute loads, accumulator operations, STOP and read-only ALU memory
access. These complement the exhaustive ALU tests and the existing EI/HALT tests.

Known remaining boundaries include reload-cycle TIMA/TMA write priority and IE
changes during interrupt-entry stack writes, which need finer bus scheduling.
Some HALT/interrupt ROMs still fail or time out. See the
[conformance notes](../../../../docs/rom-conformance.md) for recorded results.

References: [opcode tables](https://gbdev.io/gb-opcodes/optables/),
[CPU registers](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html), and
[interrupts](https://gbdev.io/pandocs/Interrupts.html).
