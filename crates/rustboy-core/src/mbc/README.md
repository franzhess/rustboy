# Cartridge memory bank controllers (MBCs)

This module is part of `rustboy-core`, not a separate crate. An MBC maps a cartridge's
ROM and RAM into the Game Boy CPU's limited address space. Games select banks by
writing to control registers in the ROM address range; these writes change the
mapping rather than modifying the ROM bytes.

## Common address map and interface

| CPU address | Cartridge function |
| --- | --- |
| `0000–3FFF` | Lower 16 KiB ROM window, usually bank 0 |
| `4000–7FFF` | Upper 16 KiB ROM window, usually switchable |
| `A000–BFFF` | 8 KiB external RAM window, or an RTC register on MBC3 |

Addresses elsewhere belong to the rest of the Game Boy hardware, not this module.
The MMU passes full CPU addresses to `read_rom`/`write_rom`. For `read_ram`/`write_ram`,
it subtracts `0xA000` first, so the mapper receives an offset in `0000–1FFF`.

A banked address is normally calculated as:

```text
ROM index = selected bank × 0x4000 + offset within the 16 KiB window
RAM index = selected bank × 0x2000 + offset within the 8 KiB window
```

### Cartridge headers

`Cartridge::from_bytes` returns `Result<Cartridge, RomLoadError>`. Oversized images,
images too short to read the cartridge-type byte, and unsupported cartridge types
produce typed validation errors. `RomLoadError` implements `Display` and
`std::error::Error`; its variants have no underlying source. Filesystem and ZIP
errors belong to the ROM adapter, which can wrap these cartridge errors in its own
error chain.

`Cartridge::from_bytes` selects the implementation using header byte `0x0147`:

| Type code | Hardware configuration | Rustboy implementation |
| --- | --- | --- |
| `00` | ROM only | `mbc0.rs` |
| `01`, `02`, `03` | MBC1; MBC1 + RAM; MBC1 + RAM + battery | `mbc1.rs` |
| `05`, `06` | MBC2 with internal RAM, without/with battery | `mbc2.rs` |
| `08`, `09` | ROM + RAM, without/with battery, no MBC | Not accepted |
| `0F–13` | MBC3 variants with optional RTC, RAM, and battery | Not accepted; `mbc3.rs` is a placeholder |
| `19–1B` | MBC5; MBC5 + RAM; MBC5 + RAM + battery | `mbc5.rs` |
| `1C–1E` | MBC5 + rumble, optionally RAM and battery | Accepted by `mbc5.rs`; rumble behavior is not implemented |

Header byte `0x0148` describes ROM capacity; `0x0149` describes external RAM capacity:

| RAM size code | Capacity |
| --- | --- |
| `00` | None |
| `01` | 2 KiB (legacy value) |
| `02` | 8 KiB / one bank |
| `03` | 32 KiB / four banks |
| `04` | 128 KiB / sixteen banks |
| `05` | 64 KiB / eight banks |

Only sizes supported by the particular mapper apply. MBC1 uses codes `01–03` and
requires a RAM-bearing cartridge type. MBC2 always has its own 512 four-bit cells;
its internal RAM capacity does not come from `0x0149`. The current MBC5 implementation
allocates sixteen RAM banks regardless of the header; header-based RAM sizing is
not yet implemented there. Battery type codes describe hardware persistence, not
automatic save-file support: these mappers do not load or save RAM themselves.

### Address-line masks and mirroring

On smaller ROM chips, some of the controller's address lines are not connected.
Selecting an oversized bank number therefore aliases an existing bank rather than
necessarily reading beyond the chip. MBC1 and MBC2 model this with `rom_mask`:

```text
rom_mask = next_power_of_two(ROM byte length) - 1
ROM index = calculated index & rom_mask
```

For a 64 KiB ROM, the mask is `0xFFFF`. Selecting bank 5 produces `0x14000` at the
start of the window; masking gives `0x4000`, which is bank 1. The mask is based on
the loaded image length. Rounding up gives truncated images a contiguous address
mask; bounds-checked accesses to missing bytes return `0xFF`.

MBC1 RAM uses the same principle with `ram.len() - 1`, after checking that RAM
exists and is enabled. Its supported RAM capacities are powers of two. A 2 KiB
chip mirrors within the 8 KiB CPU window, while an 8 KiB chip ignores RAM-bank bits.

## ROM only (`Mbc0`)

The CPU's `0000–7FFF` range directly addresses a ROM of up to 32 KiB. There are no
bank registers and ROM writes have no effect. Rustboy currently accepts only type
`00`; its external RAM reads return zero and writes are ignored.

## Standard MBC1

MBC1 combines a five-bit low-bank register (`bank1`) and a two-bit shared register
(`bank2`). It can address up to 128 ROM banks (2 MiB) and four RAM banks (32 KiB).
Actual cartridge wiring limits which ROM/RAM capacity combinations are available;
large-ROM boards commonly have at most one RAM bank.

### Control writes

| CPU address | Effect |
| --- | --- |
| `0000–1FFF` | Enable RAM when the value's low nibble is `0xA`; otherwise disable it |
| `2000–3FFF` | Set `bank1` from bits 0–4; remap zero to one |
| `4000–5FFF` | Set `bank2` from bits 0–1 |
| `6000–7FFF` | Select ROM mode (bit 0 clear) or RAM mode (bit 0 set) |

Both bank registers retain their values across mode changes. Writes to `bank2`
always update the same register; the mode determines how reads use it.

### Read mapping

| CPU window | ROM mode (0) | RAM mode (1) |
| --- | --- | --- |
| `0000–3FFF` | ROM bank 0 | ROM bank `bank2 << 5` (0, 32, 64, or 96) |
| `4000–7FFF` | ROM bank `(bank2 << 5) \| bank1` | Same mapping as ROM mode |
| `A000–BFFF` | RAM bank 0 | RAM bank `bank2` |

RAM mode does **not** discard the high ROM-bank bits. Disabled or absent RAM reads
as `0xFF`, and writes are ignored.

The zero-to-one substitution happens **before** masking disconnected ROM address
lines. For example, on a 16-bank ROM, writing `0x00` to `bank1` selects bank 1, but
writing `0x10` can select bank 0 after the disconnected high bit is masked away.
On a full-size standard MBC1 ROM, low-register zero remapping prevents selecting
banks 0, 32, 64, and 96 in the upper window; RAM mode can expose them in the lower window.

## MBC1M multicarts

MBC1M uses alternate wiring to divide a 1 MiB ROM into four 256 KiB game groups.
Each group contains sixteen 16 KiB banks. It uses the same control registers as
MBC1, but `bank2` supplies ROM bank bits 4–5 rather than 5–6.

`bank_shift` expresses that wiring difference:

| Configuration | `bank_shift` | Low-bank mask | Banks per group |
| --- | --- | --- | --- |
| Standard MBC1 | 5 | `(1 << 5) - 1 = 0x1F` | 32 |
| MBC1M | 4 | `(1 << 4) - 1 = 0x0F` | 16 |

The shared mapping formula is:

```text
upper ROM bank = (bank2 << bank_shift) | (bank1 & ((1 << bank_shift) - 1))
lower ROM bank = 0 in ROM mode, or bank2 << bank_shift in RAM mode
```

With `bank2 = 2` and `bank1 = 3`, standard MBC1 selects upper bank 67, while MBC1M
selects bank 35. MBC1M's lower window in RAM mode selects a game base: 0, 16, 32, or 48.

MBC1M still performs the five-bit register's zero-to-one substitution before
discarding bit 4. Consequently, writing `0x00` selects offset 1 within a game,
but writing `0x10` selects offset 0. Masking to four bits before zero substitution
would incorrectly make both writes select offset 1.

### Why the Nintendo logo is checked

MBC1M has no distinct cartridge-type code. Its main header looks like standard
MBC1, so Rustboy uses a heuristic to choose the wiring. The 48-byte Nintendo logo
normally occupies `0x0104–0x0133` of a Game Boy game header. Another copy at the
header position in bank 16 suggests another game beginning 256 KiB into the ROM.

Rustboy selects MBC1M when:

1. The ROM is exactly 1 MiB (64 banks).
2. The main header contains the expected logo.
3. Bank 16 contains the same logo at its header position.

The logo is a detection signature; it does not participate in address calculations.
This is not definitive hardware identification: modified logos can cause false
negatives, and a normal ROM containing both signatures can cause a false positive.
The Mooneye multicart ROM includes additional logos to support this heuristic.

## MBC2

MBC2 supports up to sixteen ROM banks (256 KiB) and contains **512 four-bit RAM
cells**, addressed as 512 bytes with only the low nibble writable.

| CPU address | Effect of a write |
| --- | --- |
| `0000–3FFF`, address bit 8 clear | RAM enable: low data nibble must equal `0xA` |
| `0000–3FFF`, address bit 8 set | ROM bank: low four data bits, with zero remapped to one |
| `4000–7FFF` | Ignored |

The distinction is address bit 8 (`address & 0x100`), **not** a split into `0000–1FFF`
and `2000–3FFF`. For example, `0x0000` and `0x2000` both control RAM, while `0x0100`
and `0x2100` both control the ROM bank.

- `0000–3FFF` always reads ROM bank 0.
- `4000–7FFF` reads the selected bank, with chip-size mirroring after zero remapping.
- `A000–BFFF` mirrors the internal RAM every 512 addresses (`offset & 0x1FF`).
- RAM writes store `value & 0x0F`; reads return the stored nibble ORed with `0xF0`.
- Disabled RAM reads as `0xFF` and ignores writes.

## MBC3 (hardware reference; not implemented)

MBC3 normally supports up to 128 ROM banks (2 MiB), four RAM banks (32 KiB), and
an optional real-time clock (RTC):

| CPU address | Effect of a write |
| --- | --- |
| `0000–1FFF` | Enable/disable RAM and RTC access |
| `2000–3FFF` | Select a seven-bit ROM bank; zero maps to one |
| `4000–5FFF` | Select RAM bank `00–03`, or RTC register `08–0C` |
| `6000–7FFF` | A `0` then `1` sequence latches RTC values for reading |

The lower ROM window stays at bank 0. The upper window selects a ROM bank.
`A000–BFFF` accesses the selected RAM bank or RTC register. RTC registers represent
seconds, minutes, hours, and a day counter with halt and carry flags.

Rustboy currently rejects MBC3 type codes, including the `0x10` cartridge used by
the RTC conformance test. Future RTC emulation must preserve the core's deterministic
timing boundary; the core must not directly read the host clock.

## MBC5

MBC5 supports up to 512 ROM banks (8 MiB) and sixteen RAM banks (128 KiB) on
non-rumble cartridges. ROM and RAM bank selection are independent; there is no
MBC1-style banking mode.

| CPU address | Hardware control |
| --- | --- |
| `0000–1FFF` | RAM enable/disable (`0x0A` enables access) |
| `2000–2FFF` | Low eight bits of the ROM bank number |
| `3000–3FFF` | Bit 0 supplies the ninth ROM-bank bit |
| `4000–5FFF` | RAM bank selection |
| `6000–7FFF` | No bank-control effect |

`0000–3FFF` reads bank 0, `4000–7FFF` reads the selected nine-bit ROM bank, and
`A000–BFFF` reads/writes the selected RAM bank. Unlike MBC1/MBC2, **ROM bank 0 is
valid in the upper window**. Each ROM-bank register write preserves the other
register's bits.

On rumble cartridges, RAM-selection bit 3 controls the motor, leaving bits 0–2
for eight RAM banks. Rustboy currently treats all accepted MBC5 types alike and
does not separate the rumble bit. Its RAM implementation also lacks header-based
capacity handling and returns zero on disabled reads.

The separate `fix/05-mbc5-rom-banking` review adds masking of the ninth-bit register
and disconnected ROM address lines; those corrections are not part of fix/04.

## Regression tests and references

Mapper unit tests construct synthetic cartridge images; external ROMs are not needed:

```sh
cargo test -p rustboy-core mbc::
```

With the external suites installed, run `cargo rom-tests -- mbc`. See
[`docs/rom-conformance.md`](../../../../docs/rom-conformance.md) for results and review history.

Hardware references:

- [Pan Docs: cartridge header](https://gbdev.io/pandocs/The_Cartridge_Header.html)
- [Pan Docs: MBC1](https://gbdev.io/pandocs/MBC1.html)
- [Pan Docs: MBC2](https://gbdev.io/pandocs/MBC2.html)
- [Pan Docs: MBC3](https://gbdev.io/pandocs/MBC3.html)
- [Pan Docs: MBC5](https://gbdev.io/pandocs/MBC5.html)
