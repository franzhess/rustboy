# CPU: SM83 instruction execution

This module implements the Game Boy's SM83 CPU inside
[`rustboy-core`](../../README.md). It is an 8-bit processor with a 16-bit address
space, related to the 8080/Z80 family but with its own instruction set and timing.
It owns the MMU and drives devices using emulated cycles, not host time.

## Source layout

| File | Responsibility |
| --- | --- |
| [`mod.rs`](mod.rs) | Step loop, fetch helpers, stack, interrupt dispatch, EI/HALT state |
| [`registers.rs`](registers.rs) | Registers, flags, register-pair access, HL auto-increment/decrement |
| [`alu.rs`](alu.rs) | Arithmetic/logic operations and their flag updates |
| [`opcodes.rs`](opcodes.rs) | Base opcode dispatch and instruction durations |
| [`opcodes_cb.rs`](opcodes_cb.rs) | CB-prefixed rotates, shifts, bit tests, resets, and sets |
| [`timing_tests.rs`](timing_tests.rs) | Reference timings for every legal base/CB instruction |

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
its boundary depends on the operation. Setting AF masks F with `0xF0` so POP AF
cannot set nonexistent flag bits.

`Registers::get16(&self, name)` reads BC, DE, HL or SP without requiring mutable
access. `get_hl()` likewise reads the pair without changing it.

`(HL)` denotes the byte in memory at the address held in HL, not the low byte of
HL. `post_increment_hl` and `post_decrement_hl` return the original address and
update HL with 16-bit wrapping arithmetic. Instructions such as `LD (HL+),A`
therefore use the old address for their memory access. These helpers update the
pair before returning; they do not model the hardware's bus-phase ordering.

Construction starts at PC `0x0100`, with AF=`01B0`, BC=`0013`, DE=`00D8`, HL=`014D`,
and SP=`FFFE`. These are current implementation defaults, not the result of an
emulated boot ROM or a guarantee of exact DMG post-boot state. IME starts clear.

## One CPU step

`Cpu::tick` performs the following sequence:

1. Collect device interrupt requests and check whether interrupt entry is needed.
2. If an interrupt is serviced, perform entry without executing a handler instruction.
3. Otherwise, execute one instruction when awake, or consume a HALT idle step.
4. Complete pending EI-delay bookkeeping after an executed instruction.
5. Advance the MMU's timer, PPU, and APU by the step's total T-cycles.

The opcode handlers return `OpcodeResult::Executed(t_cycles)` or
`OpcodeResult::UnknownOpcode`. The public machine returns the executed **base**
opcode; CB instructions report `0xCB`.
Interrupt entry and HALT idle return no opcode. An awake `next_opcode()` peek can
be superseded by an interrupt and must not be used as an execution trace.

All legal opcode encodings are dispatched. The eleven illegal base encodings
return `UnknownOpcode`; the step loop currently prints a diagnostic, enters its
halted state, and consumes four cycles. This fallback does not model hardware
illegal-opcode lockup semantics.

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
handler. STOP currently consumes its padding byte and reuses the halted state;
its distinct low-power/input-resume behavior is not implemented.

## Tests and current boundaries

```sh
cargo test -p rustboy-core cpu::
cargo test -p rustboy-core cpu::alu::tests
cargo test -p rustboy-core cpu::timing_tests
```

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

Known remaining boundaries include reload-cycle TIMA/TMA write priority and IE
changes during interrupt-entry stack writes, which need finer bus scheduling.
Some HALT/interrupt ROMs still fail or time out. See the
[conformance notes](../../../../docs/rom-conformance.md) for recorded results.

References: [opcode tables](https://gbdev.io/gb-opcodes/optables/),
[CPU registers](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html), and
[interrupts](https://gbdev.io/pandocs/Interrupts.html).
